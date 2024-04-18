#![allow(dead_code)]
#![allow(unused_variables)]

mod agent;
mod producer;

use log::info;
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;

use crate::agent::AgentRunner;
use crate::producer::{KafkaProducer, Producer};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long, short='s', default_value_t=String::from("localhost:9092"))]
    bootstrap_server: String,

    #[arg(long, short = 'a', default_value_t = 10_000)]
    num_agents: u32,

    #[arg(long, short = 'g', default_value_t = 1000)]
    num_green_threads: u32,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let cli = Cli::parse();

    info!("Connecting to Kafka cluster: {}", cli.bootstrap_server);

    let producer: Arc<dyn Producer> = Arc::new(KafkaProducer::new(cli.bootstrap_server.clone()));

    let imu_topic = String::from("driver_imu_data");
    let trips_topic = String::from("trips");

    let agents_per_runner = cli.num_agents / cli.num_green_threads;

    let mut tasks = tokio::task::JoinSet::new();

    for _ in 0..cli.num_green_threads {
        let producer = Arc::clone(&producer);
        let imu_topic = imu_topic.clone();
        tasks.spawn(async move {
            let runner = AgentRunner::new(agents_per_runner, imu_topic, producer);
            runner.send_test_events().await;
        });
    }

    while let Some(res) = tasks.join_next().await {
        // let idx = res.unwrap();
    }

    info!("Done!")

    // let topic = String::from("test-topic");
    // let key = String::from("test-key");
    // let msg = String::from("hello world2!").into_bytes();
    //
    // producer
    //     .send(topic, key, &msg)
    //     .await
    //     .expect("message not sent");
    //
    // info!("success");
}

// fake IMU data generator
// pass kafka futureproducer as Arc reference to greenthreads
// trip start/stop time
// async fn produce(brokers: &str, topic_name: &str) {
//     let producer: &FutureProducer = &ClientConfig::new()
//         .set("bootstrap.servers", brokers)
//         .set("message.timeout.ms", "5000")
//         .create()
//         .expect("Producer creation error");
//
//     // This loop is non blocking: all messages will be sent one after the other, without waiting
//     // for the results.
//     let futures = (0..5)
//         .map(|i| async move {
//             // The send operation on the topic returns a future, which will be
//             // completed once the result or failure from Kafka is received.
//             let delivery_status = producer
//                 .send(
//                     FutureRecord::to(topic_name)
//                         .payload(&format!("Message {}", i))
//                         .key(&format!("Key {}", i))
//                         .headers(OwnedHeaders::new().insert(Header {
//                             key: "header_key",
//                             value: Some("header_value"),
//                         })),
//                     Duration::from_secs(0),
//                 )
//                 .await;
//
//             // This will be executed when the result is received.
//             info!("Delivery status for message {} received", i);
//             delivery_status
//         })
//         .collect::<Vec<_>>();
//
//     // This loop will wait until all delivery statuses have been received.
//     for future in futures {
//         info!("Future completed. Result: {:?}", future.await);
//     }
// }
