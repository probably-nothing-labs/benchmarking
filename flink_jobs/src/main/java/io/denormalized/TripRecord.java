package io.denormalized;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;

public class TripRecord {
    @JsonProperty("driver_id")
    private String driverId;

    @JsonProperty("event_name")
    private String eventType;

    @JsonProperty("meta")
    private Meta meta;

    @JsonProperty("occurred_at_ms")
    private long occurredAtMs;

    @JsonProperty("trip_id")
    private String tripId;

    // Default constructor
    public TripRecord() {
        // Explicit default constructor
    }

    @JsonIgnoreProperties(ignoreUnknown = true)
    public static class Meta {
        @JsonProperty("nonsense")
        private String nonsense;

        // Default constructor
        public Meta() {
            // Explicit default constructor
        }

        // Getters and Setters
        public String getNonsense() { return nonsense; }
        public void setNonsense(String nonsense) { this.nonsense = nonsense; }
    }

    // Getters and Setters
    public String getDriverId() { return driverId; }
    public void setDriverId(String driverId) { this.driverId = driverId; }
    public String getEventType() { return eventType; }
    public void setEventType(String eventType) { this.eventType = eventType; }
    public Meta getMeta() { return meta; }
    public void setMeta(Meta meta) { this.meta = meta; }
    public long getOccurredAtMs() { return occurredAtMs; }
    public void setOccurredAtMs(long occurredAtMs) { this.occurredAtMs = occurredAtMs; }
    public String getTripId() { return tripId; }
    public void setTripId(String tripId) { this.tripId = tripId; }
}
