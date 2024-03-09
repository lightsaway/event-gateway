use std::{borrow::Borrow, fmt, time::Duration};
use duration_str::deserialize_duration;
use async_trait::async_trait;
 use serde::ser::StdError;
use rdkafka::producer::FutureProducer;
use rdkafka::ClientConfig;
use rdkafka::producer::FutureRecord;
use kafka::{
    client::{Compression, KafkaClient, RequiredAcks, DEFAULT_CONNECTION_IDLE_TIMEOUT_MILLIS},
    producer::Producer,
};
use futures::FutureExt;
use serde::{
    de::{self, Unexpected, Visitor},
    Deserialize, Deserializer, Serialize,
};

use kafka::producer::Record;

use crate::model::event::Event;

use super::publisher::{ Publisher, PublisherError};
pub struct SerdeCompression(pub Compression);

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KCompression {
    NONE,
    GZIP,
    SNAPPY,
}

impl fmt::Display for KCompression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KCompression::NONE => write!(f, "none"),
            KCompression::GZIP => write!(f, "gzip"),
            KCompression::SNAPPY => write!(f, "snappy"),
        }
    }
}

impl From<Box<dyn StdError>> for PublisherError {
    fn from(e: Box<dyn StdError>) -> Self {
        PublisherError::Generic(format!("{:?}", e))
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KRequiredAcks {
    NONE,
    ONE,
    ALL,
}

impl From<KRequiredAcks> for RequiredAcks {
    fn from(kra: KRequiredAcks) -> Self {
        match kra {
            KRequiredAcks::NONE => RequiredAcks::None,
            KRequiredAcks::ONE => RequiredAcks::One,
            KRequiredAcks::ALL => RequiredAcks::All,
        }
    }
}

#[derive(Debug, Deserialize)]
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
    pub fn new(cfg: KafkaPublisherConfig) -> Self {
        let producer: FutureProducer = ClientConfig::new()
                .set("bootstrap.servers",  cfg.brokers.join(",") )
                .set("client.id", cfg.client_id)
                .set("compression.type", cfg.compression.to_string())
                .set("connections.max.idle.ms", cfg.conn_idle_timeout.as_millis().to_string())
                .create()
                .expect("Producer creation error");
        KafkaPublisher {
            producer,
            metadata_field_as_key: cfg.metadata_field_as_key,
        }
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
        let value = serde_json::to_string(&payload).map_err(|e| PublisherError::Generic(e.to_string()))?;
        let result = self.producer.send(
            FutureRecord::to(&topic)
                .key(key)
                .payload(&value),
            Duration::from_secs(0),
        ).await;
        result.map(|_| ())
            .map_err(|(e, _)| PublisherError::Generic(e.to_string()))
    }
}
