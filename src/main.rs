#![allow(dead_code)]
#![allow(unused_variables)]

mod agent;
mod producer;

use log::info;
use std::time::Duration;

use clap::Parser;

use rdkafka::config::ClientConfig;
use rdkafka::message::{Header, OwnedHeaders};
use rdkafka::producer::{FutureProducer, FutureRecord};

use crate::producer::{KafkaProducer, Producer};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long, short='s', default_value_t=String::from("localhost:9092"))]
    bootstrap_server: String,
}

async fn produce(brokers: &str, topic_name: &str) {
    let producer: &FutureProducer = &ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation error");

    // This loop is non blocking: all messages will be sent one after the other, without waiting
    // for the results.
    let futures = (0..5)
        .map(|i| async move {
            // The send operation on the topic returns a future, which will be
            // completed once the result or failure from Kafka is received.
            let delivery_status = producer
                .send(
                    FutureRecord::to(topic_name)
                        .payload(&format!("Message {}", i))
                        .key(&format!("Key {}", i))
                        .headers(OwnedHeaders::new().insert(Header {
                            key: "header_key",
                            value: Some("header_value"),
                        })),
                    Duration::from_secs(0),
                )
                .await;

            // This will be executed when the result is received.
            info!("Delivery status for message {} received", i);
            delivery_status
        })
        .collect::<Vec<_>>();

    // This loop will wait until all delivery statuses have been received.
    for future in futures {
        info!("Future completed. Result: {:?}", future.await);
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    println!("{}", cli.bootstrap_server);

    let mut producer = KafkaProducer::new(cli.bootstrap_server);
    let topic = String::from("test-topic");
    let key = String::from("test-key");
    let msg = String::from("hello world2!").into_bytes();

    producer
        .send(topic, key, &msg)
        .await
        .expect("message not sent");

    println!("success?");
}

// fake IMU data generator
// pass kafka futureproducer as Arc reference to greenthreads
// trip start/stop time
//
