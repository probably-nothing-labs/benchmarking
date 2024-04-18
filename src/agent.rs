use crate::producer::Producer;
use uuid::Uuid;

use std::sync::Arc;

pub struct Agent {
    id: Uuid,
    imu_topic: String,
}

impl Agent {
    pub fn new(imu_topic: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            imu_topic,
        }
    }
}

pub struct AgentRunner {
    agents: Vec<Agent>,
    producer: Arc<dyn Producer>,
}

impl AgentRunner {
    pub fn new(num_agents: u32, imu_topic: String, producer: Arc<dyn Producer>) -> Self {
        let agents = (0..num_agents)
            .map(|_| Agent::new(imu_topic.clone()))
            .collect();
        Self { agents, producer }
    }

    pub async fn send_test_events(&self) {
        let futures = self
            .agents
            .iter()
            .map(|agent| async {
                let topic = String::from("test-topic");
                let key = String::from("test-key");
                let msg = String::from(format!("hello world! {}", agent.id)).into_bytes();

                self.producer
                    .send(topic, key, &msg)
                    .await
                    .expect("Message not sent")
            })
            .collect::<Vec<_>>();

        for future in futures {
            future.await;
        }
        log::info!("Complete!");
    }
}
