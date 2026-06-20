use async_trait::async_trait;
use duration_str::deserialize_duration;
use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    ClientConfig,
};
use serde::Deserialize;
use std::{fmt, time::Duration};

use crate::model::event::Event;

use super::publisher::{Publisher, PublisherError};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum KCompression {
    None,
    Gzip,
    Snappy,
}

impl fmt::Display for KCompression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KCompression::None => write!(f, "none"),
            KCompression::Gzip => write!(f, "gzip"),
            KCompression::Snappy => write!(f, "snappy"),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum KRequiredAcks {
    None,
    One,
    All,
}

impl fmt::Display for KRequiredAcks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KRequiredAcks::None => write!(f, "0"),
            KRequiredAcks::One => write!(f, "1"),
            KRequiredAcks::All => write!(f, "all"),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct KafkaPublisherConfig {
    brokers: Vec<String>,
    compression: KCompression,
    client_id: String,
    required_acks: KRequiredAcks,
    #[serde(deserialize_with = "deserialize_duration")]
    conn_idle_timeout: Duration,
    #[serde(deserialize_with = "deserialize_duration")]
    message_timeout: Duration,
    #[serde(deserialize_with = "deserialize_duration")]
    ack_timeout: Duration,
    metadata_field_as_key: Option<String>,
}

pub struct KafkaPublisher {
    producer: FutureProducer,
    metadata_field_as_key: Option<String>,
}

impl KafkaPublisher {
    pub fn new(cfg: KafkaPublisherConfig) -> Result<Self, PublisherError> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", cfg.brokers.join(","))
            .set("client.id", cfg.client_id)
            .set("compression.type", cfg.compression.to_string())
            .set("acks", cfg.required_acks.to_string())
            .set(
                "connections.max.idle.ms",
                cfg.conn_idle_timeout.as_millis().to_string(),
            )
            .set(
                "message.timeout.ms",
                cfg.message_timeout.as_millis().to_string(),
            )
            .set(
                "request.timeout.ms",
                cfg.ack_timeout.as_millis().to_string(),
            )
            .create()
            .map_err(|error| {
                PublisherError::Generic(format!("failed to create Kafka producer: {error}"))
            })?;
        Ok(KafkaPublisher {
            producer,
            metadata_field_as_key: cfg.metadata_field_as_key,
        })
    }
}

#[async_trait]
impl Publisher<Event> for KafkaPublisher {
    async fn publish_one(&self, topic: &str, payload: Event) -> Result<(), PublisherError> {
        let default_key = &payload.id.to_string();
        let key = self
            .metadata_field_as_key
            .as_ref()
            .and_then(|k| payload.metadata.get(k))
            .unwrap_or(default_key);
        let value =
            serde_json::to_string(&payload).map_err(|e| PublisherError::Generic(e.to_string()))?;
        let result = self
            .producer
            .send(
                FutureRecord::to(topic).key(key).payload(&value),
                Duration::ZERO,
            )
            .await;
        result
            .map(|_| ())
            .map_err(|(e, _)| PublisherError::Generic(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::{KCompression, KRequiredAcks};

    #[test]
    fn maps_kafka_configuration_values() {
        assert_eq!(KCompression::None.to_string(), "none");
        assert_eq!(KCompression::Gzip.to_string(), "gzip");
        assert_eq!(KCompression::Snappy.to_string(), "snappy");
        assert_eq!(KRequiredAcks::None.to_string(), "0");
        assert_eq!(KRequiredAcks::One.to_string(), "1");
        assert_eq!(KRequiredAcks::All.to_string(), "all");
    }
}
