package io.denormalized;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;

public record TripRecord(
        @JsonProperty("driver_id") String driverId,
        @JsonProperty("event_type") String eventType,
        @JsonProperty("meta") Meta meta,
        @JsonProperty("occurred_at_ms") long occurredAtMs,
        @JsonProperty("trip_id") String tripId
) {
    @JsonIgnoreProperties(ignoreUnknown = true)
    public record Meta(String[] nonsense) {}
}
