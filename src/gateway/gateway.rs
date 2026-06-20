use async_trait::async_trait;
use log::debug;
use std::{collections::HashMap, sync::Arc};

use crate::{
    model::{
        event::{Data, Event},
        routing::{DataSchema, TopicRoutingRule, TopicValidationConfig},
    },
    publisher::publisher::{Publisher, PublisherError},
    router::router::{TopicRouter, TopicRoutings},
    store::storage::{Storage, StorageError},
};

use futures::TryFutureExt;
use jsonschema::{Draft, JSONSchema};
use serde_json::{Map, Value};
use uuid::Uuid;

#[async_trait]
pub trait GateWay: Send + Sync {
    async fn handle(&self, event: &Event) -> Result<(), GatewayError>;

    async fn add_routing_rule(&self, rule: &TopicRoutingRule) -> Result<(), GatewayError>;
    async fn update_routing_rule(
        &self,
        id: Uuid,
        rule: &TopicRoutingRule,
    ) -> Result<(), GatewayError>;
    async fn get_routing_rules(&self) -> Result<Vec<TopicRoutingRule>, GatewayError>;
    async fn delete_routing_rule(&self, id: &Uuid) -> Result<(), GatewayError>;

    async fn add_topic_validation(&self, v: &TopicValidationConfig) -> Result<(), GatewayError>;
    async fn get_topic_validations(
        &self,
    ) -> Result<HashMap<String, Vec<TopicValidationConfig>>, GatewayError>;
    async fn delete_topic_validation(&self, id: &Uuid) -> Result<(), GatewayError>;
}

#[derive(Debug)]
pub enum GatewayError {
    SchemaInvalid(String),
    NoTopicToRoute(String),
    InternalError(String),
}

impl std::fmt::Display for GatewayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GatewayError::SchemaInvalid(msg) => write!(f, "Schema validation failed: {}", msg),
            GatewayError::NoTopicToRoute(msg) => write!(f, "No topic to route: {}", msg),
            GatewayError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

pub struct EventGateway {
    publisher: Box<dyn Publisher<Event>>,
    store: Arc<Box<dyn Storage>>,
}

impl EventGateway {
    pub fn new(
        publisher: Box<dyn Publisher<Event> + Sync + Send>,
        store: Box<dyn Storage>,
    ) -> Self {
        EventGateway {
            publisher,
            store: Arc::new(store),
        }
    }
}

impl From<StorageError> for GatewayError {
    fn from(e: StorageError) -> Self {
        GatewayError::InternalError(format!("{:?}", e)) // Convert store error to an internal error
    }
}

impl From<Box<dyn std::error::Error>> for GatewayError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        GatewayError::InternalError(format!("{:?}", e)) // Convert store error to an internal error
    }
}

impl From<PublisherError> for GatewayError {
    fn from(e: PublisherError) -> Self {
        GatewayError::InternalError(format!("{:?}", e)) // Convert store error to an internal error
    }
}

#[async_trait]
impl GateWay for EventGateway {
    async fn handle(&self, event: &Event) -> Result<(), GatewayError> {
        let rules = self
            .store
            .get_all_rules()
            .await
            .map_err(GatewayError::from)?;
        let routings = TopicRoutings { rules };

        match routings.route(&event) {
            Some(routing) => {
                let topic_schemas = self
                    .store
                    .get_validations_for_topic(routing.topic.as_str())
                    .await
                    .map_err(GatewayError::from)?;
                let schemas: Vec<&DataSchema> = topic_schemas
                    .iter()
                    .filter(|&v| {
                        v.event_type == event.event_type && v.event_version == event.event_version
                    })
                    .collect();

                let validation_errors = match &event.data {
                    Data::Json(j) => {
                        let json = Value::Object(
                            j.into_iter()
                                .map(|(k, v)| (k.clone(), v.clone()))
                                .collect::<Map<_, _>>(),
                        );
                        debug!(
                            "Validating schema for event data: {} [topic={}]",
                            json, routing.topic
                        );

                        // Collect validation errors from all schemas
                        let mut schema_errors = Vec::new();
                        for schema in &schemas {
                            if let Err(errors) = schema.schema.validate(&json) {
                                let error_details = errors
                                    .iter()
                                    .map(|e| {
                                        format!(
                                            "Field '{}': {} (at schema path: {})",
                                            e.instance_path, e.message, e.schema_path
                                        )
                                    })
                                    .collect::<Vec<_>>()
                                    .join("; ");
                                schema_errors.push((schema, error_details));
                            }
                        }
                        schema_errors
                    }
                    Data::String(_) => Vec::new(),
                    Data::Binary(_) => Vec::new(),
                };

                if !validation_errors.is_empty() {
                    let (failed_schema, error_details) = &validation_errors[0];
                    let error_msg = format!(
                        "Event {} failed schema validation for '{}': {}",
                        event.id, failed_schema.name, error_details
                    );

                    return Err(GatewayError::SchemaInvalid(error_msg));
                }

                self.publisher
                    .publish_one(routing.topic.as_str(), event.to_owned())
                    .await
                    .map_err(GatewayError::from)
            }
            None => Err(GatewayError::NoTopicToRoute(format!(
                "No topic to route event: {:?}",
                event.id
            ))),
        }
    }

    async fn add_topic_validation(&self, v: &TopicValidationConfig) -> Result<(), GatewayError> {
        self.store
            .add_topic_validation(v)
            .await
            .map_err(GatewayError::from)
    }

    async fn delete_topic_validation(&self, id: &Uuid) -> Result<(), GatewayError> {
        self.store
            .delete_topic_validation(id)
            .await
            .map_err(GatewayError::from)
    }

    async fn add_routing_rule(&self, rule: &TopicRoutingRule) -> Result<(), GatewayError> {
        self.store.add_rule(rule).await.map_err(GatewayError::from)
    }

    async fn update_routing_rule(
        &self,
        id: Uuid,
        rule: &TopicRoutingRule,
    ) -> Result<(), GatewayError> {
        self.store
            .update_rule(id, rule)
            .await
            .map_err(GatewayError::from)
    }

    async fn get_routing_rules(&self) -> Result<Vec<TopicRoutingRule>, GatewayError> {
        self.store.get_all_rules().await.map_err(GatewayError::from)
    }

    async fn delete_routing_rule(&self, id: &Uuid) -> Result<(), GatewayError> {
        self.store
            .delete_rule(id.to_owned())
            .await
            .map_err(GatewayError::from)
    }

    async fn get_topic_validations(
        &self,
    ) -> Result<HashMap<String, Vec<TopicValidationConfig>>, GatewayError> {
        self.store
            .get_all_topic_validations()
            .await
            .map_err(GatewayError::from)
    }
}
