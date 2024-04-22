package io.denormalized;

import org.apache.flink.api.common.eventtime.WatermarkStrategy;
import org.apache.flink.api.common.state.ListState;
import org.apache.flink.api.common.state.ListStateDescriptor;
import org.apache.flink.api.common.state.MapState;
import org.apache.flink.api.common.state.MapStateDescriptor;
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


import java.util.List;
import java.util.ArrayList;


public class DataStreamJob {

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
                .keyBy(IMURecord::driverId);

        DataStream<TripRecord> tripStream = env.fromSource(tripSource, WatermarkStrategy.noWatermarks(), "Trip Source")
                .map(jsonString -> jsonParser.readValue(jsonString, TripRecord.class))
                .keyBy(TripRecord::driverId);

        DataStream<JoinedRecord> joinedStream = imuStream
                .connect(tripStream)
                .keyBy(IMURecord::driverId, TripRecord::driverId)
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
                                "tripStartState", String.class, Long.class));
                    }

                    @Override
                    public void processElement1(IMURecord imuValue, Context ctx, Collector<JoinedRecord> out) throws Exception {
                        if (tripStartState.contains(imuValue.driverId())) {
                            if (imuState.contains(imuValue.driverId())) {
                                List<IMURecord> records = imuState.get(imuValue.driverId());
                                records.add(imuValue);
                                imuState.put(imuValue.driverId(), records);
                            } else {
                                imuState.put(imuValue.driverId(), new ArrayList<>());
                            }
                        }
                    }

                    @Override
                    public void processElement2(TripRecord trip, Context ctx, Collector<JoinedRecord> out) throws Exception {
                        if ("TRIP_START".equals(trip.eventType())) {
                            tripStartState.put(trip.driverId(), trip.occurredAtMs());
                        } else if ("TRIP_END".equals(trip.eventType()) && tripStartState.contains(trip.driverId())) {
                            List<IMURecord> records = imuState.get(trip.driverId());
                            if (records != null) {
                                out.collect(new JoinedRecord(trip.driverId(), trip.tripId(), records));
                                imuState.remove(trip.driverId());
                            }
                            tripStartState.remove(trip.driverId());
                        }
                    }
                });

        // TODO: This pipeline fails because flink is not able to properly serialize the JoinedRecord class, how do I fix?
        joinedStream.print();

        env.execute("Flink Kafka Join Job");
    }
}
