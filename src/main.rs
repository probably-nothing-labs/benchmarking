#![allow(dead_code)]
#![allow(unused_variables)]

mod agent;
mod imu_types;
mod producer;

use log::info;
use std::sync::Arc;

use clap::Parser;

use crate::agent::AgentRunner;
use crate::producer::{KafkaProducer, Producer};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long, short='s', default_value_t=String::from("localhost:9092"))]
    bootstrap_server: String,

    #[arg(long, short = 'd', default_value_t = 10)]
    duration_s: u32,

    #[arg(long, short = 'a', default_value_t = 10_000)]
    num_agents: u32,

    #[arg(long, short = 'g', default_value_t = 1000)]
    num_green_threads: u32,

    #[arg(long, default_value_t = 4)]
    imu_tick_rate: u32,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let cli = Cli::parse();

    info!("Connecting to Kafka cluster: {}", cli.bootstrap_server);

    let producer: Arc<dyn Producer> = Arc::new(KafkaProducer::new(cli.bootstrap_server.clone()));

    let imu_topic = String::from("driver_imu_data");
    let trips_topic = String::from("trips");
    let imu_tick_rate = std::time::Duration::new(cli.imu_tick_rate.into(), 0);

    let agents_per_runner = cli.num_agents / cli.num_green_threads;

    let mut tasks = tokio::task::JoinSet::new();

    for _ in 0..cli.num_green_threads {
        let producer = Arc::clone(&producer);
        let imu_topic = imu_topic.clone();
        tasks.spawn(async move {
            let runner = AgentRunner::new(
                agents_per_runner,
                producer,
                imu_topic,
                imu_tick_rate.clone(),
            );
            runner.run(std::time::Duration::new(cli.duration_s.into(), 0)).await
        });
    }

    let mut total_events = 0;
    while let Some(res) = tasks.join_next().await {
        let run_res = res.unwrap();
        total_events += run_res.imu_measurements_count;
    }

    info!("Done!");
    info!("Total events: {}", total_events);
}
