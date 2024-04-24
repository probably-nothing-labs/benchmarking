use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::Mutex;

use crate::events;
use crate::imu_types::IMUMeasurement;
use crate::producer::Producer;

type WritableFile = Arc<Mutex<BufWriter<File>>>;

fn get_timestamp_ms() -> u64 {
    let now = SystemTime::now();
    let since_the_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");

    since_the_epoch.as_millis() as u64
}

#[derive(Clone, Debug)]
struct IMUState {
    measurements_count: u64,
    next_action_time: Instant,
}

#[derive(Clone, Debug)]
struct Trip {
    id: Uuid,
    imu_count: u64,
    start_time: Instant,
    end_time: Instant,
}

#[derive(Clone, Debug)]
pub struct SimulationConfig {
    pub junk_data_size: usize,

    pub imu_topic: String,
    pub imu_delta: Duration,

    pub trips_topic: String,
    pub first_trip_delay_s: u64, // max time till first trip is taken
    pub trip_delay_min_s: u64,
    pub trip_delay_max_s: u64,
    pub trip_min_length_s: u64,
    pub trip_max_length_s: u64,
}

struct Agent {
    id: Uuid,
    config: SimulationConfig,
    log_file: WritableFile,
    imu_state: IMUState,
    trip_start_at: Instant,
    current_trip: Option<Trip>,
    trip_count: u64,
}

impl Agent {
    fn new(config: SimulationConfig, log_file: WritableFile) -> Self {
        let mut rng = rand::thread_rng();
        let now = Instant::now();

        let delta = Duration::from_secs(rng.gen_range(0..=config.imu_delta.as_secs()));
        let first_imu_action_time = now + delta;

        let trip_delta = Duration::from_secs(rng.gen_range(0..=config.first_trip_delay_s.into()));
        let trip_start_at = now + trip_delta;

        Self {
            id: Uuid::new_v4(),
            log_file,
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
            start_time: now,
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

    async fn end_trip(&mut self) -> events::Event {
        let trip = self.current_trip.as_ref().unwrap();
        log::debug!(
            "driver {} ending trip {} imu count {}",
            self.id,
            trip.id,
            trip.imu_count
        );

        let payload = events::Event::Trip(events::Trip::End {
            trip_id: trip.id,
            driver_id: self.id,
            occurred_at_ms: get_timestamp_ms(),
            meta: self.get_meta(),
        });

        let entry = format!("{},{},{}\n", self.id, trip.id, trip.imu_count);
        self.write_log_entry(entry.as_bytes()).await;

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

    async fn write_log_entry(&self, line: &[u8]) {
        let mut file = self.log_file.lock().await;
        let _ = (*file).write(line).await;
    }

    async fn next(&mut self) -> Option<events::Event> {
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

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct AgentRunnerResult {
    pub imu_measurements_count: u64,
    pub total_trips: u64,
}

impl std::ops::Add for AgentRunnerResult {
    type Output = AgentRunnerResult;

    fn add(self, other: AgentRunnerResult) -> AgentRunnerResult {
        AgentRunnerResult {
            imu_measurements_count: self.imu_measurements_count + other.imu_measurements_count,
            total_trips: self.total_trips + other.total_trips,
        }
    }
}

impl std::ops::AddAssign for AgentRunnerResult {
    fn add_assign(&mut self, other: AgentRunnerResult) {
        self.imu_measurements_count += other.imu_measurements_count;
        self.total_trips += other.total_trips;
    }
}

pub struct AgentRunner {
    agents: Vec<Agent>,
    config: SimulationConfig,
    producer: Arc<dyn Producer>,
    log_file: WritableFile,
}

impl AgentRunner {
    pub fn new(
        num_agents: u64,
        producer: Arc<dyn Producer>,
        config: SimulationConfig,
        log_file: WritableFile,
    ) -> Self {
        let agents = (0..num_agents)
            .map(|_| Agent::new(config.clone(), Arc::clone(&log_file)))
            .collect();

        Self {
            agents,
            config,
            producer,
            log_file,
        }
    }

    async fn end_all_trips(&mut self) {
        log::debug!("Ending all trips");
        let futures = self
            .agents
            .iter_mut()
            .map(|agent| async {
                if let Some(_) = agent.current_trip {
                    let key = agent.id.to_string();
                    let e = agent.end_trip().await;
                    let msg = serde_json::to_vec(&e).expect("Failed to serialize");

                    self.producer
                        .send(self.config.trips_topic.clone(), key, &msg)
                        .await
                        .expect("Message not sent");
                }
            })
            .collect::<Vec<_>>();

        for future in futures {
            future.await;
        }
    }

    pub async fn run(mut self, duration: Duration) -> AgentRunnerResult {
        log::debug!("Starting Agent Traffic, {} agents", self.agents.len());
        log::debug!("{:?}", self.config);

        let mut current = Instant::now();
        let end = Instant::now() + duration;

        // Main loop, keep iterating until duration has passed
        while current <= end {
            let futures = self
                .agents
                .iter_mut()
                .map(|agent| async {
                    if let Some(e) = agent.next().await {
                        let topic = match e {
                            events::Event::IMUMeasurement { .. } => self.config.imu_topic.clone(),
                            events::Event::Trip(_) => self.config.trips_topic.clone(),
                        };

                        let key = agent.id.to_string();
                        let msg = serde_json::to_vec(&e).expect("Failed to serialize");

                        self.producer
                            .send(topic, key, &msg)
                            .await
                            .expect("Message not sent");
                    } else {
                    }
                })
                .collect::<Vec<_>>();

            for future in futures {
                future.await;
            }

            tokio::time::sleep(Duration::from_millis(1)).await;
            current = Instant::now();
        }

        self.end_all_trips().await;

        let imu_measurements_count = self
            .agents
            .iter()
            .map(|agent| agent.imu_state.measurements_count)
            .sum();

        let total_trips = self.agents.iter().map(|agent| agent.trip_count).sum();

        AgentRunnerResult {
            imu_measurements_count,
            total_trips,
        }
    }
}
