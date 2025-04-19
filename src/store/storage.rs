use crate::model::routing::{DataSchema, TopicRoutingRule, TopicValidationConfig};
use serde::Deserialize;
use serde_json;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::io::{self};
use uuid::Uuid;
use async_trait::async_trait;

#[derive(Debug)]
pub enum StorageError {
    NotFound,
    IoError(std::io::Error),
    SerializationError(serde_json::Error),
}

impl Error for StorageError {}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            StorageError::IoError(_) => write!(f, "connection error occurred"),
            StorageError::NotFound => write!(f, "item not found"),
            StorageError::SerializationError(_) => write!(f, "serialization error occurred"),
        }
    }
}

impl From<io::Error> for StorageError {
    fn from(err: io::Error) -> StorageError {
        StorageError::IoError(err)
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(err: serde_json::Error) -> StorageError {
        StorageError::SerializationError(err)
    }
}

#[async_trait]
pub trait Storage: Send + Sync {
    async fn add_rule(&self, rule: &TopicRoutingRule) -> Result<(), StorageError>;
    async fn get_rule(&self, id: Uuid) -> Result<Option<TopicRoutingRule>, StorageError>;
    async fn get_all_rules(&self) -> Result<Vec<TopicRoutingRule>, StorageError>;
    async fn update_rule(&self, id: Uuid, rule: &TopicRoutingRule) -> Result<(), StorageError>;
    async fn delete_rule(&self, id: Uuid) -> Result<(), StorageError>;

    async fn add_topic_validation(&self, v: &TopicValidationConfig) -> Result<(), StorageError>;
    async fn get_all_topic_validations(&self) -> Result<HashMap<String, Vec<DataSchema>>, StorageError>;

    async fn get_validations_for_topic(&self, topic: &str) -> Result<Vec<DataSchema>, StorageError> {
        let validations = self.get_all_topic_validations().await?;
        Ok(validations.get(topic).cloned().unwrap_or_default())
    }

    async fn delete_topic_validation(&self, id: &Uuid) -> Result<(), StorageError>;
}

#[derive(Deserialize)]
pub struct InMemoryStorage {
    routing_rules: Vec<TopicRoutingRule>,
    topic_validations: HashMap<String, Vec<DataSchema>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        InMemoryStorage {
            routing_rules: Vec::new(),
            topic_validations: HashMap::new(),
        }
    }

    pub fn with_initial_routing_rules(mut self, rules: Vec<TopicRoutingRule>) -> Self {
        self.routing_rules = rules;
        self
    }

    pub fn with_initial_topic_validations(
        mut self,
        validations: HashMap<String, Vec<DataSchema>>,
    ) -> Self {
        self.topic_validations = validations;
        self
    }
}

#[async_trait]
impl Storage for InMemoryStorage {
    async fn add_rule(&self, rule: &TopicRoutingRule) -> Result<(), StorageError> {
        todo!()
    }

    async fn get_rule(&self, id: Uuid) -> Result<Option<TopicRoutingRule>, StorageError> {
        Ok(self.routing_rules.iter().find(|r| r.id == id).cloned())
    }

    async fn get_all_rules(&self) -> Result<Vec<TopicRoutingRule>, StorageError> {
        Ok(self.routing_rules.clone())
    }

    async fn update_rule(&self, id: Uuid, rule: &TopicRoutingRule) -> Result<(), StorageError> {
        todo!()
    }

    async fn delete_rule(&self, id: Uuid) -> Result<(), StorageError> {
        todo!()
    }

    async fn add_topic_validation(&self, v: &TopicValidationConfig) -> Result<(), StorageError> {
        todo!()
    }

    async fn get_all_topic_validations(&self) -> Result<HashMap<String, Vec<DataSchema>>, StorageError> {
        Ok(self.topic_validations.clone())
    }

    async fn delete_topic_validation(&self, id: &Uuid) -> Result<(), StorageError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::model::expressions::{Condition, StringExpression};

    use super::*;

    #[test]
    fn test_in_memory_storage() {
        let mut storage = InMemoryStorage::new();
        let rule = TopicRoutingRule {
            id: Uuid::new_v4(),
            order: 0,
            topic: "topic".to_string(),
            description: None,
            event_version_condition: Some(Condition::ONE(StringExpression::Equals {
                value: "1.0".to_string(),
            })),
            event_type_condition: Condition::ONE(StringExpression::Equals {
                value: "event".to_string(),
            }),
        };

        // Test adding and retrieving a rule
        // storage.add_rule(rule.clone()).unwrap();
        // let retrieved = storage.get_rule(rule.id).unwrap();
        // assert_eq!(retrieved, Some(rule));
    }
}
