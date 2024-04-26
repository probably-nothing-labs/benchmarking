use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use uuid::Uuid;

pub type Meta = Map<String, Value>;


#[derive(Serialize, Deserialize)]
#[serde(tag = "event_name")]
pub enum Trip {
    #[serde(rename = "TRIP_START")]
    Start {
        trip_id: Uuid,
        driver_id: Uuid,
        occurred_at_ms: u64,
        meta: Meta,
    },
    #[serde(rename = "TRIP_END")]
    End {
        trip_id: Uuid,
        driver_id: Uuid,
        occurred_at_ms: u64,
        meta: Meta,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Event {
    Trip(Trip),
    IMUMeasurement {
        driver_id: Uuid,
        occurred_at_ms: u64,
        imu_measurement: crate::imu_types::IMUMeasurement,
        meta: Meta,
    },
}
