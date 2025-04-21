use std::{collections::HashMap, fmt::Binary};

use crate::{
    model::{
        event::{Data, Event},
        routing::{DataSchema, TopicRoutingRule, TopicValidationConfig},
    },
    publisher::publisher::{Publisher, PublisherError},
    router::router::{TopicRouter, TopicRoutings},
    store::storage::{Storage, StorageError},
};

use axum::http::uri::Scheme;
use jsonschema::{Draft, JSONSchema};
use log::{debug, info};
use serde_json::{Map, Value};
use uuid::Uuid;
use futures::TryFutureExt;
use std::sync::Arc;

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

pub struct EventGateway {
    publisher: Box<dyn Publisher<Event>>,
    store: Box<dyn Storage>,
}

impl EventGateway {
    pub fn new(
        publisher: Box<dyn Publisher<Event> + Sync + Send>,
        store: Box<dyn Storage>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(EventGateway { publisher, store })
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
            Some(topic) => {
                let topic_schemas = self
                    .store
                    .get_validations_for_topic(topic)
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
                            json, topic
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
                if let Some(schema_details) = invalid_schema {
                    return Err(GatewayError::SchemaInvalid(format!(
                        "Data of event with id {} doesnt match schema {}",
                        event.id, schema_details.name
                    )));
                }
                let result = self.publisher.publish_one(&topic, event.to_owned()).await;
                result.map_err(GatewayError::from)
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

