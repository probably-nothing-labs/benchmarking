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


public class DataStreamJob {

	public static void main(String[] args) throws Exception {
		final StreamExecutionEnvironment env = StreamExecutionEnvironment.getExecutionEnvironment();
		ObjectMapper jsonParser = new ObjectMapper();

		String bootstrapServers = System.getenv("BOOTSTRAP_SERVERS");
		if (bootstrapServers == null) {
			System.out.println("BOOTSTRAP_SERVERS environment variable is not set.");
			return;
		}

		// Sample data:
		// {"driver_id":"6a14ff58-ab3e-4eff-989c-65e201114e80","imu":{"accelerometer":{"x":0.5556259155273438,"y":-1.1763477325439453,"z":1.7439284324645996},"gps":{"altitude":911.7141723632812,"latitude":-12.958308219294679,"longitude":-42.68208847452988,"speed":80.06229400634766},"gyroscope":{"x":-0.008079607039690018,"y":-0.004127614200115204,"z":-0.0009696455672383308},"timestamp":"2024-04-22T17:53:47.676419Z"},"meta":{"nonsense":["MMMMMMMMMM"]},"occurred_at_ms":1713808427676}
		KafkaSource<String> imuSource = KafkaSource.<String>builder().setBootstrapServers(bootstrapServers)
				.setTopics("driver-imu-data").setGroupId("flink-imu-group")
				.setValueOnlyDeserializer(new SimpleStringSchema()).setStartingOffsets(OffsetsInitializer.earliest())
				.build();

		// Sample data:
		// {
		// "driver_id": "753fe907-8d10-4985-a220-b7fa07d6416a",
		// "event_type": "TRIP_START",
		// "meta": {
		// "nonsense": [
		// "MMMMMMMMMM"
		// ]
		// },
		// "occurred_at_ms": 1713808582350,
		// "trip_id": "b0b044e7-a7b1-4281-bbe6-5fc377ff104c"
		// },
		// {
		// "driver_id": "753fe907-8d10-4985-a220-b7fa07d6416a",
		// "event_type": "TRIP_END",
		// "meta": {
		// "nonsense": [
		// "MMMMMMMMMM"
		// ]
		// },
		// "occurred_at_ms": 1713808612352,
		// "trip_id": "b0b044e7-a7b1-4281-bbe6-5fc377ff104c"
		// }
		KafkaSource<String> tripSource = KafkaSource.<String>builder().setBootstrapServers(bootstrapServers)
				.setTopics("trips").setGroupId("flink-trip-group").setValueOnlyDeserializer(new SimpleStringSchema())
				.setStartingOffsets(OffsetsInitializer.earliest()).build();

		DataStream<JsonNode> imuStream = env.fromSource(imuSource, WatermarkStrategy.noWatermarks(), "IMU Source")
				.map(jsonParser::readTree).keyBy(json -> json.get("driver_id").asText());

		DataStream<JsonNode> tripStream = env.fromSource(tripSource, WatermarkStrategy.noWatermarks(), "Trip Source")
				.map(jsonParser::readTree).keyBy(json -> json.get("driver_id").asText());

		// TODO: join imuSource and tripSource using a session window join.
		// The session window opens when tripSource receives a TRIP_START event and ends
		// which it receives a TRIP_END event. all imu data recieved from the driver of
		// the trip designated by trip_id should be grouped grouped into a single record
		// that is emitted downstream, along with the driver_id and trip_id, after the
		// TRIP_END event is received and the window is closed.

		tripStream.connect(imuStream).process(new KeyedCoProcessFunction<String, JsonNode, JsonNode, String>() {
			private transient MapState<String, ListState<JsonNode>> imuData;

			@Override
			public void open(Configuration config) {
				imuData = getRuntimeContext().getMapState(new MapStateDescriptor<>("imuData", String.class,
						new ListStateDescriptor<>("imuRecords", JsonNode.class)));
			}

			@Override
			public void processElement1(JsonNode tripEvent, Context ctx, Collector<String> out) throws Exception {
				String tripId = tripEvent.get("trip_id").asText();
				String eventType = tripEvent.get("event_type").asText();

				if (eventType.equals("TRIP_START")) {
					// Initialize the list state for this trip
					imuData.put(tripId, getRuntimeContext()
							.getListState(new ListStateDescriptor<>("imuRecordsForTrip", JsonNode.class)));
				} else if (eventType.equals("TRIP_END") && imuData.contains(tripId)) {
					StringBuilder result = new StringBuilder();
					result.append("{ \"trip_id\": \"").append(tripId).append("\", \"driver_id\": \"")
							.append(tripEvent.get("driver_id").asText()).append("\", \"imu_data\": [");
					boolean first = true;
					for (JsonNode imu : imuData.get(tripId).get()) {
						if (!first) {
							result.append(", ");
						}
						result.append(imu.toString());
						first = false;
					}
					result.append("] }");
					out.collect(result.toString());
					imuData.remove(tripId);
				}
			}

			@Override
			public void processElement2(JsonNode imuDataEvent, Context ctx, Collector<String> out) throws Exception {
				String driverId = imuDataEvent.get("driver_id").asText();
				for (String key : imuData.keys()) {
					ListState<JsonNode> list = imuData.get(key);
					if (list != null) {
						list.add(imuDataEvent);
					}
				}
			}
		}).print();
		env.execute("Flink Kafka Join Job");
	}
}
