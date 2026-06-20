use crate::model::event::Event;
use crate::model::routing::{DataSchema, TopicRoutingRule, TopicValidationConfig};
use crate::model::topic::Topic;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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

impl Error for StorageError {}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            StorageError::IoError(_) => write!(f, "connection error occurred"),
            StorageError::NotFound => write!(f, "item not found"),
            StorageError::SerializationError(_) => write!(f, "serialization error occurred"),
            StorageError::DatabaseError(_) => write!(f, "database error occurred"),
            StorageError::PoolError(_) => write!(f, "connection pool error occurred"),
            StorageError::Other(ref msg) => write!(f, "other error: {}", msg),
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredEvent {
    pub id: Uuid,
    pub event_id: Uuid,
    pub event_type: String,
    pub event_version: Option<String>,
    pub routing_id: Option<Uuid>,
    pub destination_topic: Option<String>,
    pub failure_reason: Option<String>,
    pub stored_at: DateTime<Utc>,
    pub event_data: serde_json::Value,
}

#[async_trait]
pub trait Storage: Send + Sync {
    async fn add_rule(&self, rule: &TopicRoutingRule) -> Result<(), StorageError>;
    async fn get_rule(&self, id: Uuid) -> Result<Option<TopicRoutingRule>, StorageError>;
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

    async fn store_event(
        &self,
        event: &Event,
        routing_id: Option<Uuid>,
        destination_topic: Option<String>,
        failure_reason: Option<String>,
    ) -> Result<(), StorageError>;
    async fn get_event(&self, id: Uuid) -> Result<Option<StoredEvent>, StorageError>;
    async fn get_events_by_type(
        &self,
        event_type: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<StoredEvent>, StorageError>;
    async fn get_events_by_routing(
        &self,
        routing_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<StoredEvent>, StorageError>;
    async fn get_sample_events(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Event>, i64), StorageError>;
}

#[derive(Deserialize)]
pub struct InMemoryStorage {
    routing_rules: RwLock<Vec<TopicRoutingRule>>,
    topic_validations: RwLock<HashMap<String, Vec<TopicValidationConfig>>>,
    #[serde(default)]
    events: RwLock<Vec<StoredEvent>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        InMemoryStorage {
            routing_rules: RwLock::new(Vec::new()),
            topic_validations: RwLock::new(HashMap::new()),
            events: RwLock::new(Vec::new()),
        }
    }

    pub fn with_initial_routing_rules(self, rules: Vec<TopicRoutingRule>) -> Self {
        self.routing_rules
            .write()
            .expect("in-memory routing rules lock poisoned")
            .extend(rules);
        self
    }

    pub fn with_initial_topic_validations(
        self,
        validations: HashMap<String, Vec<TopicValidationConfig>>,
    ) -> Self {
        self.topic_validations
            .write()
            .expect("in-memory topic validations lock poisoned")
            .extend(validations);
        self
    }

    fn lock_error(name: &str) -> StorageError {
        StorageError::Other(format!("in-memory {name} lock poisoned"))
    }

    fn page<T: Clone>(items: &[T], limit: i64, offset: i64) -> Vec<T> {
        let offset = offset.max(0) as usize;
        let limit = limit.max(0) as usize;
        items.iter().skip(offset).take(limit).cloned().collect()
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

    async fn get_rule(&self, id: Uuid) -> Result<Option<TopicRoutingRule>, StorageError> {
        let rules = self
            .routing_rules
            .read()
            .map_err(|_| Self::lock_error("routing rules"))?;
        Ok(rules.iter().find(|rule| rule.id == id).cloned())
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

    async fn store_event(
        &self,
        event: &Event,
        routing_id: Option<Uuid>,
        destination_topic: Option<String>,
        failure_reason: Option<String>,
    ) -> Result<(), StorageError> {
        let stored_event = StoredEvent {
            id: Uuid::new_v4(),
            event_id: event.id,
            event_type: event.event_type.clone(),
            event_version: event.event_version.clone(),
            routing_id,
            destination_topic,
            failure_reason,
            stored_at: Utc::now(),
            event_data: serde_json::to_value(event)?,
        };
        self.events
            .write()
            .map_err(|_| Self::lock_error("events"))?
            .push(stored_event);
        Ok(())
    }

    async fn get_event(&self, id: Uuid) -> Result<Option<StoredEvent>, StorageError> {
        let events = self.events.read().map_err(|_| Self::lock_error("events"))?;
        Ok(events.iter().find(|event| event.id == id).cloned())
    }

    async fn get_events_by_type(
        &self,
        event_type: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<StoredEvent>, StorageError> {
        let events = self.events.read().map_err(|_| Self::lock_error("events"))?;
        let mut matching: Vec<_> = events
            .iter()
            .filter(|event| event.event_type == event_type)
            .cloned()
            .collect();
        matching.sort_by_key(|event| std::cmp::Reverse(event.stored_at));
        Ok(Self::page(&matching, limit, offset))
    }

    async fn get_events_by_routing(
        &self,
        routing_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<StoredEvent>, StorageError> {
        let events = self.events.read().map_err(|_| Self::lock_error("events"))?;
        let mut matching: Vec<_> = events
            .iter()
            .filter(|event| event.routing_id == Some(routing_id))
            .cloned()
            .collect();
        matching.sort_by_key(|event| std::cmp::Reverse(event.stored_at));
        Ok(Self::page(&matching, limit, offset))
    }

    async fn get_sample_events(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Event>, i64), StorageError> {
        let events = self.events.read().map_err(|_| Self::lock_error("events"))?;
        let mut sorted = events.clone();
        sorted.sort_by_key(|event| std::cmp::Reverse(event.stored_at));
        let total = sorted.len() as i64;
        let events = Self::page(&sorted, limit, offset)
            .into_iter()
            .map(|stored| serde_json::from_value(stored.event_data))
            .collect::<Result<Vec<_>, _>>()?;
        Ok((events, total))
    }
}

#[cfg(test)]
mod tests {
    use crate::model::event::Data;
    use crate::model::expressions::{Condition, StringExpression};

    use super::*;

    fn rule(order: i32) -> TopicRoutingRule {
        TopicRoutingRule {
            id: Uuid::new_v4(),
            order,
            topic: Topic::new("topic").unwrap(),
            description: None,
            event_version_condition: Some(Condition::ONE(StringExpression::Equals {
                value: "1.0".to_string(),
            })),
            event_type_condition: Condition::ONE(StringExpression::Equals {
                value: "event".to_string(),
            }),
        }
    }

    fn event(event_type: &str) -> Event {
        Event {
            id: Uuid::new_v4(),
            event_type: event_type.to_string(),
            event_version: Some("1.0".to_string()),
            metadata: HashMap::new(),
            transport_metadata: None,
            data_type: None,
            data: Data::String("payload".to_string()),
            timestamp: None,
            origin: Some("test".to_string()),
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
        assert_eq!(storage.get_rule(first.id).await.unwrap(), Some(updated));

        storage.delete_rule(first.id).await.unwrap();
        assert_eq!(storage.get_rule(first.id).await.unwrap(), None);
        assert!(matches!(
            storage.delete_rule(first.id).await,
            Err(StorageError::NotFound)
        ));
    }

    #[tokio::test]
    async fn stores_and_queries_events() {
        let storage = InMemoryStorage::new();
        let routing_id = Uuid::new_v4();
        let first = event("first");
        let second = event("second");

        storage
            .store_event(&first, Some(routing_id), Some("topic".to_string()), None)
            .await
            .unwrap();
        storage
            .store_event(&second, None, None, Some("failure".to_string()))
            .await
            .unwrap();

        let by_type = storage.get_events_by_type("first", 10, 0).await.unwrap();
        assert_eq!(by_type.len(), 1);
        assert_eq!(by_type[0].event_id, first.id);

        let by_routing = storage
            .get_events_by_routing(routing_id, 10, 0)
            .await
            .unwrap();
        assert_eq!(by_routing.len(), 1);
        assert_eq!(by_routing[0].event_id, first.id);

        let (sampled, total) = storage.get_sample_events(1, 1).await.unwrap();
        assert_eq!(total, 2);
        assert_eq!(sampled, vec![first]);
    }
}
