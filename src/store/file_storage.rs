use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use uuid::Uuid;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::model::routing::{DataSchema, TopicRoutingRule, TopicValidationConfig};
use crate::model::event::Event;
use crate::store::storage::{Storage, StorageError, StoredEvent};
use crate::model::expressions::{Condition, StringExpression};

pub struct FileStorage {
    base_path: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct FileDatabase {
    rules: HashMap<Uuid, TopicRoutingRule>,
    topic_validations: HashMap<String, Vec<TopicValidationConfig>>,
}

impl FileStorage {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        FileStorage {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    fn ensure_dir(&self, path: &Path) -> io::Result<()> {
        if !path.exists() {
            fs::create_dir_all(path)?;
        }
        Ok(())
    }

    fn get_rules_path(&self) -> PathBuf {
        self.base_path.join("rules")
    }

    fn get_rule_path(&self, id: Uuid) -> PathBuf {
        self.get_rules_path().join(format!("{}.json", id))
    }

    fn get_validations_path(&self) -> PathBuf {
        self.base_path.join("validations")
    }

    fn get_validation_path(&self, id: Uuid) -> PathBuf {
        self.get_validations_path().join(format!("{}.json", id))
    }

    fn get_events_path(&self) -> PathBuf {
        self.base_path.join("events")
    }

    fn get_event_path(&self, id: Uuid) -> PathBuf {
        self.get_events_path().join(format!("{}.json", id))
    }
}

#[async_trait]
impl Storage for FileStorage {
    async fn add_rule(&self, rule: &TopicRoutingRule) -> Result<(), StorageError> {
        self.ensure_dir(&self.get_rules_path())?;
        let path = self.get_rule_path(rule.id);
        let json = serde_json::to_string_pretty(rule)?;
        fs::write(path, json)?;
        Ok(())
    }

    async fn get_rule(&self, id: Uuid) -> Result<Option<TopicRoutingRule>, StorageError> {
        let path = self.get_rule_path(id);
        if !path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(path)?;
        let rule: TopicRoutingRule = serde_json::from_str(&content)?;
        Ok(Some(rule))
    }

    async fn get_all_rules(&self) -> Result<Vec<TopicRoutingRule>, StorageError> {
        let rules_path = self.get_rules_path();
        if !rules_path.exists() {
            return Ok(Vec::new());
        }

        let mut rules = Vec::new();
        for entry in fs::read_dir(rules_path)? {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(entry.path())?;
                let rule: TopicRoutingRule = serde_json::from_str(&content)?;
                rules.push(rule);
            }
        }
        rules.sort_by_key(|r| r.order);
        Ok(rules)
    }

    async fn update_rule(&self, id: Uuid, rule: &TopicRoutingRule) -> Result<(), StorageError> {
        let path = self.get_rule_path(id);
        if !path.exists() {
            return Err(StorageError::NotFound);
        }
        let json = serde_json::to_string_pretty(rule)?;
        fs::write(path, json)?;
        Ok(())
    }

    async fn delete_rule(&self, id: Uuid) -> Result<(), StorageError> {
        let path = self.get_rule_path(id);
        if !path.exists() {
            return Err(StorageError::NotFound);
        }
        fs::remove_file(path)?;
        Ok(())
    }

    async fn add_topic_validation(&self, v: &TopicValidationConfig) -> Result<(), StorageError> {
        self.ensure_dir(&self.get_validations_path())?;
        let path = self.get_validation_path(v.id);
        let json = serde_json::to_string_pretty(v)?;
        fs::write(path, json)?;
        Ok(())
    }

    async fn get_all_topic_validations(&self) -> Result<HashMap<String, Vec<TopicValidationConfig>>, StorageError> {
        let validations_path = self.get_validations_path();
        if !validations_path.exists() {
            return Ok(HashMap::new());
        }

        let mut validations = HashMap::new();
        for entry in fs::read_dir(validations_path)? {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(entry.path())?;
                let validation: TopicValidationConfig = serde_json::from_str(&content)?;
                validations.entry(validation.topic.clone())
                    .or_insert_with(Vec::new)
                    .push(validation);
            }
        }
        Ok(validations)
    }

    async fn delete_topic_validation(&self, id: &Uuid) -> Result<(), StorageError> {
        let path = self.get_validation_path(*id);
        if !path.exists() {
            return Err(StorageError::NotFound);
        }
        fs::remove_file(path)?;
        Ok(())
    }

    async fn store_event(&self, event: &Event, routing_id: Option<Uuid>, destination_topic: Option<String>, failure_reason: Option<String>) -> Result<(), StorageError> {
        self.ensure_dir(&self.get_events_path())?;
        
        let stored_event = StoredEvent {
            id: Uuid::new_v4(),
            event_id: event.id,
            event_type: event.event_type.clone(),
            event_version: event.event_version.clone(),
            routing_id,
            destination_topic,
            failure_reason,
            stored_at: Utc::now(),
            event_data: match &event.data {
                crate::model::event::Data::Json(data) => serde_json::to_value(data)?,
                crate::model::event::Data::String(s) => serde_json::to_value(s)?,
                crate::model::event::Data::Binary(b) => serde_json::to_value(b)?,
            },
        };

        let path = self.get_event_path(stored_event.id);
        let json = serde_json::to_string_pretty(&stored_event)?;
        fs::write(path, json)?;
        Ok(())
    }

    async fn get_event(&self, id: Uuid) -> Result<Option<StoredEvent>, StorageError> {
        let path = self.get_event_path(id);
        if !path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(path)?;
        let event: StoredEvent = serde_json::from_str(&content)?;
        Ok(Some(event))
    }

    async fn get_events_by_type(&self, event_type: &str, limit: i64, offset: i64) -> Result<Vec<StoredEvent>, StorageError> {
        let events_path = self.get_events_path();
        if !events_path.exists() {
            return Ok(Vec::new());
        }

        let mut events = Vec::new();
        for entry in fs::read_dir(events_path)? {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(entry.path())?;
                let event: StoredEvent = serde_json::from_str(&content)?;
                if event.event_type == event_type {
                    events.push(event);
                }
            }
        }

        // Sort by stored_at in descending order
        events.sort_by(|a, b| b.stored_at.cmp(&a.stored_at));

        // Apply pagination
        let start = offset as usize;
        let end = (offset + limit) as usize;
        Ok(events.into_iter().skip(start).take(end - start).collect())
    }

    async fn get_events_by_routing(&self, routing_id: Uuid, limit: i64, offset: i64) -> Result<Vec<StoredEvent>, StorageError> {
        let events_path = self.get_events_path();
        if !events_path.exists() {
            return Ok(Vec::new());
        }

        let mut events = Vec::new();
        for entry in fs::read_dir(events_path)? {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(entry.path())?;
                let event: StoredEvent = serde_json::from_str(&content)?;
                if event.routing_id == Some(routing_id) {
                    events.push(event);
                }
            }
        }

        // Sort by stored_at in descending order
        events.sort_by(|a, b| b.stored_at.cmp(&a.stored_at));

        // Apply pagination
        let start = offset as usize;
        let end = (offset + limit) as usize;
        Ok(events.into_iter().skip(start).take(end - start).collect())
    }

    async fn get_sample_events(&self, limit: i64, offset: i64) -> Result<(Vec<Event>, i64), StorageError> {
        // File storage doesn't support event sampling
        Ok((Vec::new(), 0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use uuid::Uuid;

    fn create_dummy_rule() -> TopicRoutingRule {
        TopicRoutingRule {
            id: Uuid::new_v4(),
            order: 0,
            topic: "test_topic".to_string(),
            description: None,
            event_version_condition: None,
            event_type_condition: Condition::ONE(StringExpression::Equals {
                value: "test_event".to_string(),
            }),
        }
    }

    async fn test_file_storage_add_and_get_rule() -> Result<(), StorageError> {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path());
        let rule = create_dummy_rule();
        storage.add_rule(&rule).await?;
        let retrieved = storage.get_rule(rule.id).await?.unwrap();
        assert_eq!(retrieved.id, rule.id);
        assert_eq!(retrieved.topic, rule.topic);
        Ok(())
    }

    async fn test_file_storage_get_all_rules() -> Result<(), StorageError> {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path());
        let rule1 = create_dummy_rule();
        let rule2 = create_dummy_rule();
        storage.add_rule(&rule1).await?;
        storage.add_rule(&rule2).await?;
        let rules = storage.get_all_rules().await?;
        assert_eq!(rules.len(), 2);
        Ok(())
    }

    async fn test_file_storage_update_rule() -> Result<(), StorageError> {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path());
        let mut rule = create_dummy_rule();
        storage.add_rule(&rule).await?;
        rule.topic = "updated_topic".to_string();
        storage.update_rule(rule.id, &rule).await?;
        let retrieved = storage.get_rule(rule.id).await?.unwrap();
        assert_eq!(retrieved.topic, "updated_topic");
        Ok(())
    }

    async fn test_file_storage_delete_rule() -> Result<(), StorageError> {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path());
        let rule = create_dummy_rule();
        storage.add_rule(&rule).await?;
        storage.delete_rule(rule.id).await?;
        assert!(storage.get_rule(rule.id).await?.is_none());
        Ok(())
    }
}
