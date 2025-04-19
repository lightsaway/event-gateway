use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use uuid::Uuid;
use async_trait::async_trait;

use crate::model::routing::{DataSchema, TopicRoutingRule, TopicValidationConfig};
use crate::store::storage::Storage;
use crate::store::storage::StorageError;

pub struct FileStorage {
    filepath: PathBuf,
    lock: Mutex<()>,
}

#[derive(Serialize, Deserialize)]
struct FileDatabase {
    rules: HashMap<Uuid, TopicRoutingRule>,
    topic_validations: HashMap<String, Vec<TopicValidationConfig>>,
}

impl FileStorage {
    pub fn new(filepath: PathBuf) -> Self {
        FileStorage {
            filepath,
            lock: Mutex::new(()),
        }
    }

    fn raw(&self) -> Result<String, StorageError> {
        let mut file = File::open(&self.filepath)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents)
    }

    fn read_database(&self) -> Result<FileDatabase, StorageError> {
        if self.filepath.exists() && self.filepath.metadata()?.len() > 0 {
            let file = File::open(&self.filepath)?;
            let reader = io::BufReader::new(file);
            // Attempt to deserialize the JSON content into a HashMap
            let db = serde_json::from_reader(reader)?;
            Ok(db)
        } else {
            // If the file doesn't exist or is empty, return an empty HashMap
            Ok(FileDatabase {
                rules: HashMap::new(),
                topic_validations: HashMap::new(),
            })
        }
    }

    fn write_database(&self, db: FileDatabase) -> Result<(), StorageError> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.filepath)?;
        let writer = io::BufWriter::new(file);
        serde_json::to_writer(writer, &db)?;
        Ok(())
    }
}

#[async_trait]
impl Storage for FileStorage {
    async fn add_rule(&self, rule: &TopicRoutingRule) -> Result<(), StorageError> {
        let _lock = self.lock.lock().unwrap();
        let mut dbs = self.read_database()?;
        dbs.rules.insert(rule.id, rule.to_owned());
        self.write_database(dbs)
    }

    async fn get_rule(&self, id: Uuid) -> Result<Option<TopicRoutingRule>, StorageError> {
        let _lock = self.lock.lock().unwrap();
        let dbs = self.read_database()?;
        Ok(dbs.rules.get(&id).cloned())
    }

    async fn get_all_rules(&self) -> Result<Vec<TopicRoutingRule>, StorageError> {
        let _lock = self.lock.lock().unwrap();
        let dbs = self.read_database()?;
        Ok(dbs.rules.into_iter().map(|(_, rule)| rule).collect())
    }

    async fn update_rule(&self, id: Uuid, rule: &TopicRoutingRule) -> Result<(), StorageError> {
        let _lock = self.lock.lock().unwrap();
        let mut dbs = self.read_database()?;

        if dbs.rules.contains_key(&id) {
            dbs.rules.insert(id, rule.to_owned());
            self.write_database(dbs)
        } else {
            Err(StorageError::NotFound)
        }
    }

    async fn delete_rule(&self, id: Uuid) -> Result<(), StorageError> {
        let _lock = self.lock.lock().unwrap();
        let mut dbs = self.read_database()?;

        if dbs.rules.remove(&id).is_some() {
            self.write_database(dbs)
        } else {
            Err(StorageError::NotFound)
        }
    }

    async fn add_topic_validation(
        &self,
        v: &crate::model::routing::TopicValidationConfig,
    ) -> Result<(), StorageError> {
        todo!()
    }

    async fn get_all_topic_validations(&self) -> Result<HashMap<String, Vec<DataSchema>>, StorageError> {
        let _lock = self.lock.lock().unwrap();
        let dbs = self.read_database()?;
        let validations: HashMap<String, Vec<DataSchema>> = dbs.topic_validations
            .into_iter()
            .map(|(topic, configs)| {
                let schemas = configs.into_iter()
                    .map(|config| config.schema)
                    .collect();
                (topic, schemas)
            })
            .collect();
        Ok(validations)
    }

    async fn delete_topic_validation(&self, id: &Uuid) -> Result<(), StorageError> {
        let _lock = self.lock.lock().unwrap();
        let mut dbs = self.read_database()?;
        let topic_validations: HashMap<String, Vec<TopicValidationConfig>> = dbs
            .topic_validations
            .iter()
            .map(|(k, v)| {
                (
                    k.to_owned(),
                    v.iter()
                        .cloned()
                        .filter(|validation| validation.to_owned().id != *id)
                        .collect(),
                )
            })
            .collect();
        dbs.topic_validations = topic_validations;
        self.write_database(dbs)
    }
}

#[cfg(test)]
mod tests {
    use crate::model::expressions::{Condition, StringExpression};

    use super::*;
    use std::fs::File;
    use tempfile::NamedTempFile;

    // Helper function to create a new TopicRoutingRule with dummy data
    fn create_dummy_rule() -> TopicRoutingRule {
        TopicRoutingRule {
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
        }
    }

    #[tokio::test]
    async fn test_file_storage_add_and_get_rule() -> Result<(), StorageError> {
        // Create a temporary file
        let file: NamedTempFile = NamedTempFile::new()?;
        let file_path = file.path().to_path_buf();
        // Instantiate FileStorage
        let mut storage = FileStorage::new(file_path);

        // Create and add a new rule to the storage
        let rule = create_dummy_rule();
        storage.add_rule(&rule.clone()).await?;

        // Retrieve the rule we just added
        let retrieved_rule = storage.get_rule(rule.id).await?;

        // Check if the retrieved rule is the same as the one we added
        assert_eq!(retrieved_rule, Some(rule));
        Ok(())
    }

    #[tokio::test]
    async fn test_file_storage_get_all_rules() -> Result<(), StorageError> {
        let file: NamedTempFile = NamedTempFile::new()?;
        let file_path = file.path().to_path_buf();

        let mut storage = FileStorage::new(file_path);

        // Add multiple rules
        let rule1 = create_dummy_rule();
        let rule2 = create_dummy_rule();
        storage.add_rule(&rule1.clone()).await?;
        storage.add_rule(&rule2.clone()).await?;

        // Retrieve all rules
        let rules = storage.get_all_rules().await?;

        // We should have 2 rules now
        assert_eq!(rules.len(), 2);
        Ok(())
    }

    #[tokio::test]
    async fn test_file_storage_update_rule() -> Result<(), StorageError> {
        let file: NamedTempFile = NamedTempFile::new()?;
        let file_path = file.path().to_path_buf();

        let mut storage = FileStorage::new(file_path);

        // Add a rule, then update it
        let mut rule = create_dummy_rule();
        storage.add_rule(&rule.clone()).await?;

        // Change the rule's field, here we'll just clone and modify the id for simplicity
        rule.description = Some("new description".to_string());
        storage.update_rule(rule.id, &rule).await?;

        // Check if the update was successful
        let retrieved_rule = storage.get_rule(rule.id).await?;
        assert_eq!(retrieved_rule, Some(rule));
        Ok(())
    }

    #[tokio::test]
    async fn test_file_storage_delete_rule() -> Result<(), StorageError> {
        let file: NamedTempFile = NamedTempFile::new()?;
        let file_path = file.path().to_path_buf();

        let mut storage = FileStorage::new(file_path);

        // Add a rule, then delete it
        let rule = create_dummy_rule();
        storage.add_rule(&rule.clone()).await?;
        storage.delete_rule(rule.id).await?;

        // Check if the rule is gone
        let retrieved_rule = storage.get_rule(rule.id).await?;
        assert_eq!(retrieved_rule, None);
        Ok(())
    }
}
