use crate::model::routing::{DataSchema, TopicRoutingRule, TopicValidationConfig};
use serde::Deserialize;
use serde_json;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::io::{self};
use uuid::Uuid;

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

pub trait Storage: Send + Sync {
    fn add_rule(&self, rule: &TopicRoutingRule) -> Result<(), StorageError>;
    fn get_rule(&self, id: Uuid) -> Result<Option<TopicRoutingRule>, StorageError>;
    fn get_all_rules(&self) -> Result<Vec<TopicRoutingRule>, StorageError>;
    fn update_rule(&self, id: Uuid, rule: &TopicRoutingRule) -> Result<(), StorageError>;
    fn delete_rule(&self, id: Uuid) -> Result<(), StorageError>;

    fn add_topic_validation(&self, v: &TopicValidationConfig) -> Result<(), StorageError>;
    fn get_all_topic_validations(&self) -> Result<&HashMap<String, Vec<DataSchema>>, StorageError>;

    fn get_validations_for_topic(&self, topic: &str) -> Result<&Vec<DataSchema>, StorageError> {
        let validations = self.get_all_topic_validations()?;
        // Use a static empty vector
        static EMPTY: Vec<DataSchema> = Vec::new();
        Ok(validations.get(topic).unwrap_or(&EMPTY))
    }

    fn delete_topic_validation(&self, id: &Uuid) -> Result<(), StorageError>;
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

impl Storage for InMemoryStorage {
    fn add_rule(&self, rule: &TopicRoutingRule) -> Result<(), StorageError> {
        todo!()
    }

    fn get_rule(&self, id: Uuid) -> Result<Option<TopicRoutingRule>, StorageError> {
        Ok(self.routing_rules.iter().find(|r| r.id == id).cloned())
    }

    fn get_all_rules(&self) -> Result<Vec<TopicRoutingRule>, StorageError> {
        Ok(self.routing_rules.clone())
    }

    fn update_rule(&self, id: Uuid, rule: &TopicRoutingRule) -> Result<(), StorageError> {
        // match self.routing_rules.get_mut(&id) {
        //     Some(existing_rule) => {
        //         *existing_rule = rule;
        //         Ok(())
        //     }
        //     None => Err(StorageError::NotFound),
        // }
        todo!()
    }

    fn delete_rule(&self, id: Uuid) -> Result<(), StorageError> {
        todo!()
        // if self.routing_rules.remove(&id).is_some() {
        //     Ok(())
        // } else {
        //     Err(StorageError::NotFound)
        // }
    }

    fn add_topic_validation(&self, v: &TopicValidationConfig) -> Result<(), StorageError> {
        todo!()
    }

    fn get_all_topic_validations(&self) -> Result<&HashMap<String, Vec<DataSchema>>, StorageError> {
        Ok(&self.topic_validations)
    }

    fn delete_topic_validation(&self, id: &Uuid) -> Result<(), StorageError> {
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
