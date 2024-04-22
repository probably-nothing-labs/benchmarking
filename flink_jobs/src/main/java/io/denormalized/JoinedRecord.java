package io.denormalized;

import com.fasterxml.jackson.annotation.JsonProperty;

import java.util.List;

public record JoinedRecord(
        @JsonProperty("driver_id") String driverId,
        @JsonProperty("trip_id") String tripId,
        @JsonProperty("measurements") List<IMURecord> measurements
) {
}