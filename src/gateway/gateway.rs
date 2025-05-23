use std::{collections::HashMap, sync::Arc};
use tokio::task;
use log::{debug, error, info};

use crate::{
    model::{
        event::{Data, Event},
        routing::{DataSchema, TopicRoutingRule, TopicValidationConfig},
        topic::Topic,
    },
    publisher::publisher::{Publisher, PublisherError},
    router::router::{TopicRouter, TopicRoutings},
    store::storage::{Storage, StorageError},
};

use jsonschema::{Draft, JSONSchema};
use serde_json::{Map, Value};
use uuid::Uuid;
use futures::TryFutureExt;

pub trait GateWay {
    async fn handle(&self, event: &Event) -> Result<(), GatewayError>;

    async fn add_routing_rule(&self, rule: &TopicRoutingRule) -> Result<(), GatewayError>;
    async fn update_routing_rule(&self, id: Uuid, rule: &TopicRoutingRule) -> Result<(), GatewayError>;
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

struct EventSamplingConfig {
    enabled: bool,
    threshold: f64,
}

impl EventSamplingConfig {
    fn new(enabled: bool, threshold: f64) -> Self {
        Self { enabled, threshold }
    }

    fn should_store_event(&self, event: &Event) -> bool {
        if !self.enabled {
            return false;
        }
        // Convert event ID to a number between 0 and 1
        let id_bytes = event.id.as_bytes();
        let hash: u32 = id_bytes.iter().fold(0, |acc, &x| acc.wrapping_add(x as u32));
        let normalized = (hash as f64) / (u32::MAX as f64);
        
        // Store if the normalized value is less than the threshold percentage
        normalized <= (self.threshold / 100.0)
    }
}

pub struct EventGateway {
    publisher: Box<dyn Publisher<Event>>,
    store: Arc<Box<dyn Storage>>,
    sampling: EventSamplingConfig,
}

impl EventGateway {
    pub fn new(
        publisher: Box<dyn Publisher<Event> + Sync + Send>,
        store: Box<dyn Storage>,
        sampling_enabled: bool,
        sampling_threshold: f64,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(EventGateway { 
            publisher, 
            store: Arc::new(store),
            sampling: EventSamplingConfig::new(sampling_enabled, sampling_threshold),
        })
    }

    pub async fn get_sample_events(&self, limit: i64, offset: i64) -> Result<(Vec<Event>, i64), GatewayError> {
        self.store.get_sample_events(limit, offset).await.map_err(GatewayError::from)
    }

    fn should_store_event(&self, event: &Event) -> bool {
        self.sampling.should_store_event(event)
    }

    fn store_event_in_background(
        &self,
        event: &Event,
        routing_id: Option<Uuid>,
        topic: Topic,
        failure_reason: Option<String>,
    ) {
        if !self.should_store_event(event) {
            return;
        }

        let store = Arc::clone(&self.store);
        let event = event.clone();
        let topic_str = topic.into_string();
        task::spawn(async move {
            if let Err(e) = store.store_event(&event, routing_id, Some(topic_str), failure_reason).await {
                error!("Failed to store event in background: {:?}", e);
            }
        });
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

impl GateWay for EventGateway {
    async fn handle(&self, event: &Event) -> Result<(), GatewayError> {
        let rules = self.store.get_all_rules().await.map_err(GatewayError::from)?;
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

                let invalid_schema = match &event.data {
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
                        schemas.iter().find_map(|&schema| {
                            if schema.schema.is_valid(&json) {
                                None
                            } else {
                                Some(schema)
                            }
                        })
                    }
                    Data::String(_) => None,
                    Data::Binary(_) => None,
                };

                // Check schema validation first, regardless of storage threshold
                if let Some(schema_details) = invalid_schema {
                    let error_msg = format!(
                        "Data of event with id {} doesnt match schema {}",
                        event.id, schema_details.name
                    );
                    
                    // Store the event with schema validation error if it passes the threshold
                    self.store_event_in_background(
                        event,
                        Some(routing.id),
                        routing.topic.clone(),
                        Some(format!("Schema validation failed: {}", schema_details.name))
                    );
                    
                    return Err(GatewayError::SchemaInvalid(error_msg));
                }

                // Try to publish the event
                let result = self.publisher.publish_one(routing.topic.as_str(), event.to_owned()).await;
                
                match result {
                    Ok(_) => {
                        self.store_event_in_background(
                            event,
                            Some(routing.id),
                            routing.topic.clone(),
                            None
                        );
                    }
                    Err(e) => {
                        self.store_event_in_background(
                            event,
                            Some(routing.id),
                            routing.topic.clone(),
                            Some(format!("Failed to publish event: {:?}", e))
                        );
                        return Err(GatewayError::InternalError(format!("Failed to publish event: {:?}", e)));
                    }
                }
                result.map_err(GatewayError::from)
            }
            None => {
                self.store_event_in_background(
                    event,
                    None,
                    Topic::new("").unwrap_or_else(|_| Topic::new("unknown").unwrap()),
                    Some("No topic to route event".to_string())
                );
                Err(GatewayError::NoTopicToRoute(format!(
                    "No topic to route event: {:?}",
                    event.id
                )))
            }
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

    async fn update_routing_rule(&self, id: Uuid, rule: &TopicRoutingRule) -> Result<(), GatewayError> {
        self.store.update_rule(id, rule).await.map_err(GatewayError::from)
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

