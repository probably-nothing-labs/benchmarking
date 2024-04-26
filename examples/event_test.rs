extern crate benchmarking;
use benchmarking::events::{Event, Trip};
use benchmarking::imu_types::IMUMeasurement as Measurement;

use serde_json::{Map, Value};
use uuid::Uuid;

fn main() {
    let mut meta = Map::new();
    meta.insert(String::from("test"), Value::from(String::from("hello")));

    let t = Trip::Start {
        trip_id: Uuid::new_v4(),
        driver_id: Uuid::new_v4(),
        occurred_at_ms: 10,
        meta: meta.clone(),
    };

    println!("{}", serde_json::to_string_pretty(&t).unwrap());

    // let event = Event::Trip(t);
    let event = Event::IMUMeasurement {
        driver_id: Uuid::new_v4(),
        occurred_at_ms: 182,
        imu_measurement: Measurement::new(),
        meta: Map::new(),
    };

    match event {
        Event::Trip(t) => {
            println!("{}", serde_json::to_string_pretty(&t).unwrap());
        }
        Event::IMUMeasurement { .. } => {
            println!("{}", serde_json::to_string_pretty(&event).unwrap());
        }
    }

    let payload = Event::Trip(Trip::End {
        trip_id: Uuid::new_v4(),
        driver_id: Uuid::new_v4(),
        occurred_at_ms: 129,
        meta: meta.clone(),
    });

    println!("{}", serde_json::to_string_pretty(&payload).unwrap());
}
