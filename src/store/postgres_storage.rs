use crate::model::routing::{DataSchema, TopicRoutingRule, TopicValidationConfig};
use crate::store::storage::{Storage, StorageError};
use deadpool_postgres::{Config, Pool, Runtime};
use serde_json::Value;
use std::collections::HashMap;
use tokio_postgres::NoTls;
use uuid::Uuid;
use async_trait::async_trait;

pub struct PostgresStorage {
    pool: Pool,
}

impl From<tokio_postgres::Error> for StorageError {
    fn from(err: tokio_postgres::Error) -> StorageError {
        StorageError::IoError(std::io::Error::new(std::io::ErrorKind::Other, err))
    }
}

impl PostgresStorage {
    pub async fn new(config: &crate::configuration::PostgresDatabaseConfig) -> Result<Self, StorageError> {
        // Run migrations first
        crate::store::migrations::run_migrations(config)
            .await
            .map_err(|e| StorageError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        let mut cfg = Config::new();
        
        // Parse endpoint which contains both host and port
        let (host, port) = parse_endpoint(&config.endpoint);
        cfg.host = Some(host);
        cfg.port = Some(port);
        cfg.user = Some(config.username.clone());
        cfg.password = Some(config.password.clone());
        cfg.dbname = Some(config.dbname.clone());

        let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)
            .map_err(|e| StorageError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        Ok(PostgresStorage { pool })
    }
}

// Helper function to parse endpoint into host and port
fn parse_endpoint(endpoint: &str) -> (String, u16) {
    if let Some(colon_pos) = endpoint.find(':') {
        let (host, port_str) = endpoint.split_at(colon_pos);
        let port = port_str.trim_start_matches(':').parse::<u16>().unwrap_or(5432);
        (host.to_string(), port)
    } else {
        (endpoint.to_string(), 5432) // Default PostgreSQL port
    }
}

#[async_trait]
impl Storage for PostgresStorage {
    async fn add_rule(&self, rule: &TopicRoutingRule) -> Result<(), StorageError> {
        let client = self.pool.get().await.map_err(|e| StorageError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        
        let stmt = client.prepare_cached(
            "INSERT INTO routing_rules (id, order_num, topic, description, event_version_condition, event_type_condition) 
             VALUES ($1, $2, $3, $4, $5, $6)"
        ).await?;

        client.execute(
            &stmt,
            &[
                &rule.id,
                &rule.order,
                &rule.topic,
                &rule.description,
                &serde_json::to_value(&rule.event_version_condition)?,
                &serde_json::to_value(&rule.event_type_condition)?,
            ],
        ).await?;

        Ok(())
    }

    async fn get_rule(&self, id: Uuid) -> Result<Option<TopicRoutingRule>, StorageError> {
        let client = self.pool.get().await.map_err(|e| StorageError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        
        let stmt = client.prepare_cached(
            "SELECT id, order_num, topic, description, event_version_condition, event_type_condition 
             FROM routing_rules WHERE id = $1"
        ).await?;

        let row = client.query_opt(&stmt, &[&id]).await?;
        
        if let Some(row) = row {
            let event_version_condition: Value = row.get("event_version_condition");
            let event_type_condition: Value = row.get("event_type_condition");
            
            Ok(Some(TopicRoutingRule {
                id: row.get("id"),
                order: row.get("order_num"),
                topic: row.get("topic"),
                description: row.get("description"),
                event_version_condition: serde_json::from_value(event_version_condition)?,
                event_type_condition: serde_json::from_value(event_type_condition)?,
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_all_rules(&self) -> Result<Vec<TopicRoutingRule>, StorageError> {
        let client = self.pool.get().await.map_err(|e| StorageError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        
        let stmt = client.prepare_cached(
            "SELECT id, order_num, topic, description, event_version_condition, event_type_condition 
             FROM routing_rules ORDER BY order_num"
        ).await?;

        let rows = client.query(&stmt, &[]).await?;
        
        let mut rules = Vec::with_capacity(rows.len());
        for row in rows {
            let event_version_condition: Value = row.get("event_version_condition");
            let event_type_condition: Value = row.get("event_type_condition");
            
            rules.push(TopicRoutingRule {
                id: row.get("id"),
                order: row.get("order_num"),
                topic: row.get("topic"),
                description: row.get("description"),
                event_version_condition: serde_json::from_value(event_version_condition)?,
                event_type_condition: serde_json::from_value(event_type_condition)?,
            });
        }
        
        Ok(rules)
    }

    async fn update_rule(&self, id: Uuid, rule: &TopicRoutingRule) -> Result<(), StorageError> {
        let client = self.pool.get().await.map_err(|e| StorageError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        
        let stmt = client.prepare_cached(
            "UPDATE routing_rules 
             SET order_num = $2, topic = $3, description = $4, 
                 event_version_condition = $5, event_type_condition = $6,
                 updated_at = NOW()
             WHERE id = $1"
        ).await?;

        let result = client.execute(
            &stmt,
            &[
                &id,
                &rule.order,
                &rule.topic,
                &rule.description,
                &serde_json::to_value(&rule.event_version_condition)?,
                &serde_json::to_value(&rule.event_type_condition)?,
            ],
        ).await?;

        if result == 0 {
            Err(StorageError::NotFound)
        } else {
            Ok(())
        }
    }

    async fn delete_rule(&self, id: Uuid) -> Result<(), StorageError> {
        let client = self.pool.get().await.map_err(|e| StorageError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        
        let stmt = client.prepare_cached("DELETE FROM routing_rules WHERE id = $1").await?;
        let result = client.execute(&stmt, &[&id]).await?;

        if result == 0 {
            Err(StorageError::NotFound)
        } else {
            Ok(())
        }
    }

    async fn add_topic_validation(&self, v: &TopicValidationConfig) -> Result<(), StorageError> {
        let client = self.pool.get().await.map_err(|e| StorageError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        
        let stmt = client.prepare_cached(
            "INSERT INTO topic_validations (id, topic, schema) VALUES ($1, $2, $3)"
        ).await?;

        client.execute(
            &stmt,
            &[
                &v.id,
                &v.topic,
                &serde_json::to_value(&v.schema)?,
            ],
        ).await?;

        Ok(())
    }

    async fn get_all_topic_validations(&self) -> Result<HashMap<String, Vec<DataSchema>>, StorageError> {
        let client = self.pool.get().await.map_err(|e| StorageError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        
        let stmt = client.prepare_cached(
            "SELECT topic, schema FROM topic_validations"
        ).await?;

        let rows = client.query(&stmt, &[]).await?;
        
        let mut validations = HashMap::new();
        for row in rows {
            let topic: String = row.get("topic");
            let schema: Value = row.get("schema");
            let schema: DataSchema = serde_json::from_value(schema)?;
            
            validations.entry(topic)
                .or_insert_with(Vec::new)
                .push(schema);
        }
        
        Ok(validations)
    }

    async fn delete_topic_validation(&self, id: &Uuid) -> Result<(), StorageError> {
        let client = self.pool.get().await.map_err(|e| StorageError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        
        let stmt = client.prepare_cached("DELETE FROM topic_validations WHERE id = $1").await?;
        let result = client.execute(&stmt, &[&id]).await?;

        if result == 0 {
            Err(StorageError::NotFound)
        } else {
            Ok(())
        }
    }
} 