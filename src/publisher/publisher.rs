use crate::model::event::Event;
use async_trait::async_trait;
use log::info;
use std::fmt;

#[derive(Debug)]
pub enum PublisherError {
    Generic(String),
}

impl fmt::Display for PublisherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PublisherError::Generic(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for PublisherError {}

#[derive(Clone, Debug, Default)]
pub struct PublishContext {
    pub group_metadata_field: Option<String>,
}

#[async_trait]
pub trait Publisher<T>: Send + Sync {
    async fn publish_one(
        &self,
        topic: &str,
        payload: T,
        context: PublishContext,
    ) -> Result<(), PublisherError>;
}

pub struct NoOpPublisher;

#[async_trait]
impl Publisher<Event> for NoOpPublisher {
    async fn publish_one(
        &self,
        topic: &str,
        payload: Event,
        _context: PublishContext,
    ) -> Result<(), PublisherError> {
        let event_json =
            serde_json::to_string(&payload).map_err(|e| PublisherError::Generic(e.to_string()))?;
        info!("published to topic: {topic:?} and event: {event_json:?}");
        Ok(())
    }
}
