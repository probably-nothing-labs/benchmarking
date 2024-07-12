use std::time::Duration;

use async_trait::async_trait;

use rdkafka::config::ClientConfig;
use rdkafka::producer::future_producer::OwnedDeliveryResult;
use rdkafka::producer::FutureProducer;
use rdkafka::producer::Producer;
use rdkafka::producer::FutureRecord;
use rdkafka::util::Timeout;

#[async_trait]
pub trait MsgProducer: Sync + Send {
    async fn send<'a>(&'a self, topic: String, key: String, msg: &Vec<u8>) -> OwnedDeliveryResult;
}

#[derive(Clone)]
pub struct KafkaProducer {
    kafka_producer: FutureProducer,
}

impl KafkaProducer {
    pub fn new(bootstrap_server: String) -> Self {
        let kafka_producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", bootstrap_server)
            .set("message.timeout.ms", "60000")
            .create()
            .expect("Producer creation error");

        Self { kafka_producer }
    }
}

#[async_trait]
impl MsgProducer for KafkaProducer {
    async fn send<'a>(&'a self, topic: String, key: String, msg: &Vec<u8>) -> OwnedDeliveryResult {
        let delivery_status = self
            .kafka_producer
            .send(
                FutureRecord::to(topic.as_str())
                    .payload(msg)
                    .key(key.as_str()),
                // Duration::from_secs(0),
                Timeout::Never,
            )
            .await;

        delivery_status
    }
}

impl Drop for KafkaProducer {
    fn drop(&mut self) {
        self.kafka_producer.flush(Duration::from_secs(10)).unwrap();
        println!("All messages have been delivered.");
    }
}
