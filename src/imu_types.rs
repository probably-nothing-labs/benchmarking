use rand::Rng;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AccelerometerMeasurement {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GyroscopeMeasurement {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GPSMeasurement {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f32,
    pub speed: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IMUMeasurement {
    pub timestamp: DateTime<Utc>,
    pub accelerometer: Option<AccelerometerMeasurement>,
    pub gyroscope: Option<GyroscopeMeasurement>,
    pub gps: Option<GPSMeasurement>,
}

impl IMUMeasurement {
    pub fn new() -> Self{
        let mut rng = rand::thread_rng();

        Self {
            timestamp: chrono::Utc::now(),
            accelerometer: Some(AccelerometerMeasurement {
                x: rng.gen_range(-2.0..2.0),
                y: rng.gen_range(-2.0..2.0),
                z: rng.gen_range(-2.0..2.0),
            }),
            gyroscope: Some(GyroscopeMeasurement {
                x: rng.gen_range(-0.01..0.01),
                y: rng.gen_range(-0.01..0.01),
                z: rng.gen_range(-0.01..0.01),
            }),
            gps: Some(GPSMeasurement {
                latitude: rng.gen_range(-90.0..90.0),
                longitude: rng.gen_range(-180.0..180.0),
                altitude: rng.gen_range(0.0..1000.0),
                speed: rng.gen_range(0.0..100.0),
            }),
        }
    }
}
