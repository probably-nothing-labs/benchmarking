# benchmarking

Build flink job: `cd flink_jobs ; mvn clean package`
Submit flink job to docker cluster: `BOOTSTRAP_SERVERS=kafka101:29092,kafka102:29092,kafka103:29092 ~/Desktop/flink-1.19.0/bin/flink run -m localhost:8081 target/flink_jobs-1.0-SNAPSHOT.jar`
