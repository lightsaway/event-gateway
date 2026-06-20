use crate::model::event::Event;
use async_trait::async_trait;
use log::info;
use std::error::Error;

#[derive(Debug)]
pub enum PublisherError {
    Generic(String),
}

#[async_trait]
pub trait Publisher<T>: Send + Sync {
    async fn publish_one(&self, topic: &str, payload: T) -> Result<(), PublisherError>;
}

pub struct NoOpPublisher;

#[async_trait]
impl Publisher<Event> for NoOpPublisher {
    async fn publish_one(&self, topic: &str, payload: Event) -> Result<(), PublisherError> {
        let event_json =
            serde_json::to_string(&payload).map_err(|e| PublisherError::Generic(e.to_string()))?;
        info!(
            "published to topic: {:?} and event: {:?}",
            topic, event_json
        );
        Ok(())
    }
}
