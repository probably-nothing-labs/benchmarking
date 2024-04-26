package io.denormalized;

import org.apache.flink.api.common.eventtime.WatermarkStrategy;
import org.apache.flink.api.common.state.ListState;
import org.apache.flink.api.common.state.ListStateDescriptor;
import org.apache.flink.api.common.state.MapState;
import org.apache.flink.api.common.state.MapStateDescriptor;
import org.apache.flink.connector.kafka.sink.KafkaRecordSerializationSchema;
import org.apache.flink.connector.kafka.sink.KafkaSink;
import org.apache.flink.streaming.api.datastream.DataStream;
import org.apache.flink.streaming.api.environment.StreamExecutionEnvironment;
import org.apache.flink.streaming.api.functions.co.KeyedCoProcessFunction;
import org.apache.flink.util.Collector;
import org.apache.flink.api.common.serialization.SimpleStringSchema;
import org.apache.flink.connector.kafka.source.KafkaSource;
import org.apache.flink.connector.kafka.source.enumerator.initializer.OffsetsInitializer;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.apache.flink.configuration.Configuration;
import org.apache.flink.api.common.typeinfo.TypeInformation;
import org.apache.flink.api.common.typeinfo.TypeHint;
import org.apache.flink.api.common.typeinfo.BasicTypeInfo;
import org.apache.flink.api.java.typeutils.ListTypeInfo;


import java.util.Collections;
import java.util.List;
import java.util.ArrayList;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class DataStreamJob {
    private static final Logger LOG = LoggerFactory.getLogger(DataStreamJob.class);

    public static void main(String[] args) throws Exception {
        final StreamExecutionEnvironment env = StreamExecutionEnvironment.getExecutionEnvironment();

        ObjectMapper jsonParser = new ObjectMapper();


        String bootstrapServers = System.getenv("BOOTSTRAP_SERVERS");
        if (bootstrapServers == null) {
            System.out.println("BOOTSTRAP_SERVERS environment variable is not set.");
            return;
        }

        KafkaSource<String> imuSource = KafkaSource.<String>builder()
                .setBootstrapServers(bootstrapServers)
                .setTopics("driver-imu-data")
                .setGroupId("flink-imu-group")
                .setValueOnlyDeserializer(new SimpleStringSchema())
                .setStartingOffsets(OffsetsInitializer.earliest())
                .build();

        KafkaSource<String> tripSource = KafkaSource.<String>builder()
                .setBootstrapServers(bootstrapServers)
                .setTopics("trips")
                .setGroupId("flink-trip-group")
                .setValueOnlyDeserializer(new SimpleStringSchema())
                .setStartingOffsets(OffsetsInitializer.earliest())
                .build();

        DataStream<IMURecord> imuStream = env.fromSource(imuSource, WatermarkStrategy.noWatermarks(), "IMU Source")
                .map(jsonString -> jsonParser.readValue(jsonString, IMURecord.class))
                .keyBy(IMURecord::getDriverId);

        DataStream<TripRecord> tripStream = env.fromSource(tripSource, WatermarkStrategy.noWatermarks(), "Trip Source")
                .map(jsonString -> jsonParser.readValue(jsonString, TripRecord.class))
                .keyBy(TripRecord::getDriverId);

        DataStream<JoinedRecord> joinedStream = imuStream
                .connect(tripStream)
                .keyBy(IMURecord::getDriverId, TripRecord::getDriverId)
                .process(new KeyedCoProcessFunction<String, IMURecord, TripRecord, JoinedRecord>() {
                    private transient MapState<String, List<IMURecord>> imuState;
                    private transient MapState<String, Long> tripStartState;

                    @Override
                    public void open(Configuration parameters) {
                        MapStateDescriptor<String, List<IMURecord>> desc = new MapStateDescriptor<>(
                                "imuState",
                                BasicTypeInfo.STRING_TYPE_INFO,
                                new ListTypeInfo<>(IMURecord.class)
                        );
                        imuState = getRuntimeContext().getMapState(desc);

                        tripStartState = getRuntimeContext().getMapState(new MapStateDescriptor<>(
                                "tripStartState", BasicTypeInfo.STRING_TYPE_INFO, BasicTypeInfo.LONG_TYPE_INFO));
                    }

                    @Override
                    public void processElement1(IMURecord imuValue, Context ctx, Collector<JoinedRecord> out) throws Exception {
                        if (tripStartState.contains(imuValue.getDriverId())) {
                            if (imuState.contains(imuValue.getDriverId())) {
                                List<IMURecord> records = imuState.get(imuValue.getDriverId());
                                records.add(imuValue);
                                imuState.put(imuValue.getDriverId(), records);
                            } else {
                                imuState.put(imuValue.getDriverId(), new ArrayList<>(Collections.singletonList(imuValue)));
                            }
                        }
                    }

                    @Override
                    public void processElement2(TripRecord trip, Context ctx, Collector<JoinedRecord> out) throws Exception {
                        if ("TRIP_START".equals(trip.getEventType())) {
                            tripStartState.put(trip.getDriverId(), trip.getOccurredAtMs());
                        } else if ("TRIP_END".equals(trip.getEventType()) && tripStartState.contains(trip.getDriverId())) {
                            List<IMURecord> records = imuState.get(trip.getDriverId());
                            if (records != null && !records.isEmpty()) {
                                out.collect(new JoinedRecord(trip.getDriverId(), trip.getTripId(), records));
                                imuState.remove(trip.getDriverId());
                            }
                            tripStartState.remove(trip.getDriverId());
                        }
                    }
                });

        KafkaRecordSerializationSchema<JoinedRecord> serializer = KafkaRecordSerializationSchema.builder()
                .setTopic("results")
                .setValueSerializationSchema(new JoinedRecordSerializationSchema())
                .build();

        KafkaSink<JoinedRecord> kafkaSink = KafkaSink.<JoinedRecord>builder()
                .setBootstrapServers(bootstrapServers)
                .setRecordSerializer(serializer)
                .build();

        joinedStream.sinkTo(kafkaSink);

        env.execute("Flink Kafka Join Job");
    }
}
