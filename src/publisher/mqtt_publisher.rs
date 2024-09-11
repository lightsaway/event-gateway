use crate::model::event::Event;
use crate::publisher::publisher::{Publisher, PublisherError};
use async_trait::async_trait;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QosLevel {
    AtMostOnce,
    AtLeastOnce,
    ExactlyOnce,
}

impl From<QosLevel> for QoS {
    fn from(qos: QosLevel) -> Self {
        match qos {
            QosLevel::AtMostOnce => QoS::AtMostOnce,
            QosLevel::AtLeastOnce => QoS::AtLeastOnce,
            QosLevel::ExactlyOnce => QoS::ExactlyOnce,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct MqttPublisherConfig {
    pub host: String,
    pub port: u16,
    pub client_id: String,
    pub keep_alive: u64,
    pub clean_session: bool,
    pub qos: QosLevel,
    pub retain: bool,
}

pub struct MqttPublisher {
    client: AsyncClient,
    qos: QoS,
    retain: bool,
}

impl MqttPublisher {
    pub fn new(config: MqttPublisherConfig) -> Self {
        let mut mqttoptions = MqttOptions::new(config.client_id, config.host, config.port);
        mqttoptions.set_keep_alive(Duration::from_secs(config.keep_alive));
        mqttoptions.set_clean_session(config.clean_session);

        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

        // Spawn a task to handle connection events
        tokio::spawn(async move {
            loop {
                if let Err(e) = eventloop.poll().await {
                    eprintln!("MQTT Event Loop Error: {:?}", e);
                }
            }
        });

        MqttPublisher {
            client,
            qos: config.qos.into(),
            retain: config.retain,
        }
    }
}

#[async_trait]
impl Publisher<Event> for MqttPublisher {
    async fn publish_one(&self, topic: &str, payload: Event) -> Result<(), PublisherError> {
        let payload_json = serde_json::to_string(&payload)
            .map_err(|e| PublisherError::Generic(e.to_string()))?;

        self.client
            .publish(topic, self.qos, self.retain, payload_json)
            .await
            .map_err(|e| PublisherError::Generic(e.to_string()))?;

        Ok(())
    }
}
