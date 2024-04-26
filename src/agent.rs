use rand::Rng;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::events;
use crate::imu_types::IMUMeasurement;
use crate::agent_runner::SimulationConfig;

fn get_timestamp_ms() -> u64 {
    let now = SystemTime::now();
    let since_the_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");

    since_the_epoch.as_millis() as u64
}

#[derive(Clone, Debug)]
pub struct IMUState {
    pub measurements_count: u64,
    pub next_action_time: Instant,
}

#[derive(Clone, Debug)]
pub struct Trip {
    pub id: Uuid,
    pub imu_count: u64,
    pub end_time: Instant,
}

pub struct Agent {
    pub id: Uuid,
    pub config: SimulationConfig,
    pub imu_state: IMUState,
    pub trip_start_at: Instant,
    pub current_trip: Option<Trip>,
    pub trip_count: u64,
}

impl Agent {
    pub fn new(config: SimulationConfig) -> Self {
        let mut rng = rand::thread_rng();
        let now = Instant::now();

        let delta = Duration::from_secs(rng.gen_range(0..=config.imu_delta.as_secs()));
        let first_imu_action_time = now + delta;

        let trip_delta = Duration::from_secs(rng.gen_range(0..=config.first_trip_delay_s.into()));
        let trip_start_at = now + trip_delta;

        Self {
            id: Uuid::new_v4(),
            config,
            imu_state: IMUState {
                measurements_count: 0,
                next_action_time: first_imu_action_time,
            },
            trip_start_at,
            current_trip: None,
            trip_count: 0,
        }
    }

    fn get_meta(&self) -> events::Meta {
        // populate the message with junk data to control the average message size
        let data = String::from_iter(std::iter::repeat('M').take(self.config.junk_data_size));

        let mut meta = serde_json::Map::new();
        meta.insert("nonsense".to_string(), serde_json::Value::from(data));

        meta
    }

    fn get_imu_message(&mut self) -> events::Event {
        if let Some(trip) = self.current_trip.as_mut() {
            trip.imu_count += 1;
        }

        events::Event::IMUMeasurement {
            driver_id: self.id,
            occurred_at_ms: get_timestamp_ms(),
            imu_measurement: IMUMeasurement::new(),
            meta: self.get_meta(),
        }
    }

    fn start_trip(&mut self) -> events::Event {
        let mut rng = rand::thread_rng();

        let now = Instant::now();
        let trip_length =
            rng.gen_range(self.config.trip_min_length_s..=self.config.trip_max_length_s);
        let end = now + Duration::from_secs(trip_length);

        let trip_id = Uuid::new_v4();
        let trip = Trip {
            id: trip_id,
            imu_count: 0,
            end_time: end,
        };
        self.current_trip = Some(trip);

        events::Event::Trip(events::Trip::Start {
            trip_id,
            driver_id: self.id,
            occurred_at_ms: get_timestamp_ms(),
            meta: self.get_meta(),
        })
    }

    pub async fn end_trip(&mut self) -> events::Event {
        let trip = self.current_trip.as_ref().unwrap();
        tracing::debug!(
            "driver {} ending trip {} imu count {}",
            self.id,
            trip.id,
            trip.imu_count
        );

        let payload = events::Event::Trip(events::Trip::End {
            trip_id: trip.id.clone(),
            driver_id: self.id.clone(),
            occurred_at_ms: get_timestamp_ms(),
            meta: self.get_meta(),
        });

        tracing::info!(
            target: crate::logging::LOG_TARGET,
            driver_id = ?self.id,
            trip_id = ?trip.id,
            imu_count = trip.imu_count,
            "trip end",
        );

        // Calculate when the next Trip for this agent should start
        let mut rng = rand::thread_rng();
        let now = Instant::now();

        let trip_delta = Duration::from_secs(
            rng.gen_range(self.config.trip_delay_min_s..=self.config.trip_delay_max_s),
        );
        let trip_start_at = now + trip_delta;

        self.trip_start_at = trip_start_at;
        self.current_trip = None;
        self.trip_count += 1;

        payload
    }

    pub async fn next(&mut self) -> Option<events::Event> {
        let now = Instant::now();
        if self.imu_state.next_action_time <= now {
            self.imu_state.next_action_time = now + self.config.imu_delta;
            self.imu_state.measurements_count += 1;

            return Some(self.get_imu_message());
        } else {
            if let Some(trip) = &self.current_trip {
                if trip.end_time <= now {
                    // Return trip end event
                    return Some(self.end_trip().await);
                } else {
                    // On a trip, but it isn't over
                    // Noop
                }
            } else if self.trip_start_at >= now {
                return Some(self.start_trip());
            }

            None
        }
    }
}
