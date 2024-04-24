extern crate benchmarking;
use benchmarking::events::{Event, Trip};
use benchmarking::imu_types::IMUMeasurement as Measurement;

use uuid::Uuid;
use serde_json::{Map, Value};

fn main() {
    let mut meta = Map::new();
    meta.insert(String::from("test"), Value::from(String::from("hello")));

    let t = Trip::Start {
        trip_id: Uuid::new_v4(),
        driver_id: Uuid::new_v4(),
        occurred_at_ms: 10,
        meta,
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
        Event::IMUMeasurement{..} => {
            println!("{}", serde_json::to_string_pretty(&event).unwrap());
        } 
    }
}
