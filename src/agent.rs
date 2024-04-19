use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use uuid::Uuid;

use std::sync::Arc;

use crate::imu_types::IMUMeasurement;
use crate::producer::Producer;

pub struct Agent {
    id: Uuid,
    imu_measurements_count: u32,
    imu_delta: Duration,
    next_action_time: Instant,
}

impl Agent {
    pub fn new(imu_delta: Duration) -> Self {
        let mut rng = rand::thread_rng();

        let now = Instant::now();
        let delta = Duration::new(rng.gen_range(0..imu_delta.as_secs()), 0);

        let first_action_time = now + delta;

        Self {
            id: Uuid::new_v4(),
            imu_measurements_count: 0,
            imu_delta,
            next_action_time: first_action_time,
        }
    }

    pub fn get_imu_message(&self, junk_data_size: usize) -> Vec<u8> {
        // populate the message with junk data to control the average message size
        let nonesense_str = String::from_iter(std::iter::repeat('M').take(junk_data_size));

        let json_payload = serde_json::json!({
            "driver_id": self.id,
            "imu": IMUMeasurement::new(),
            "meta": {"nonsense": [nonesense_str]}
        });

        let payload = serde_json::to_vec(&json_payload).expect("Failed to serialize");

        payload
    }

    pub fn start_trip() {}

    pub fn next(&mut self) -> Option<Vec<u8>> {
        let now = Instant::now();
        if self.next_action_time >= now {
            self.next_action_time = now + self.imu_delta;
            self.imu_measurements_count += 1;

            Some(self.get_imu_message(10_000))
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AgentRunnerResult {
    pub imu_measurements_count: u32,
}

pub struct AgentRunner {
    agents: Vec<Agent>,
    producer: Arc<dyn Producer>,
    imu_topic: String,
    imu_duration: Duration,
}

impl AgentRunner {
    pub fn new(
        num_agents: u32,
        producer: Arc<dyn Producer>,
        imu_topic: String,
        imu_duration: Duration,
    ) -> Self {
        let agents = (0..num_agents)
            .map(|_| Agent::new(imu_duration.clone()))
            .collect();
        Self {
            agents,
            producer,
            imu_topic,
            imu_duration,
        }
    }

    pub async fn run(mut self, duration: Duration) -> AgentRunnerResult {
        let mut current = Instant::now();
        let end = Instant::now() + duration;

        while current <= end {
            let futures = self
                .agents
                .iter_mut()
                .map(|agent| async {
                    if let Some(msg) = agent.next() {
                        let topic = self.imu_topic.clone();
                        let key = agent.id.to_string();

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

            current = Instant::now();
        }

        let total_count: u32 = self
            .agents
            .iter()
            .map(|agent| agent.imu_measurements_count)
            .sum();

        AgentRunnerResult {
            imu_measurements_count: total_count,
        }
    }
}
