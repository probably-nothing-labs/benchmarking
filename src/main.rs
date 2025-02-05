#![allow(dead_code)]
#![allow(unused_variables)]

mod agent;
mod agent_runner;
mod events;
mod imu_types;
mod logging;
mod producer;

use clap::Parser;
use std::sync::Arc;

use crate::agent_runner::{AgentRunner, AgentRunnerResult, SimulationConfig};
use crate::producer::{KafkaProducer, MsgProducer};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long, short='s', default_value_t=String::from("localhost:19092,localhost:29092,localhost:39092"), help="Speifcy kafka bootstrap servers")]
    bootstrap_server: String,

    #[arg(
        long,
        short = 'd',
        default_value_t = 10,
        help = "The length (in seconds) the simulation should run for"
    )]
    duration_s: u64,

    #[arg(
        long,
        short = 'a',
        default_value_t = 10,
        help = "The number of agents to run"
    )]
    num_agents: u64,

    #[arg(
        long,
        short = 'g',
        default_value_t = 32,
        help = "The number of tokio green threads used to run the simulation"
    )]
    num_green_threads: u64,

    #[arg(
        long,
        default_value_t = 1,
        help = "The rate at which an agent will emit an IMU measurement"
    )]
    imu_tick_rate_s: u64,

    #[arg(long, default_value_t=String::from("driver-imu-data"))]
    imu_topic: String,

    #[arg(long, default_value_t=String::from("trips"))]
    trips_topic: String,

    #[arg(
        long,
        default_value_t = 10,
        help = "target msg size in bytes. This number of \"junk bytes\" will be added to each message to adjust the message size"
    )]
    target_msg_size: usize,
}

#[tokio::main]
async fn main() {
    let _guard = crate::logging::configure_logging();

    let cli = Cli::parse();
    let pid = std::process::id();

    tracing::info!(
        pid,
        bootstrap_server = cli.bootstrap_server,
        "Connecting to Kafka cluster"
    );

    let producer: Arc<dyn MsgProducer> = Arc::new(KafkaProducer::new(cli.bootstrap_server.clone()));

    let num_green_threads = if cli.num_agents < cli.num_green_threads {
        1
    } else {
        cli.num_green_threads
    };

    let agents_per_runner = cli.num_agents / num_green_threads;

    let mut tasks = tokio::task::JoinSet::new();

    let imu_topic = cli.imu_topic.clone();
    let imu_delta = std::time::Duration::from_secs(cli.imu_tick_rate_s);
    let trips_topic = cli.trips_topic.clone();
    let target_msg_size = cli.target_msg_size.clone();
    let config = SimulationConfig {
        junk_data_size: target_msg_size,

        imu_topic,
        imu_delta,

        trips_topic,
        first_trip_delay_s: 1,
        trip_delay_min_s: 1,
        trip_delay_max_s: 2,
        trip_min_length_s: 1,
        trip_max_length_s: 2,
    };

    tracing::info!(target: crate::logging::LOG_ALL, "");
    tracing::info!(
        target: crate::logging::LOG_ALL,
        pid,
        num_green_threads,
        agents_per_runner,
        config=?config,
        "starting traffic",
    );

    for _ in 0..num_green_threads {
        let config = config.clone();
        let producer = Arc::clone(&producer);

        tasks.spawn(async move {
            AgentRunner::new(agents_per_runner, producer, config.clone())
                .run(std::time::Duration::new(cli.duration_s.into(), 0))
                .await
        });
    }

    let mut results = AgentRunnerResult::default();
    while let Some(res) = tasks.join_next().await {
        // let run_res = res.unwrap();
        match res {
            Ok(run_res) => results += run_res,
            Err(err) => tracing::error!("{:?}", err),
        }
    }

    tracing::info!(target: crate::logging::LOG_ALL, pid, results=?results, "simulation complete");
}
