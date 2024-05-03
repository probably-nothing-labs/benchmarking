# benchmarking

Configure a kafka cluster with logging:
- clone https://github.com/probably-nothing-labs/kafka-monitoring-stack-docker-compose
- run `docker compose -f denormalized-benchmark-cluster.yml up`
- Connect to the kafka cluster using `localhost:19092,localhost:29092,localhost:39092`

## Flink Job
- Run the flink cluster: `docker compose -f docker-compose-flink.yaml up`
- Build flink job: `cd flink_jobs ; mvn clean package`
- Submit flink job to docker cluster: `BOOTSTRAP_SERVERS=kafka101:29092,kafka102:29092,kafka103:29092 ~/Desktop/flink-1.19.0/bin/flink run -m localhost:8081 target/flink_jobs-1.0-SNAPSHOT.jar`
