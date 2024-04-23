package io.denormalized;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.apache.flink.api.common.serialization.SerializationSchema;

public class JoinedRecordSerializationSchema implements SerializationSchema<JoinedRecord> {

    private static final ObjectMapper mapper = new ObjectMapper();

    @Override
    public byte[] serialize(JoinedRecord record) {
        try {
            return mapper.writeValueAsBytes(record);
        } catch (Exception e) {
            throw new RuntimeException("Failed to serialize JoinedRecord to JSON", e);
        }
    }
}
