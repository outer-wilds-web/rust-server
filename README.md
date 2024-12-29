To launch Kafka : 

```shell
docker-compose up -d
```

Then to open the bash of Kafka : 

```shell
docker exec -it kafka bash
```

In the console of Kakfa, you can create these 2 topics :
```shell
kafka-topics --create --topic ship-positions --bootstrap-server localhost:9092 --partitions 1 --replication-factor 1
kafka-topics --create --topic planet-positions --bootstrap-server localhost:9092 --partitions 1 --replication-factor 1
```

To see the messages of the topic, you can write this command (changing the topic name) : 
```shell
kafka-console-consumer --topic planet-positions     --bootstrap-server localhost:9092
```
