use crate::model::routing::{DataSchema, TopicRoutingRule, TopicValidationConfig};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::io;
use std::sync::RwLock;
use uuid::Uuid;

#[derive(Debug)]
pub enum StorageError {
    NotFound,
    IoError(std::io::Error),
    SerializationError(serde_json::Error),
    DatabaseError(tokio_postgres::Error),
    PoolError(deadpool_postgres::PoolError),
    Other(String),
}

impl Error for StorageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            StorageError::IoError(error) => Some(error),
            StorageError::SerializationError(error) => Some(error),
            StorageError::DatabaseError(error) => Some(error),
            StorageError::PoolError(error) => Some(error),
            StorageError::NotFound | StorageError::Other(_) => None,
        }
    }
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StorageError::IoError(error) => write!(f, "I/O error: {error}"),
            StorageError::NotFound => write!(f, "item not found"),
            StorageError::SerializationError(error) => {
                write!(f, "serialization error: {error}")
            }
            StorageError::DatabaseError(error) => write!(f, "database error: {error}"),
            StorageError::PoolError(error) => write!(f, "connection pool error: {error}"),
            StorageError::Other(msg) => write!(f, "other error: {msg}"),
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

impl From<tokio_postgres::Error> for StorageError {
    fn from(err: tokio_postgres::Error) -> StorageError {
        StorageError::DatabaseError(err)
    }
}

impl From<deadpool_postgres::PoolError> for StorageError {
    fn from(err: deadpool_postgres::PoolError) -> StorageError {
        StorageError::PoolError(err)
    }
}

#[async_trait]
pub trait Storage: Send + Sync {
    async fn add_rule(&self, rule: &TopicRoutingRule) -> Result<(), StorageError>;
    async fn get_all_rules(&self) -> Result<Vec<TopicRoutingRule>, StorageError>;
    async fn update_rule(&self, id: Uuid, rule: &TopicRoutingRule) -> Result<(), StorageError>;
    async fn delete_rule(&self, id: Uuid) -> Result<(), StorageError>;

    async fn add_topic_validation(&self, v: &TopicValidationConfig) -> Result<(), StorageError>;
    async fn get_all_topic_validations(
        &self,
    ) -> Result<HashMap<String, Vec<TopicValidationConfig>>, StorageError>;

    async fn get_validations_for_topic(
        &self,
        topic: &str,
    ) -> Result<Vec<DataSchema>, StorageError> {
        let validations = self.get_all_topic_validations().await?;
        Ok(validations
            .get(topic)
            .map(|configs| configs.iter().map(|c| c.schema.clone()).collect())
            .unwrap_or_default())
    }

    async fn delete_topic_validation(&self, id: &Uuid) -> Result<(), StorageError>;
}

#[derive(Deserialize)]
pub struct InMemoryStorage {
    routing_rules: RwLock<Vec<TopicRoutingRule>>,
    topic_validations: RwLock<HashMap<String, Vec<TopicValidationConfig>>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        InMemoryStorage {
            routing_rules: RwLock::new(Vec::new()),
            topic_validations: RwLock::new(HashMap::new()),
        }
    }

    fn lock_error(name: &str) -> StorageError {
        StorageError::Other(format!("in-memory {name} lock poisoned"))
    }
}

#[async_trait]
impl Storage for InMemoryStorage {
    async fn add_rule(&self, rule: &TopicRoutingRule) -> Result<(), StorageError> {
        let mut rules = self
            .routing_rules
            .write()
            .map_err(|_| Self::lock_error("routing rules"))?;
        if rules.iter().any(|existing| existing.id == rule.id) {
            return Err(StorageError::Other(format!(
                "routing rule {} already exists",
                rule.id
            )));
        }
        rules.push(rule.clone());
        rules.sort_by_key(|item| (item.order, item.id));
        Ok(())
    }

    async fn get_all_rules(&self) -> Result<Vec<TopicRoutingRule>, StorageError> {
        let rules = self
            .routing_rules
            .read()
            .map_err(|_| Self::lock_error("routing rules"))?;
        Ok(rules.clone())
    }

    async fn update_rule(&self, id: Uuid, rule: &TopicRoutingRule) -> Result<(), StorageError> {
        let mut rules = self
            .routing_rules
            .write()
            .map_err(|_| Self::lock_error("routing rules"))?;
        let existing = rules
            .iter_mut()
            .find(|existing| existing.id == id)
            .ok_or(StorageError::NotFound)?;
        let mut updated = rule.clone();
        updated.id = id;
        *existing = updated;
        rules.sort_by_key(|item| (item.order, item.id));
        Ok(())
    }

    async fn delete_rule(&self, id: Uuid) -> Result<(), StorageError> {
        let mut rules = self
            .routing_rules
            .write()
            .map_err(|_| Self::lock_error("routing rules"))?;
        let original_len = rules.len();
        rules.retain(|rule| rule.id != id);
        (rules.len() != original_len)
            .then_some(())
            .ok_or(StorageError::NotFound)
    }

    async fn add_topic_validation(&self, v: &TopicValidationConfig) -> Result<(), StorageError> {
        let mut validations = self
            .topic_validations
            .write()
            .map_err(|_| Self::lock_error("topic validations"))?;
        if validations
            .values()
            .flatten()
            .any(|existing| existing.id == v.id)
        {
            return Err(StorageError::Other(format!(
                "topic validation {} already exists",
                v.id
            )));
        }
        validations
            .entry(v.topic.as_str().to_string())
            .or_default()
            .push(v.clone());
        Ok(())
    }

    async fn get_all_topic_validations(
        &self,
    ) -> Result<HashMap<String, Vec<TopicValidationConfig>>, StorageError> {
        let validations = self
            .topic_validations
            .read()
            .map_err(|_| Self::lock_error("topic validations"))?;
        Ok(validations.clone())
    }

    async fn delete_topic_validation(&self, id: &Uuid) -> Result<(), StorageError> {
        let mut validations = self
            .topic_validations
            .write()
            .map_err(|_| Self::lock_error("topic validations"))?;
        let mut removed = false;
        validations.retain(|_, topic_validations| {
            let original_len = topic_validations.len();
            topic_validations.retain(|validation| validation.id != *id);
            removed |= topic_validations.len() != original_len;
            !topic_validations.is_empty()
        });
        removed.then_some(()).ok_or(StorageError::NotFound)
    }
}

#[cfg(test)]
mod tests {
    use crate::model::expressions::{Condition, StringExpression};
    use crate::model::topic::Topic;

    use super::*;

    fn rule(order: i32) -> TopicRoutingRule {
        TopicRoutingRule {
            id: Uuid::new_v4(),
            order,
            topic: Topic::new("topic").unwrap(),
            description: None,
            group_metadata_field: None,
            event_version_condition: Some(Condition::ONE(StringExpression::Equals {
                value: "1.0".to_string(),
            })),
            event_type_condition: Condition::ONE(StringExpression::Equals {
                value: "event".to_string(),
            }),
        }
    }

    #[tokio::test]
    async fn stores_updates_and_deletes_routing_rules() {
        let storage = InMemoryStorage::new();
        let first = rule(2);
        let second = rule(1);

        storage.add_rule(&first).await.unwrap();
        storage.add_rule(&second).await.unwrap();
        assert_eq!(
            storage
                .get_all_rules()
                .await
                .unwrap()
                .iter()
                .map(|item| item.id)
                .collect::<Vec<_>>(),
            vec![second.id, first.id]
        );

        let mut updated = first.clone();
        updated.order = 0;
        storage.update_rule(first.id, &updated).await.unwrap();
        assert_eq!(
            storage
                .get_all_rules()
                .await
                .unwrap()
                .into_iter()
                .find(|item| item.id == first.id),
            Some(updated)
        );

        storage.delete_rule(first.id).await.unwrap();
        assert!(storage
            .get_all_rules()
            .await
            .unwrap()
            .iter()
            .all(|item| item.id != first.id));
        assert!(matches!(
            storage.delete_rule(first.id).await,
            Err(StorageError::NotFound)
        ));
    }
}
