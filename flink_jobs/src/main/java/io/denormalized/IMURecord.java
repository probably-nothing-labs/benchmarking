package io.denormalized;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;

public class IMURecord {
    @JsonProperty("driver_id")
    private String driverId;

    @JsonProperty("imu_measurement")
    private ImuData imu;

    @JsonProperty("meta")
    private MetaData meta;

    @JsonProperty("occurred_at_ms")
    private long occurredAtMs;

    // Default constructor
    public IMURecord() {
        // Explicit default constructor
    }

    @JsonIgnoreProperties(ignoreUnknown = true)
    public static class ImuData {
        @JsonProperty("accelerometer")
        private Accelerometer accelerometer;

        @JsonProperty("gps")
        private Gps gps;

        @JsonProperty("gyroscope")
        private Gyroscope gyroscope;

        @JsonProperty("timestamp")
        private String timestamp;

        // Default constructor
        public ImuData() {
            // Explicit default constructor
        }

        // Inner classes with their default constructors and getters/setters
        @JsonIgnoreProperties(ignoreUnknown = true)
        public static class Accelerometer {
            private double x, y, z;

            // Default constructor
            public Accelerometer() {
                // Explicit default constructor
            }

            public double getX() { return x; }
            public void setX(double x) { this.x = x; }
            public double getY() { return y; }
            public void setY(double y) { this.y = y; }
            public double getZ() { return z; }
            public void setZ(double z) { this.z = z; }
        }

        @JsonIgnoreProperties(ignoreUnknown = true)
        public static class Gps {
            private double altitude, latitude, longitude, speed;

            // Default constructor
            public Gps() {
                // Explicit default constructor
            }

            public double getAltitude() { return altitude; }
            public void setAltitude(double altitude) { this.altitude = altitude; }
            public double getLatitude() { return latitude; }
            public void setLatitude(double latitude) { this.latitude = latitude; }
            public double getLongitude() { return longitude; }
            public void setLongitude(double longitude) { this.longitude = longitude; }
            public double getSpeed() { return speed; }
            public void setSpeed(double speed) { this.speed = speed; }
        }

        @JsonIgnoreProperties(ignoreUnknown = true)
        public static class Gyroscope {
            private double x, y, z;

            // Default constructor
            public Gyroscope() {
                // Explicit default constructor
            }

            public double getX() { return x; }
            public void setX(double x) { this.x = x; }
            public double getY() { return y; }
            public void setY(double y) { this.y = y; }
            public double getZ() { return z; }
            public void setZ(double z) { this.z = z; }
        }
    }

    @JsonIgnoreProperties(ignoreUnknown = true)
    public static class MetaData {
        @JsonProperty("nonsense")
        private String nonsense;

        // Default constructor
        public MetaData() {
            // Explicit default constructor
        }

        public String getNonsense() { return nonsense; }
        public void setNonsense(String nonsense) { this.nonsense = nonsense; }
    }

    // Getters and Setters
    public String getDriverId() { return driverId; }
    public void setDriverId(String driverId) { this.driverId = driverId; }
    public ImuData getImu() { return imu; }
    public void setImu(ImuData imu) { this.imu = imu; }
    public MetaData getMeta() { return meta; }
    public void setMeta(MetaData meta) { this.meta = meta; }
    public long getOccurredAtMs() { return occurredAtMs; }
    public void setOccurredAtMs(long occurredAtMs) { this.occurredAtMs = occurredAtMs; }
}
