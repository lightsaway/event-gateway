use crate::model::event::Event;
use crate::publisher::publisher::{PublishContext, Publisher, PublisherError};
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
    #[serde(default)]
    pub group_metadata_field: Option<String>,
}

fn default_max_connections() -> u32 {
    DEFAULT_MAX_CONNECTIONS
}

fn select_group_metadata_field<'a>(
    routing_field: Option<&'a str>,
    global_field: Option<&'a str>,
) -> Option<&'a str> {
    routing_field.or(global_field)
}

pub struct PgmqPublisher {
    pool: PgPool,
    delay_seconds: i32,
    group_metadata_field: Option<String>,
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
            group_metadata_field: config.group_metadata_field,
        })
    }

    fn headers(event: &Event, group_metadata_field: Option<&str>) -> Result<Value, PublisherError> {
        let mut headers = json!({
            "event_id": event.id,
            "event_type": event.event_type,
            "event_version": event.event_version,
        });

        if let Some(field) = group_metadata_field {
            if field.trim().is_empty() {
                return Err(PublisherError::Generic(
                    "PGMQ group metadata field cannot be empty".to_string(),
                ));
            }

            let group = event.metadata.get(field).ok_or_else(|| {
                PublisherError::Generic(format!(
                    "event '{}' of type '{}' is missing configured PGMQ group metadata field '{}'",
                    event.id, event.event_type, field
                ))
            })?;

            if group.trim().is_empty() {
                return Err(PublisherError::Generic(format!(
                    "event '{}' of type '{}' has an empty PGMQ group metadata field '{}'",
                    event.id, event.event_type, field
                )));
            }

            headers["x-pgmq-group"] = Value::String(group.clone());
        }

        Ok(headers)
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
        if self
            .group_metadata_field
            .as_deref()
            .is_some_and(|field| field.trim().is_empty())
        {
            return Err(PublisherError::Generic(
                "PGMQ group_metadata_field cannot be empty".to_string(),
            ));
        }
        Ok(())
    }
}

#[async_trait]
impl Publisher<Event> for PgmqPublisher {
    async fn publish_one(
        &self,
        queue_name: &str,
        event: Event,
        context: PublishContext,
    ) -> Result<(), PublisherError> {
        let message = serde_json::to_value(&event)
            .map_err(|error| PublisherError::Generic(error.to_string()))?;
        let group_metadata_field = select_group_metadata_field(
            context.group_metadata_field.as_deref(),
            self.group_metadata_field.as_deref(),
        );
        let headers = Self::headers(&event, group_metadata_field)?;

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
        default_max_connections, select_group_metadata_field, PgmqPublisher, PgmqPublisherConfig,
        DEFAULT_MAX_CONNECTIONS,
    };
    use crate::model::event::{Data, Event};
    use crate::publisher::publisher::PublishContext;
    use std::collections::HashMap;
    use uuid::Uuid;

    fn event(event_type: &str, metadata: HashMap<String, String>) -> Event {
        Event {
            id: Uuid::new_v4(),
            event_type: event_type.to_string(),
            event_version: Some("1".to_string()),
            data: Data::Json(HashMap::new()),
            data_type: None,
            transport_metadata: None,
            metadata,
            origin: None,
            timestamp: None,
        }
    }

    #[test]
    fn uses_bounded_default_pool_size() {
        assert_eq!(default_max_connections(), DEFAULT_MAX_CONNECTIONS);
    }

    #[test]
    fn creates_relay_visible_headers() {
        let event = event("order.created", HashMap::new());

        let headers = PgmqPublisher::headers(&event, None).unwrap();
        assert_eq!(headers["event_id"], event.id.to_string());
        assert_eq!(headers["event_type"], "order.created");
        assert_eq!(headers["event_version"], "1");
        assert!(headers.get("x-pgmq-group").is_none());
    }

    #[test]
    fn adds_group_header_from_global_metadata_field() {
        let event = event(
            "order.created",
            HashMap::from([("aggregate_id".to_string(), "order-42".to_string())]),
        );

        let headers = PgmqPublisher::headers(&event, Some("aggregate_id")).unwrap();

        assert_eq!(headers["x-pgmq-group"], "order-42");
    }

    #[test]
    fn routing_group_field_can_override_global_field() {
        let event = event(
            "order.created",
            HashMap::from([
                ("aggregate_id".to_string(), "global-42".to_string()),
                ("order_id".to_string(), "order-42".to_string()),
            ]),
        );
        let context = PublishContext {
            group_metadata_field: Some("order_id".to_string()),
        };
        let global = Some("aggregate_id");
        let selected_field =
            select_group_metadata_field(context.group_metadata_field.as_deref(), global);
        let headers = PgmqPublisher::headers(&event, selected_field).unwrap();

        assert_eq!(headers["x-pgmq-group"], "order-42");
    }

    #[test]
    fn routing_group_field_falls_back_to_global_field() {
        let event = event(
            "order.created",
            HashMap::from([("aggregate_id".to_string(), "order-42".to_string())]),
        );
        let context = PublishContext::default();
        let global = Some("aggregate_id");
        let selected_field =
            select_group_metadata_field(context.group_metadata_field.as_deref(), global);

        let headers = PgmqPublisher::headers(&event, selected_field).unwrap();

        assert_eq!(headers["x-pgmq-group"], "order-42");
    }

    #[test]
    fn rejects_event_missing_configured_group_field() {
        let event = event("order.created", HashMap::new());

        let error = PgmqPublisher::headers(&event, Some("aggregate_id"))
            .unwrap_err()
            .to_string();

        assert!(error.contains("missing configured PGMQ group metadata field 'aggregate_id'"));
    }

    #[test]
    fn rejects_invalid_configuration_without_connecting() {
        let config = PgmqPublisherConfig {
            connection_url: String::new(),
            max_connections: 0,
            delay_seconds: -1,
            group_metadata_field: None,
        };

        assert_eq!(
            config.validate().unwrap_err().to_string(),
            "PGMQ connection_url cannot be empty"
        );
    }
}
