package io.denormalized;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;

// {"driver_id":"6a14ff58-ab3e-4eff-989c-65e201114e80","imu":{"accelerometer":{"x":0.5556259155273438,"y":-1.1763477325439453,"z":1.7439284324645996},"gps":{"altitude":911.7141723632812,"latitude":-12.958308219294679,"longitude":-42.68208847452988,"speed":80.06229400634766},"gyroscope":{"x":-0.008079607039690018,"y":-0.004127614200115204,"z":-0.0009696455672383308},"timestamp":"2024-04-22T17:53:47.676419Z"},"meta":{"nonsense":["MMMMMMMMMM"]},"occurred_at_ms":1713808427676}
public record IMURecord(
        @JsonProperty("driver_id") String driverId,
        @JsonProperty("imu") ImuData imu,
        @JsonProperty("meta") MetaData meta,
        @JsonProperty("occurred_at_ms") long occurredAtMs
) {

    @JsonIgnoreProperties(ignoreUnknown = true)
    public record ImuData(
            @JsonProperty("accelerometer") Accelerometer accelerometer,
            @JsonProperty("gps") Gps gps,
            @JsonProperty("gyroscope") Gyroscope gyroscope,
            @JsonProperty("timestamp") String timestamp
    ) {
    }

    @JsonIgnoreProperties(ignoreUnknown = true)
    public record Accelerometer(double x, double y, double z) {
    }

    @JsonIgnoreProperties(ignoreUnknown = true)
    public record Gps(double altitude, double latitude, double longitude, double speed) {
    }

    @JsonIgnoreProperties(ignoreUnknown = true)
    public record Gyroscope(double x, double y, double z) {
    }

    @JsonIgnoreProperties(ignoreUnknown = true)
    public record MetaData(String[] nonsense) {
    }
}