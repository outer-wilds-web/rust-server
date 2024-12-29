use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize)]
struct PlanetPosition {
    type_object: String,
    name: String,
    x: f64,
    y: f64,
    z: f64,
    timestamp: u64,
}

#[derive(Clone)]
pub struct KafkaProducer {
    producer: FutureProducer,
    topic: String,
}

impl KafkaProducer {
    pub fn new(brokers: &str, topic: &str) -> Result<Self, rdkafka::error::KafkaError> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .create()?;

        Ok(Self {
            producer,
            topic: topic.to_string(),
        })
    }

    pub async fn send_planet_positions(
        &self,
        positions: Vec<(String, (f64, f64))>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        for (name, (x, y)) in positions {
            let position = PlanetPosition {
                type_object: "planet".parse().unwrap(),
                name,
                x,
                y,
                z: 0.0,
                timestamp,
            };

            let payload = serde_json::to_string(&position)?;

            self.producer
                .send(
                    FutureRecord::to(&self.topic)
                        .payload(&payload)
                        .key(&position.name),
                    Duration::from_secs(0),
                )
                .await
                .map_err(|(err, _)| err)?;
        }

        Ok(())
    }
}