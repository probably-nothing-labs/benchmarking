use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::agent::Agent;
use crate::events;
use crate::producer::MsgProducer;

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
    producer: Arc<dyn MsgProducer>,
}

impl AgentRunner {
    pub fn new(num_agents: u64, producer: Arc<dyn MsgProducer>, config: SimulationConfig) -> Self {
        let agents = (0..num_agents)
            .map(|_| Agent::new(config.clone()))
            .collect();

        Self {
            agents,
            config,
            producer,
        }
    }

    async fn end_all_trips(&mut self) {
        let futures = self
            .agents
            .iter_mut()
            .map(|agent| async {
                if let Some(_) = agent.current_trip {
                    let key = agent.id.to_string();
                    let e = agent.end_trip().await;
                    let msg = serde_json::to_vec(&e).expect("Failed to serialize");

                    let _ = self.producer
                        .send(self.config.trips_topic.clone(), key, &msg)
                        .await
                        .map_err(map_kafka_send_err);
                }
            })
            .collect::<Vec<_>>();

        for future in futures {
            future.await;
        }
    }

    pub async fn run(mut self, duration: Duration) -> AgentRunnerResult {
        // tracing::info!(num_agents = self.agents.len(), "Starting Agent Traffic");
        // tracing::info!(config=?self.config, "Starting Agent Traffic");

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

                        let _ = self.producer
                            .send(topic, key, &msg)
                            .await
                            .map_err(map_kafka_send_err);
                        // ("Message not sent");
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

fn map_kafka_send_err(err: (rdkafka::error::KafkaError, rdkafka::message::OwnedMessage)) {
    let kafka_error = err.0;
    tracing::error!("{:?}", kafka_error);
}
