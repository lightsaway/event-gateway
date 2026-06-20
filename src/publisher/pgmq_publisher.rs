use crate::model::event::Event;
use crate::publisher::publisher::{Publisher, PublisherError};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::postgres::{PgPool, PgPoolOptions};

const DEFAULT_MAX_CONNECTIONS: u32 = 10;

#[derive(Debug, Deserialize, Clone)]
pub struct PgmqPublisherConfig {
    pub connection_url: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    #[serde(default)]
    pub delay_seconds: i32,
}

fn default_max_connections() -> u32 {
    DEFAULT_MAX_CONNECTIONS
}

pub struct PgmqPublisher {
    pool: PgPool,
    delay_seconds: i32,
}

impl PgmqPublisher {
    pub async fn new(config: PgmqPublisherConfig) -> Result<Self, PublisherError> {
        config.validate()?;

        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .connect(&config.connection_url)
            .await
            .map_err(|error| {
                PublisherError::Generic(format!("failed to connect to PGMQ: {error}"))
            })?;

        Ok(Self {
            pool,
            delay_seconds: config.delay_seconds,
        })
    }

    fn headers(event: &Event) -> Value {
        json!({
            "event_id": event.id,
            "event_type": event.event_type,
            "event_version": event.event_version,
        })
    }
}

impl PgmqPublisherConfig {
    fn validate(&self) -> Result<(), PublisherError> {
        if self.connection_url.trim().is_empty() {
            return Err(PublisherError::Generic(
                "PGMQ connection_url cannot be empty".to_string(),
            ));
        }
        if self.max_connections == 0 {
            return Err(PublisherError::Generic(
                "PGMQ max_connections must be greater than zero".to_string(),
            ));
        }
        if self.delay_seconds < 0 {
            return Err(PublisherError::Generic(
                "PGMQ delay_seconds cannot be negative".to_string(),
            ));
        }
        Ok(())
    }
}

#[async_trait]
impl Publisher<Event> for PgmqPublisher {
    async fn publish_one(&self, queue_name: &str, event: Event) -> Result<(), PublisherError> {
        let message = serde_json::to_value(&event)
            .map_err(|error| PublisherError::Generic(error.to_string()))?;
        let headers = Self::headers(&event);

        sqlx::query_scalar::<_, i64>(
            "SELECT pgmq.send($1::text, $2::jsonb, $3::jsonb, $4::integer)",
        )
        .bind(queue_name)
        .bind(message)
        .bind(headers)
        .bind(self.delay_seconds)
        .fetch_one(&self.pool)
        .await
        .map(|_| ())
        .map_err(|error| {
            PublisherError::Generic(format!(
                "failed to enqueue message in PGMQ queue '{queue_name}': {error}"
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        default_max_connections, PgmqPublisher, PgmqPublisherConfig, DEFAULT_MAX_CONNECTIONS,
    };
    use crate::model::event::{Data, Event};
    use std::collections::HashMap;
    use uuid::Uuid;

    #[test]
    fn uses_bounded_default_pool_size() {
        assert_eq!(default_max_connections(), DEFAULT_MAX_CONNECTIONS);
    }

    #[test]
    fn creates_relay_visible_headers() {
        let event = Event {
            id: Uuid::new_v4(),
            event_type: "order.created".to_string(),
            event_version: Some("1".to_string()),
            data: Data::Json(HashMap::new()),
            data_type: None,
            transport_metadata: None,
            metadata: HashMap::new(),
            origin: None,
            timestamp: None,
        };

        let headers = PgmqPublisher::headers(&event);
        assert_eq!(headers["event_id"], event.id.to_string());
        assert_eq!(headers["event_type"], "order.created");
        assert_eq!(headers["event_version"], "1");
    }

    #[test]
    fn rejects_invalid_configuration_without_connecting() {
        let config = PgmqPublisherConfig {
            connection_url: String::new(),
            max_connections: 0,
            delay_seconds: -1,
        };

        assert_eq!(
            config.validate().unwrap_err().to_string(),
            "PGMQ connection_url cannot be empty"
        );
    }
}
