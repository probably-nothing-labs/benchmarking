package io.denormalized;

import com.fasterxml.jackson.annotation.JsonProperty;

import java.util.List;

public class JoinedRecord {
    @JsonProperty("driver_id")
    private String driverId;

    @JsonProperty("trip_id")
    private String tripId;

    @JsonProperty("measurements")
    private List<IMURecord> measurements;

    // Default constructor
    public JoinedRecord() {
        // Explicit default constructor
    }

    public JoinedRecord(String driverId, String tripId, List<IMURecord> measurements) {
        this.driverId = driverId;
        this.tripId = tripId;
        this.measurements = measurements;
    }

    // Getters and Setters
    public String getDriverId() { return driverId; }
    public void setDriverId(String driverId) { this.driverId = driverId; }
    public String getTripId() { return tripId; }
    public void setTripId(String tripId) { this.tripId = tripId; }
    public List<IMURecord> getMeasurements() { return measurements; }
    public void setMeasurements(List<IMURecord> measurements) { this.measurements = measurements; }
    public String toString() {
        return String.format(">> driver_id: %s, tripId %s, num_measurements: %s", this.driverId, this.tripId, this.measurements.size());
    }
}
