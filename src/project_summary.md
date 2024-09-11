# main.rs

```rs
#![allow(warnings)]
mod configuration;
mod gateway;
mod http;
mod model;
mod publisher;
mod router;
mod store;

use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    sync::Arc,
};

use http::app_router;

use axum::response::Response;
use axum::{
    body::Body,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use config::{Config, ConfigError, Environment, File};
use configuration::{AppConfig, DatabaseConfig, PublisherConfig};
use log::{debug, error, info, warn};
use model::event::Event;
use publisher::kafka_publisher::KafkaPublisher;
use publisher::publisher::{NoOpPublisher, Publisher};
use serde::Deserialize;
use serde_json::json;
use store::{
    file_storage::FileStorage,
    storage::{InMemoryStorage, Storage},
};

use crate::gateway::gateway::EventGateway;
use crate::gateway::gateway::GateWay;

fn load_storage(config: DatabaseConfig) -> Box<dyn Storage> {
    match config {
        DatabaseConfig::File(file_config) => {
            let path = file_config.path;
            let pathBuff = PathBuf::from(path);
            Box::new(FileStorage::new(pathBuff))
        }
        DatabaseConfig::InMemory(config) => {
            let initial_data: InMemoryStorage = match config.initial_data_json {
                Some(json) => {
                    println!(" {} ", &json);
                    serde_json::from_str(&json).unwrap()
                }
                None => InMemoryStorage::new(),
            };
            Box::new(initial_data)
        }
        DatabaseConfig::Postgres(postgres_config) => {
            unimplemented!("Postgres storage not implemented")
        }
    }
}

fn load_publisher(config: PublisherConfig) -> Box<dyn Publisher<Event> + Send + Sync> {
    match config {
        PublisherConfig::NoOp => Box::new(NoOpPublisher),
        PublisherConfig::Kafka(kafka_config) => Box::new(KafkaPublisher::new(kafka_config)),
    }
}

fn load_configuration() -> Result<AppConfig, String> {
    let config_path = std::env::var("APP_CONFIG_PATH").unwrap_or("config".to_string());
    info!("Loading config from {}", config_path);
    let mut cfg = Config::builder()
        .add_source(config::File::with_name(config_path.as_str()))
        .add_source(config::Environment::with_prefix("APP"))
        .build()
        .unwrap();
    cfg.try_deserialize::<AppConfig>()
        .map_err(|e| e.to_string())
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let app_config = load_configuration().unwrap();
    info!("Loaded config: {}", app_config);
    let storage = load_storage(app_config.database);
    let publisher = load_publisher(app_config.gateway.publisher);
    let service = Arc::new(EventGateway::new(publisher, storage).unwrap());
    info!("Loaded Gateway");
    let app = app_router(service, &app_config.api).await;
    let ip = app_config.server.host.parse::<IpAddr>().unwrap();
    let addr = SocketAddr::from((ip, app_config.server.port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!(
        "🚀 Started Server at {}:{}",
        app_config.server.host, app_config.server.port
    );
    axum::serve(listener, app).await.unwrap();
}

```

# configuration.rs

```rs
use std::fmt;

use crate::publisher::kafka_publisher::KafkaPublisherConfig;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub debug_mode: bool,
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub gateway: GatewayConfig,
    pub api: ApiConfig,
}

impl fmt::Display for AppConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Configuration")?;
        writeln!(f, "=============")?;
        writeln!(f, "Debug mode: {}", self.debug_mode)?;
        writeln!(f, "Server: {}:{}", self.server.host, self.server.port)?;
        writeln!(f, "Database: {:?}", self.database)?;
        writeln!(f, "Gateway: {:?}", self.gateway)
    }
}

#[derive(Debug, Deserialize)]
pub struct GatewayConfig {
    pub metrics_enabled: bool,
    pub publisher: PublisherConfig,
}

#[derive(Debug, Deserialize)]
pub struct JwtAuthConfig {
    pub jwks_url: String,
    pub refresh_interval_secs: u64,
}

#[derive(Debug, Deserialize)]
pub struct ApiConfig {
    pub prefix: Option<String>,
    pub jwt_auth: Option<JwtAuthConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PublisherConfig {
    NoOp,
    Kafka(KafkaPublisherConfig),
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum DatabaseConfig {
    File(FileDatabaseConfig),
    InMemory(InMemoryDatabaseConfig),
    Postgres(PostgresDatabaseConfig),
}

#[derive(Debug, Deserialize)]
pub struct FileDatabaseConfig {
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct InMemoryDatabaseConfig {
    pub initial_data_json: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PostgresDatabaseConfig {
    username: String,
    password: String,
    endpoint: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::{Config, ConfigError, FileFormat};

    // Helper function to deserialize AppConfig from a string
    fn config_from_str(input: &str, format: FileFormat) -> Result<AppConfig, ConfigError> {
        Config::builder()
            .add_source(config::File::from_str(input, format))
            .build()?
            .try_deserialize::<AppConfig>()
    }

    #[test]
    fn deserialize_file_database_config() {
        let toml = r#"
            debug_mode = true

            [server]
            host = "localhost"
            port = 8080

            [database]
            type = "file"
            path = "/var/lib/myapp/data"

            [gateway]
            metrics_enabled = true
            [gateway.publisher]
            type = "noOp"
            [api]
        "#;

        let config = config_from_str(toml, FileFormat::Toml).unwrap();

        assert!(config.debug_mode);
        assert_eq!(config.server.host, "localhost");
        assert_eq!(config.server.port, 8080);
        match config.database {
            DatabaseConfig::File(file_config) => {
                assert_eq!(file_config.path, "/var/lib/myapp/data");
            }
            _ => panic!("Expected FileDatabaseConfig"),
        }
    }

    #[test]
    fn deserialize_in_memory_database_config() {
        let toml = r#"
            debug_mode = false

            [server]
            host = "localhost"
            port = 8080

            [database]
            type = "inMemory"

            [gateway]
            metrics_enabled = true
            [gateway.publisher]
            type = "noOp"

            [api]
        "#;

        let config = config_from_str(toml, FileFormat::Toml).unwrap();

        assert!(!config.debug_mode);
        match config.database {
            DatabaseConfig::InMemory(_) => (),
            _ => panic!("Expected InMemoryDatabaseConfig"),
        }
    }

    #[test]
    fn deserialize_postgres_database_config() {
        let toml = r#"
            debug_mode = false

            [server]
            host = "localhost"
            port = 8080

            [database]
            type = "postgres"
            username = "admin"
            password = "secret"
            endpoint = "localhost:5432"

            [gateway]
            metrics_enabled = true
            [gateway.publisher]
            type = "noOp"
            [api]
        "#;

        let config = config_from_str(toml, FileFormat::Toml).unwrap();

        match config.database {
            DatabaseConfig::Postgres(pg_config) => {
                assert_eq!(pg_config.username, "admin");
                assert_eq!(pg_config.password, "secret");
                assert_eq!(pg_config.endpoint, "localhost:5432");
            }
            _ => panic!("Expected PostgresDatabaseConfig"),
        }
    }
}

```

# store/storage.rs

```rs
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
        todo!()
    }

    fn delete_rule(&self, id: Uuid) -> Result<(), StorageError> {
        todo!()
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

```

# store/mod.rs

```rs
pub mod storage;
pub mod file_storage;

```

# store/file_storage.rs

```rs
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use uuid::Uuid;

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

impl Storage for FileStorage {
    fn add_rule(&self, rule: &TopicRoutingRule) -> Result<(), StorageError> {
        let _lock = self.lock.lock().unwrap();
        let mut dbs = self.read_database()?;
        dbs.rules.insert(rule.id, rule.to_owned());
        self.write_database(dbs)
    }

    fn get_rule(&self, id: Uuid) -> Result<Option<TopicRoutingRule>, StorageError> {
        let _lock = self.lock.lock().unwrap();
        let dbs = self.read_database()?;
        Ok(dbs.rules.get(&id).cloned())
    }

    fn get_all_rules(&self) -> Result<Vec<TopicRoutingRule>, StorageError> {
        let _lock = self.lock.lock().unwrap();
        let dbs = self.read_database()?;
        Ok(dbs.rules.into_iter().map(|(_, rule)| rule).collect())
    }

    fn update_rule(&self, id: Uuid, rule: &TopicRoutingRule) -> Result<(), StorageError> {
        let _lock = self.lock.lock().unwrap();
        let mut dbs = self.read_database()?;

        if dbs.rules.contains_key(&id) {
            dbs.rules.insert(id, rule.to_owned());
            self.write_database(dbs)
        } else {
            Err(StorageError::NotFound)
        }
    }

    fn delete_rule(&self, id: Uuid) -> Result<(), StorageError> {
        let _lock = self.lock.lock().unwrap();
        let mut dbs = self.read_database()?;

        if dbs.rules.remove(&id).is_some() {
            self.write_database(dbs)
        } else {
            Err(StorageError::NotFound)
        }
    }

    fn add_topic_validation(
        &self,
        v: &crate::model::routing::TopicValidationConfig,
    ) -> Result<(), StorageError> {
        todo!()
    }

    fn get_all_topic_validations(&self) -> Result<&HashMap<String, Vec<DataSchema>>, StorageError> {
        todo!()
    }

    fn delete_topic_validation(&self, id: &Uuid) -> Result<(), StorageError> {
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

    #[test]
    fn test_file_storage_add_and_get_rule() -> Result<(), StorageError> {
        // Create a temporary file
        let file: NamedTempFile = NamedTempFile::new()?;
        let file_path = file.path().to_path_buf();
        // Instantiate FileStorage
        let mut storage = FileStorage::new(file_path);

        // Create and add a new rule to the storage
        let rule = create_dummy_rule();
        storage.add_rule(&rule.clone())?;

        // Retrieve the rule we just added
        let retrieved_rule = storage.get_rule(rule.id)?;

        // Check if the retrieved rule is the same as the one we added
        assert_eq!(retrieved_rule, Some(rule));
        Ok(())
    }

    #[test]
    fn test_file_storage_get_all_rules() -> Result<(), StorageError> {
        let file: NamedTempFile = NamedTempFile::new()?;
        let file_path = file.path().to_path_buf();

        let mut storage = FileStorage::new(file_path);

        // Add multiple rules
        let rule1 = create_dummy_rule();
        let rule2 = create_dummy_rule();
        storage.add_rule(&rule1.clone())?;
        storage.add_rule(&rule2.clone())?;

        // Retrieve all rules
        let rules = storage.get_all_rules()?;

        // We should have 2 rules now
        assert_eq!(rules.len(), 2);
        Ok(())
    }

    #[test]
    fn test_file_storage_update_rule() -> Result<(), StorageError> {
        let file: NamedTempFile = NamedTempFile::new()?;
        let file_path = file.path().to_path_buf();

        let mut storage = FileStorage::new(file_path);

        // Add a rule, then update it
        let mut rule = create_dummy_rule();
        storage.add_rule(&rule.clone())?;

        // Change the rule's field, here we'll just clone and modify the id for simplicity
        rule.description = Some("new description".to_string());
        storage.update_rule(rule.id, &rule)?;

        // Check if the update was successful
        let retrieved_rule = storage.get_rule(rule.id)?;
        assert_eq!(retrieved_rule, Some(rule));
        Ok(())
    }

    #[test]
    fn test_file_storage_delete_rule() -> Result<(), StorageError> {
        let file: NamedTempFile = NamedTempFile::new()?;
        let file_path = file.path().to_path_buf();

        let mut storage = FileStorage::new(file_path);

        // Add a rule, then delete it
        let rule = create_dummy_rule();
        storage.add_rule(&rule.clone())?;
        storage.delete_rule(rule.id)?;

        // Check if the rule is gone
        let retrieved_rule = storage.get_rule(rule.id)?;
        assert_eq!(retrieved_rule, None);
        Ok(())
    }
}

```

# ruler/ruler.rs

```rs

```

# router/router.rs

```rs
use crate::model::{event::Event, routing::TopicRoutingRule};

pub struct TopicRoutings {
    pub rules: Vec<TopicRoutingRule>,
}

pub trait TopicRouter {
    fn route(&self, event: &Event) -> Option<&str>;
}

impl TopicRouter for TopicRoutings {
    fn route(&self, event: &Event) -> Option<&str> {
        for rule in &self.rules {
            let type_match = rule.event_type_condition.matches(&event.event_type);
            let version_match = match (&rule.event_version_condition, &event.event_version) {
                (Some(c), Some(v)) => c.matches(&v),
                (None, _) => true,
                _ => false,
            };
            if type_match && version_match {
                return Some(&rule.topic);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::model::{
        event::Data, event::Event, expressions::Condition, expressions::StringExpression,
    };

    use super::*;

    #[test]
    fn test_topic_router() {
        let routings = TopicRoutings {
            rules: vec![
                TopicRoutingRule {
                    id: Uuid::new_v4(),
                    order: 0,
                    topic: "topic_one".to_string(),
                    description: None,
                    event_version_condition: None,
                    event_type_condition: Condition::ONE(StringExpression::Equals {
                        value: "event_one".to_string(),
                    }),
                },
                TopicRoutingRule {
                    id: Uuid::new_v4(),
                    order: 0,
                    topic: "topic_two".to_string(),
                    description: None,
                    event_version_condition: None,
                    event_type_condition: Condition::ONE(StringExpression::Equals {
                        value: "event_two".to_string(),
                    }),
                },
            ],
        };
        let event = Event {
            id: Uuid::new_v4(),
            event_type: "event_one".to_string(),
            event_version: None,
            data: Data::String("".to_string()),
            data_type: None,
            metadata: Default::default(),
            origin: None,
            timestamp: None,
        };

        let event_two = Event {
            event_type: "event_two".to_string(),
            ..event.clone()
        };

        let event_three = Event {
            event_type: "event_three".to_string(),
            ..event.clone()
        };

        assert_eq!(routings.route(&event), Some("topic_one"));
        assert_eq!(routings.route(&event_two), Some("topic_two"));
        assert_eq!(routings.route(&event_three), None);
    }

    #[test]
    fn test_topic_router_with_version_match() {
        let routings = TopicRoutings {
            rules: vec![TopicRoutingRule {
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
            }],
        };
        let event = Event {
            id: Uuid::new_v4(),
            event_type: "event".to_string(),
            event_version: None,
            data: Data::String("".to_string()),
            data_type: None,
            metadata: Default::default(),
            origin: None,
            timestamp: None,
        };

        let event_two = Event {
            event_version: Some("1.0".to_string()),
            ..event.clone()
        };

        let event_three = Event {
            event_type: "event_three".to_string(),
            event_version: Some("3.0".to_string()),
            ..event.clone()
        };

        assert_eq!(routings.route(&event), None);
        assert_eq!(routings.route(&event_two), Some("topic"));
        assert_eq!(routings.route(&event_three), None);
    }
}

```

# router/mod.rs

```rs
pub mod router;

```

# publisher/publisher.rs

```rs
use std::error::Error;
use async_trait::async_trait;
use crate::model::event::Event;
use log::info;

#[derive(Debug)]
pub enum PublisherError {
    Generic(String),
}

#[async_trait]
pub trait Publisher<T>: Send + Sync {
   async fn publish_one(&self, topic: &str, payload: T) -> Result<(), PublisherError>;
}

pub struct NoOpPublisher;

#[async_trait]
impl Publisher<Event> for NoOpPublisher {
    async fn publish_one(&self, topic: &str, payload: Event) -> Result<(), PublisherError> {
        let event_json =
            serde_json::to_string(&payload).map_err(|e| PublisherError::Generic(e.to_string()))?;
        info!(
            "published to topic: {:?} and event: {:?}",
            topic, event_json
        );
        Ok(())
    }
}

```

# publisher/mod.rs

```rs
pub mod kafka_publisher;
pub mod publisher;

```

# publisher/kafka_publisher.rs

```rs
use std::{borrow::Borrow, fmt, time::Duration};
use duration_str::deserialize_duration;
use async_trait::async_trait;
 use serde::ser::StdError;
use rdkafka::producer::FutureProducer;
use rdkafka::ClientConfig;
use rdkafka::producer::FutureRecord;
use kafka::{
    client::{Compression, KafkaClient, RequiredAcks, DEFAULT_CONNECTION_IDLE_TIMEOUT_MILLIS},
    producer::Producer,
};
use futures::FutureExt;
use serde::{
    de::{self, Unexpected, Visitor},
    Deserialize, Deserializer, Serialize,
};

use kafka::producer::Record;

use crate::model::event::Event;

use super::publisher::{ Publisher, PublisherError};
pub struct SerdeCompression(pub Compression);

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KCompression {
    NONE,
    GZIP,
    SNAPPY,
}

impl fmt::Display for KCompression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KCompression::NONE => write!(f, "none"),
            KCompression::GZIP => write!(f, "gzip"),
            KCompression::SNAPPY => write!(f, "snappy"),
        }
    }
}

impl From<Box<dyn StdError>> for PublisherError {
    fn from(e: Box<dyn StdError>) -> Self {
        PublisherError::Generic(format!("{:?}", e))
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KRequiredAcks {
    NONE,
    ONE,
    ALL,
}

impl From<KRequiredAcks> for RequiredAcks {
    fn from(kra: KRequiredAcks) -> Self {
        match kra {
            KRequiredAcks::NONE => RequiredAcks::None,
            KRequiredAcks::ONE => RequiredAcks::One,
            KRequiredAcks::ALL => RequiredAcks::All,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct KafkaPublisherConfig {
    brokers: Vec<String>,
    compression: KCompression,
    client_id: String,
    required_acks: KRequiredAcks,
    #[serde(deserialize_with = "deserialize_duration")]
    conn_idle_timeout: Duration,
    #[serde(deserialize_with = "deserialize_duration")]
    message_timeout: Duration,
    #[serde(deserialize_with = "deserialize_duration")]
    ack_timeout: Duration,
    metadata_field_as_key: Option<String>,
}

pub struct KafkaPublisher {
    producer: FutureProducer,
    metadata_field_as_key: Option<String>,
}

impl KafkaPublisher {
    pub fn new(cfg: KafkaPublisherConfig) -> Self {
        let producer: FutureProducer = ClientConfig::new()
                .set("bootstrap.servers",  cfg.brokers.join(",") )
                .set("client.id", cfg.client_id)
                .set("compression.type", cfg.compression.to_string())
                .set("connections.max.idle.ms", cfg.conn_idle_timeout.as_millis().to_string())
                .create()
                .expect("Producer creation error");
        KafkaPublisher {
            producer,
            metadata_field_as_key: cfg.metadata_field_as_key,
        }
    }
}

#[async_trait]
impl Publisher<Event> for KafkaPublisher {
    async fn publish_one(&self, topic: &str, payload: Event) -> Result<(), PublisherError> {
        let default_key = &payload.id.to_string();
        let key = self
            .metadata_field_as_key
            .as_ref()
            .and_then(|k| payload.metadata.get(k))
            .unwrap_or(default_key);
        let value = serde_json::to_string(&payload).map_err(|e| PublisherError::Generic(e.to_string()))?;
        let result = self.producer.send(
            FutureRecord::to(&topic)
                .key(key)
                .payload(&value),
            Duration::from_secs(0),
        ).await;
        result.map(|_| ())
            .map_err(|(e, _)| PublisherError::Generic(e.to_string()))
    }
}

```

# model/routing.rs

```rs
use std::{collections::HashMap, fmt};

use super::{event::DataType, expressions::Condition};
use jsonschema::{Draft, JSONSchema};
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Serialize, Debug, PartialEq, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", content = "data")]
pub enum Schema {
    Json(JSchema),
}

impl Schema {
    pub fn is_valid(&self, data: &Value) -> bool {
        match self {
            Schema::Json(schema) => schema.compiled_schema.is_valid(data),
        }
    }
}

pub struct JSchema {
    compiled_schema: JSONSchema,
    raw_schema: Value,
    draft_version: Draft,
}

impl fmt::Debug for JSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JSchema")
            .field("raw_schema", &self.raw_schema)
            .finish()
    }
}

impl PartialEq for JSchema {
    fn eq(&self, other: &Self) -> bool {
        self.raw_schema == other.raw_schema
    }
}

impl Clone for JSchema {
    fn clone(&self) -> Self {
        // Recompile the `JSONSchema` from the stored raw schema
        let compiled_schema = JSONSchema::compile(&self.raw_schema)
            .expect("Failed to compile the schema during cloning");

        // Clone the raw schema which is just a `serde_json::Value`
        let raw_schema = self.raw_schema.clone();
        let draft_version = self.draft_version.clone();

        // Create a new `JSchema` with the recompiled schema and cloned raw schema
        JSchema {
            compiled_schema,
            raw_schema,
            draft_version,
        }
    }
}

impl<'de> Deserialize<'de> for JSchema {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        fn parse_draft_version(value: &Value) -> Draft {
            value.get("$schema").and_then(Value::as_str).map_or_else(
                || jsonschema::Draft::Draft7, // Default to Draft7 if not specified
                |uri| match uri {
                    "http://json-schema.org/draft-07/schema#" => jsonschema::Draft::Draft7,
                    "http://json-schema.org/draft-06/schema#" => jsonschema::Draft::Draft6,
                    "http://json-schema.org/draft-04/schema#" => jsonschema::Draft::Draft4,
                    _ => jsonschema::Draft::Draft7, // Default to Draft7 if unrecognized
                },
            )
        }

        // Deserialize the JSON Schema into a serde_json `Value`.
        let raw_schema = Value::deserialize(deserializer)?;
        let draft_version = parse_draft_version(&raw_schema);
        // Compile the `Value` into a `JSONSchema`.
        let compiled_schema = JSONSchema::options()
            .with_draft(draft_version) // Choose appropriate draft
            .compile(&raw_schema)
            .map_err(|e| serde::de::Error::custom(e.to_string()))?;
        Ok(JSchema {
            compiled_schema,
            raw_schema,
            draft_version,
        })
    }
}

impl Serialize for JSchema {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.raw_schema.serialize(serializer)
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Deserialize)]
pub struct DataSchema {
    pub name: String,
    pub description: Option<String>,
    pub schema: Schema,
    pub event_type: String,
    pub event_version: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Clone, Serialize, PartialEq, Deserialize)]
pub struct TopicValidationConfig {
    pub id: Uuid,
    pub topic: String,
    pub schema: DataSchema,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicRoutingRule {
    pub id: Uuid,
    pub order: i32,
    pub topic: String,
    pub event_type_condition: Condition,
    pub event_version_condition: Option<Condition>,
    pub description: Option<String>,
}

#[cfg(test)]
mod tests {
    use crate::model::expressions::StringExpression;

    use super::*;
    use serde_json;

    #[test]
    fn test_topic_routing_rule_serde() {
        let rule = TopicRoutingRule {
            id: Uuid::new_v4(),
            order: 1,
            topic: "example".into(),
            event_type_condition: Condition::ONE(StringExpression::StartsWith {
                value: "test".into(),
            }),
            event_version_condition: Some(Condition::ONE(StringExpression::Equals {
                value: "1".into(),
            })),
            description: Some("A routing rule.".into()),
        };

        let serialized = serde_json::to_string(&rule).unwrap();
        let deserialized: TopicRoutingRule = serde_json::from_str(&serialized).unwrap();
        assert_eq!(rule, deserialized);
    }

    #[test]
    fn test_data_schema_serde() {
        let raw_schema = serde_json::json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string"
                }
            },
            "required": ["name"]
        });

        let schema = DataSchema {
            name: "example".into(),
            description: Some("A schema.".into()),
            schema: Schema::Json(JSchema {
                compiled_schema: JSONSchema::compile(&raw_schema).unwrap(),
                raw_schema: raw_schema,
                draft_version: Draft::Draft7,
            }),
            event_type: "example".into(),
            event_version: Some("1".into()),
            metadata: Some(HashMap::new()),
        };

        let serialized = serde_json::to_string(&schema).unwrap();
        print!("{}", serialized);
        let deserialized: DataSchema = serde_json::from_str(&serialized).unwrap();
        assert_eq!(schema, deserialized);
    }
}

```

# model/mod.rs

```rs
pub mod event;
pub mod expressions;
pub mod routing;

```

# model/expressions.rs

```rs
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum StringExpression {
    RegexMatch {
        #[serde(
            serialize_with = "regex_serialize",
            deserialize_with = "regex_deserialize"
        )]
        value: Regex,
    },
    Equals {
        value: String,
    },
    StartsWith {
        value: String,
    },
    EndsWith {
        value: String,
    },
    Contains {
        value: String,
    },
}

impl PartialEq for StringExpression {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (StringExpression::RegexMatch { value: left }, StringExpression::RegexMatch { value: right }) => {
                left.as_str() == right.as_str()
            }
            (StringExpression::Equals { value: left }, StringExpression::Equals { value: right })
            | (StringExpression::StartsWith { value: left }, StringExpression::StartsWith { value: right })
            | (StringExpression::EndsWith { value: left }, StringExpression::EndsWith { value: right })
            | (StringExpression::Contains { value: left }, StringExpression::Contains { value: right }) => {
                left == right
            }
            _ => false,
        }
    }
}

#[derive(Clone, Serialize, PartialEq,Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Condition {
    AND(Vec<Condition>),
    OR(Vec<Condition>),
    NOT(Box<Condition>),
    ANY(),
    #[serde(untagged)]
    ONE(StringExpression),
}

fn regex_serialize<S>(regex: &Regex, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(regex.as_str())
}

fn regex_deserialize<'de, D>(deserializer: D) -> Result<Regex, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    String::deserialize(deserializer)?
        .parse()
        .map_err(D::Error::custom)
}

impl Condition {
    pub fn matches(&self, to: &str) -> bool {
        match self {
            Condition::ANY() => true,
            Condition::ONE(expr) => match expr {
                StringExpression::RegexMatch { value } => value.is_match(to),
                StringExpression::Equals { value } => value == to,
                StringExpression::StartsWith { value } => to.starts_with(value),
                StringExpression::EndsWith { value } => to.ends_with(value),
                StringExpression::Contains { value } => to.contains(value),
            },
            Condition::AND(conditions) => conditions.iter().all(|cond| cond.matches(to)),
            Condition::OR(conditions) => conditions.iter().any(|cond| cond.matches(to)),
            Condition::NOT(condition) => !condition.matches(to),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn test_any_match() {
        let condition = Condition::ANY();
        assert!(condition.matches("test123"));
        assert!(condition.matches("random"));
    }

    #[test]
    fn test_regex_match() {
        let value = Regex::new("^test.*").unwrap();
        let condition = Condition::ONE(StringExpression::RegexMatch { value });
        assert!(condition.matches("test123"));
        assert!(!condition.matches("random"));
    }

    #[test]
    fn test_equals() {
        let condition = Condition::ONE(StringExpression::Equals {
            value: "test".to_string(),
        });
        assert!(condition.matches("test"));
        assert!(!condition.matches("Test"));
    }

    #[test]
    fn test_starts_with() {
        let condition = Condition::ONE(StringExpression::StartsWith {
            value: "start".to_string(),
        });
        assert!(condition.matches("start_here"));
        assert!(!condition.matches("finish_start"));
    }

    #[test]
    fn test_ends_with() {
        let condition = Condition::ONE(StringExpression::EndsWith {
            value: "end".to_string(),
        });
        assert!(condition.matches("the_end"));
        assert!(!condition.matches("end_the"));
    }

    #[test]
    fn test_contains() {
        let condition = Condition::ONE(StringExpression::Contains {
            value: "inside".to_string(),
        });
        assert!(condition.matches("this_is_inside_that"));
        assert!(!condition.matches("outside"));
    }

    #[test]
    fn test_and_conditions() {
        let and_condition = Condition::AND(vec![
            Condition::ONE(StringExpression::StartsWith {
                value: "start".to_string(),
            }),
            Condition::ONE(StringExpression::EndsWith {
                value: "finish".to_string(),
            }),
        ]);
        assert!(and_condition.matches("start_middle_finish"));
        assert!(!and_condition.matches("start_finish_fail"));
    }

    #[test]
    fn test_or_conditions() {
        let or_condition = Condition::OR(vec![
            Condition::ONE(StringExpression::Equals {
                value: "option1".to_string(),
            }),
            Condition::ONE(StringExpression::Equals {
                value: "option2".to_string(),
            }),
        ]);
        assert!(or_condition.matches("option1"));
        assert!(or_condition.matches("option2"));
        assert!(!or_condition.matches("option3"));
    }

    #[test]
    fn test_not_condition() {
        let not_condition = Condition::NOT(Box::new(Condition::ONE(StringExpression::Equals {
            value: "nope".to_string(),
        })));
        assert!(not_condition.matches("yes"));
        assert!(!not_condition.matches("nope"));
    }

    #[test]
    fn serialize_deserialize_condition() {
        let value = Regex::new("^test.*").unwrap();
        let condition = Condition::AND(vec![
            Condition::ONE(StringExpression::RegexMatch { value }),
            Condition::ONE(StringExpression::Equals {
                value: "test123".to_string(),
            }),
        ]);

        let serialized = serde_json::to_string(&condition).expect("Failed to serialize condition");

        let deserialized: Condition =
            serde_json::from_str(&serialized).expect("Failed to deserialize condition");

        // You might want to check deserialized conditions
        // Since you cannot directly compare `Regex`, you must test the actual matching functionality
        assert!(deserialized.matches("test123"));
        assert!(!deserialized.matches("no_match"));
        assert_eq!(
            serialized,
            r#"{"and":[{"type":"regexMatch","value":"^test.*"},{"type":"equals","value":"test123"}]}"#
        );
    }
}

```

# model/event.rs

```rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Serialize, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
    Json,
    String,
    Binary,
}
#[derive(Clone, Serialize, Debug, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase", content = "content")]
pub enum Data {
    Json(HashMap<String, serde_json::Value>),
    String(String),
    Binary(Vec<u8>),
}

#[derive(Clone, Serialize, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub id: Uuid,
    pub event_type: String,
    pub event_version: Option<String>,
    pub metadata: HashMap<String, String>,
    pub data_type: Option<DataType>,
    pub data: Data,
    pub timestamp: Option<DateTime<Utc>>,
    pub origin: Option<String>,
}

impl Event {
    // fn random() -> Event {
    //     // let metadata = [
    //     //     ("key1".to_string(), "value1".to_string()),
    //     //     ("key2".to_string(), "value2".to_string()),
    //     // ]
    //     // .iter()
    //     // .cloned()
    //     // .collect();

    //     // let json_data = json!({
    //     //     "field1": "value1",
    //     //     "field2": 12345,
    //     // });

    //     // // Assuming we want to generate random data for the `Json` variant of `Data`.
    //     // let data = Data::Json(json_data.as_object().unwrap().clone());

    //     // Event {
    //     //     id: Uuid::new_v4(),
    //     //     event_type: "exampleType".to_string(),
    //     //     event_version: Some("v1".to_string()),
    //     //     metadata,
    //     //     data_type: Some(DataType::Json),
    //     //     data,
    //     //     timestamp: Some(Utc::now()),
    //     //     origin: Some("exampleOrigin".to_string()),
    //     // }
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_serde_data_type() {
        let data_type = DataType::Json;
        let serialized = serde_json::to_string(&data_type).expect("Failed to serialize");
        assert_eq!(serialized, "\"json\"");

        let deserialized: DataType =
            serde_json::from_str(&serialized).expect("Failed to deserialize");
        assert_eq!(deserialized, data_type);
    }

    #[test]
    fn test_serde() {
        let uuid = Uuid::new_v4();
        let timestamp = Utc::now();

        let data_json = json!({
            "key": "value"
        });

        let mut metadata = HashMap::new();
        metadata.insert("author".to_string(), "Alice".to_string());

        let event = Event {
            id: uuid,
            event_type: "test_type".to_string(),
            event_version: Some("1.0".to_string()),
            metadata,
            data_type: Some(DataType::Json),
            data: Data::Json(data_json.as_object().unwrap().clone().into_iter().collect()),
            timestamp: Some(timestamp),
            origin: Some("example".to_string()),
        };

        let serialized = serde_json::to_string(&event).expect("Failed to serialize");

        // Check a few serialized properties
        assert!(serialized.contains(&uuid.to_string()));
        assert!(serialized.contains("\"test_type\""));
        assert!(serialized.contains("\"author\":\"Alice\""));
        let deserialized: Event = serde_json::from_str(&serialized).expect("Failed to deserialize");
        println!("{}", serialized);
        assert_eq!(deserialized, event);
    }
}

```

# http/mod.rs

```rs
use crate::configuration::ApiConfig;
use crate::gateway::gateway::GateWay;
use crate::model::expressions::Condition;
use crate::model::routing::{DataSchema, TopicRoutingRule, TopicValidationConfig};
use crate::{gateway::gateway::EventGateway, model::event::Event};
use axum::async_trait;
use axum::extract::{FromRequestParts, Path, Request};
use axum::http::request::Parts;
use axum::middleware::Next;
use axum::routing::delete;
use axum::{
    body::Body,
    extract::{Extension, State},
    response::Response,
    routing::{get, post},
    Json, Router,
};
use futures::Future;
use hyper::header::USER_AGENT;
use hyper::StatusCode;
use jwt_authorizer::{
    Authorizer, IntoLayer, JwtAuthorizer, Refresh, RefreshStrategy, RegisteredClaims,
};
use serde::Deserialize;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tower::{Service, ServiceBuilder};
use tower_http::trace::TraceLayer;
use uuid::Uuid;

#[derive(Debug, Clone)]
struct RequestMetadata {
    originator_ip: Option<String>,
    user_agent: Option<String>,
}

#[async_trait]
impl<S> FromRequestParts<S> for RequestMetadata {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let originator_ip = parts
            .headers
            .get("x-forwarded-for")
            .and_then(|value| value.to_str().ok())
            .map(|s| s.to_owned())
            .or_else(|| {
                parts
                    .extensions
                    .get::<SocketAddr>()
                    .map(|addr| addr.ip().to_string())
            });

        let user_agent = parts
            .headers
            .get(USER_AGENT)
            .and_then(|value| value.to_str().ok())
            .map(|s| s.to_owned());

        Ok(RequestMetadata {
            originator_ip,
            user_agent,
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateTopicValidationRequest {
    topic: String,
    schema: DataSchema,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateRoutingRuleRequest {
    order: i32,
    topic: String,
    event_type_condition: Condition,
    event_version_condition: Option<Condition>,
    description: Option<String>,
}

pub async fn app_router(
    service: Arc<crate::gateway::gateway::EventGateway>,
    config: &ApiConfig,
) -> Router {
    let routes = Router::new()
        .route("/event", post(handle_event))
        .route("/routing-rules", get(read_rules))
        .route("/routing-rules", post(create_routing_rule))
        .route("/routing-rules/:id", delete(delete_routing_rule))
        .route("/topic-validations", get(read_topic_validations))
        .route("/topic-validations", post(create_topic_validation))
        .route("/topic-validations/:id", delete(delete_topic_validation))
        .route("/health-check", get(health_check))
        .with_state(service);
    let prefix = config.prefix.clone().unwrap_or("/".to_string());
    let router = match &config.jwt_auth {
        Some(cfg) => {
            let authorizer: Authorizer<RegisteredClaims> =
                JwtAuthorizer::from_jwks_url(&cfg.jwks_url)
                    .build()
                    .await
                    .unwrap();
            Router::new()
                .nest(&prefix, routes)
                .layer(authorizer.into_layer())
        }
        None => Router::new().nest(&prefix, routes),
    };
    router
        .layer(axum::middleware::from_fn(extract_request_metadata))
        .layer(TraceLayer::new_for_http())
}

async fn extract_request_metadata(
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let originator_ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_owned())
        .or_else(|| {
            req.extensions()
                .get::<SocketAddr>()
                .map(|addr| addr.ip().to_string())
        });

    let user_agent = req
        .headers()
        .get(USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_owned());

    let metadata = RequestMetadata {
        originator_ip,
        user_agent,
    };
    req.extensions_mut().insert(metadata);

    Ok(next.run(req).await)
}

async fn health_check() -> &'static str {
    r#"{ "status" : "healthy"}"#
}

async fn handle_event(
    State(service): State<Arc<EventGateway>>,
    Json(event): Json<Event>,

    Extension(claims): Extension<Option<RegisteredClaims>>,
    Extension(metadata): Extension<RequestMetadata>,
) -> Result<Response, Response> {
    let result = service.handle(&event).await;
    match result {
        Ok(_) => Ok(Response::builder().status(200).body(Body::empty()).unwrap()),
        Err(err) => match err {
            crate::gateway::gateway::GatewayError::SchemaInvalid(err) => Ok(Response::builder()
                .status(400)
                .body(Body::from(r#"{"error": "schema validation failed"}"#))
                .unwrap()),
            crate::gateway::gateway::GatewayError::NoTopicToRoute(err) => Ok(Response::builder()
                .status(406)
                .body(Body::from(r#"{"error": "no destination found"}"#))
                .unwrap()),
            crate::gateway::gateway::GatewayError::InternalError(err) => {
                Ok(Response::builder().status(500).body(Body::empty()).unwrap())
            }
        },
    }
}

async fn create_routing_rule(
    State(service): State<Arc<EventGateway>>,
    Json(request): Json<CreateRoutingRuleRequest>,
) -> Result<Response, Response> {
    let rule = TopicRoutingRule {
        id: Uuid::new_v4(),
        order: request.order,
        topic: request.topic,
        event_type_condition: request.event_type_condition,
        event_version_condition: request.event_version_condition,
        description: request.description,
    };
    let result = service.add_routing_rule(&rule).await;
    match result {
        Ok(_) => Ok(Response::builder().status(204).body(Body::empty()).unwrap()),
        Err(err) => Ok(Response::builder().status(500).body(Body::empty()).unwrap()),
    }
}

async fn delete_routing_rule(
    State(service): State<Arc<EventGateway>>,
    Path(id): Path<Uuid>,
) -> Result<Response, Response> {
    let result = service.delete_routing_rule(&id).await;
    match result {
        Ok(_) => Ok(Response::builder().status(204).body(Body::empty()).unwrap()),
        Err(err) => Ok(Response::builder().status(500).body(Body::empty()).unwrap()),
    }
}

async fn create_topic_validation(
    State(service): State<Arc<EventGateway>>,
    Json(request): Json<CreateTopicValidationRequest>,
) -> Result<Response, Response> {
    let validation = TopicValidationConfig {
        id: Uuid::new_v4(),
        topic: request.topic,
        schema: request.schema,
    };
    let result = service.add_topic_validation(&validation).await;
    match result {
        Ok(_) => Ok(Response::builder().status(204).body(Body::empty()).unwrap()),
        Err(err) => Ok(Response::builder().status(500).body(Body::empty()).unwrap()),
    }
}

async fn read_topic_validations(
    State(service): State<Arc<EventGateway>>,
) -> Result<Response, Response> {
    let result = service.get_topic_validations().await;
    match result {
        Ok(validations) => Ok(Response::builder()
            .status(200)
            .body(Body::from(serde_json::to_string(&validations).unwrap()))
            .unwrap()),
        Err(err) => Ok(Response::builder().status(500).body(Body::empty()).unwrap()),
    }
}

async fn delete_topic_validation(
    State(service): State<Arc<EventGateway>>,
    Path(id): Path<Uuid>,
) -> Result<Response, Response> {
    let result = service.delete_topic_validation(&id).await;
    match result {
        Ok(_) => Ok(Response::builder().status(204).body(Body::empty()).unwrap()),
        Err(err) => Ok(Response::builder().status(500).body(Body::empty()).unwrap()),
    }
}

async fn read_rules(State(service): State<Arc<EventGateway>>) -> Result<Response, Response> {
    let result = service.get_routing_rules().await;
    match result {
        Ok(rules) => Ok(Response::builder()
            .status(200)
            .body(Body::from(serde_json::to_string(&rules).unwrap()))
            .unwrap()),
        Err(err) => Ok(Response::builder().status(500).body(Body::empty()).unwrap()),
    }
}

```

# gateway/mod.rs

```rs
pub mod gateway;
pub mod metered;

```

# gateway/metered.rs

```rs
use crate::gateway::gateway::{GateWay, GatewayError};
use crate::model::event::Event;
use prometheus::{
    register_counter_vec, register_histogram_vec, CounterVec, Encoder, HistogramVec, Opts,
    TextEncoder,
};
use std::error::Error;

pub struct MeteredEventGateway<T: GateWay> {
    gateway: T,
    counters: CounterVec,
    histogram: HistogramVec,
}

impl<T> MeteredEventGateway<T>
where
    T: GateWay,
{
    pub fn new(gateway: T) -> Result<Self, Box<dyn std::error::Error>> {
        let counter_opts = Opts::new("events_total", "Total number of events handled");

        let counters = register_counter_vec!(
            counter_opts,
            &["event_type", "event_version", "source", "result"]
        )?;

        let histogram = register_histogram_vec!(
            "event_handling_duration_seconds",
            "Histogram of event handling durations",
            &["step"]
        )?;

        Ok(MeteredEventGateway {
            gateway,
            counters,
            histogram,
        })
    }
}

impl<T> GateWay for MeteredEventGateway<T>
where
    T: GateWay,
{
    async fn handle(&self, event: &Event) -> Result<(), GatewayError> {
        let timer = self.histogram.with_label_values(&["handle"]).start_timer();
        let result = self.gateway.handle(event).await;
        timer.observe_duration();
        let event_type_label = event.event_type.as_str(); // Change as per actual field type and structure
        let event_version_label = event.event_version.as_deref().unwrap_or("unknown");
        let source_label = event.origin.as_deref().unwrap_or("unknown");
        match &result {
            Ok(_) => {
                let labels = [
                    event_type_label,
                    event_version_label,
                    source_label,
                    "success",
                ];
                self.counters.with_label_values(&labels).inc();
            }
            Err(_) => {
                let labels = [
                    event_type_label,
                    event_version_label,
                    source_label,
                    "failure",
                ];
                self.counters.with_label_values(&labels).inc();
            }
        }
        result
    }

    async fn add_topic_validation(
        &self,
        v: &crate::model::routing::TopicValidationConfig,
    ) -> Result<(), GatewayError> {
        self.gateway.add_topic_validation(v).await
    }

    async fn delete_topic_validation(&self, id: &uuid::Uuid) -> Result<(), GatewayError> {
        self.gateway.delete_topic_validation(id).await
    }

    async fn add_routing_rule(
        &self,
        rule: &crate::model::routing::TopicRoutingRule,
    ) -> Result<(), GatewayError> {
        self.gateway.add_routing_rule(rule).await
    }

    async fn get_routing_rules(
        &self,
    ) -> Result<Vec<crate::model::routing::TopicRoutingRule>, GatewayError> {
        self.gateway.get_routing_rules().await
    }

    async fn delete_routing_rule(&self, id: &uuid::Uuid) -> Result<(), GatewayError> {
        self.gateway.delete_routing_rule(id).await
    }

    async fn get_topic_validations(
        &self,
    ) -> Result<
        &std::collections::HashMap<String, Vec<crate::model::routing::DataSchema>>,
        GatewayError,
    > {
        self.gateway.get_topic_validations().await
    }
}

```

# gateway/gateway.rs

```rs
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

pub trait GateWay {
    async fn handle(&self, event: &Event) -> Result<(), GatewayError>;

    async fn add_topic_validation(&self, v: &TopicValidationConfig) -> Result<(), GatewayError>;
    async fn delete_topic_validation(&self, id: &Uuid) -> Result<(), GatewayError>;
    async fn add_routing_rule(&self, rule: &TopicRoutingRule) -> Result<(), GatewayError>;
    async fn get_routing_rules(&self) -> Result<Vec<TopicRoutingRule>, GatewayError>;
    async fn delete_routing_rule(&self, id: &Uuid) -> Result<(), GatewayError>;

    async fn get_topic_validations(
        &self,
    ) -> Result<&HashMap<String, Vec<DataSchema>>, GatewayError>;
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
        let rules = self.store.get_all_rules().map_err(GatewayError::from)?;
        let routings = TopicRoutings { rules };

        match routings.route(&event) {
            Some(topic) => {
                let topic_schemas = self
                    .store
                    .get_validations_for_topic(topic)
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
            .map_err(GatewayError::from)
    }

    async fn delete_topic_validation(&self, id: &Uuid) -> Result<(), GatewayError> {
        self.store
            .delete_topic_validation(id)
            .map_err(GatewayError::from)
    }

    async fn add_routing_rule(&self, rule: &TopicRoutingRule) -> Result<(), GatewayError> {
        self.store.add_rule(rule).map_err(GatewayError::from)
    }

    async fn get_routing_rules(&self) -> Result<Vec<TopicRoutingRule>, GatewayError> {
        self.store.get_all_rules().map_err(GatewayError::from)
    }

    async fn delete_routing_rule(&self, id: &Uuid) -> Result<(), GatewayError> {
        self.store
            .delete_rule(id.to_owned())
            .map_err(GatewayError::from)
    }

    async fn get_topic_validations(
        &self,
    ) -> Result<&HashMap<String, Vec<DataSchema>>, GatewayError> {
        self.store
            .get_all_topic_validations()
            .map_err(GatewayError::from)
    }
}

```

