use crate::model::routing::{DataSchema, TopicRoutingRule, TopicValidationConfig};
use crate::model::event::{Event, DataType};
use crate::store::storage::{Storage, StorageError, StoredEvent};
use deadpool_postgres::{Config, Pool, Runtime};
use serde_json::Value;
use std::collections::HashMap;
use tokio_postgres::NoTls;
use uuid::Uuid;
use async_trait::async_trait;
use chrono::{DateTime, Utc, TimeZone};
use tokio_postgres::types::Type;

pub struct PostgresStorage {
    pool: Pool,
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

    async fn get_all_topic_validations(&self) -> Result<HashMap<String, Vec<TopicValidationConfig>>, StorageError> {
        let client = self.pool.get().await.map_err(|e| StorageError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        
        let stmt = client.prepare_cached(
            "SELECT id, topic, schema FROM topic_validations"
        ).await?;

        let rows = client.query(&stmt, &[]).await?;
        
        let mut validations = HashMap::new();
        for row in rows {
            let id: Uuid = row.get("id");
            let topic: String = row.get("topic");
            let schema: Value = row.get("schema");
            let schema: DataSchema = serde_json::from_value(schema)?;
            
            let config = TopicValidationConfig {
                id,
                topic: topic.clone(),
                schema,
            };
            
            validations.entry(topic)
                .or_insert_with(Vec::new)
                .push(config);
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
    
    async fn store_event(&self, event: &Event, routing_id: Option<Uuid>, destination_topic: Option<String>, failure_reason: Option<String>) -> Result<(), StorageError> {
        let client = self.pool.get().await.map_err(|e| StorageError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        
        let stmt = client.prepare_cached(
            "INSERT INTO events (id, event_id, event_type, event_version, routing_id, destination_topic, failure_reason, event_data, metadata, transport_metadata) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
        ).await?;

        let event_data = match &event.data {
            crate::model::event::Data::Json(data) => serde_json::to_value(data)?,
            crate::model::event::Data::String(s) => serde_json::to_value(s)?,
            crate::model::event::Data::Binary(b) => serde_json::to_value(b)?,
        };

        client.execute(
            &stmt,
            &[
                &Uuid::new_v4(), // Generate a new UUID for the stored event
                &event.id,
                &event.event_type,
                &event.event_version,
                &routing_id,
                &destination_topic,
                &failure_reason,
                &event_data,
                &serde_json::to_value(&event.metadata)?,
                &serde_json::to_value(&event.transport_metadata)?,
            ],
        ).await?;

        Ok(())
    }

    async fn get_event(&self, id: Uuid) -> Result<Option<StoredEvent>, StorageError> {
        let client = self.pool.get().await.map_err(|e| StorageError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        
        let stmt = client.prepare_cached(
            "SELECT id, event_id, event_type, event_version, routing_id, destination_topic, failure_reason, stored_at, event_data 
             FROM events WHERE id = $1"
        ).await?;

        let row = client.query_opt(&stmt, &[&id]).await?;
        
        if let Some(row) = row {
            Ok(Some(StoredEvent {
                id: row.get("id"),
                event_id: row.get("event_id"),
                event_type: row.get("event_type"),
                event_version: row.get("event_version"),
                routing_id: row.get("routing_id"),
                destination_topic: row.get("destination_topic"),
                failure_reason: row.get("failure_reason"),
                stored_at: row.get("stored_at"),
                event_data: row.get("event_data"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_events_by_type(&self, event_type: &str, limit: i64, offset: i64) -> Result<Vec<StoredEvent>, StorageError> {
        let client = self.pool.get().await.map_err(|e| StorageError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        
        let stmt = client.prepare_cached(
            "SELECT id, event_id, event_type, event_version, routing_id, destination_topic, failure_reason, stored_at, event_data 
             FROM events WHERE event_type = $1 ORDER BY stored_at DESC LIMIT $2 OFFSET $3"
        ).await?;

        let rows = client.query(&stmt, &[&event_type, &limit, &offset]).await?;
        
        let mut events = Vec::with_capacity(rows.len());
        for row in rows {
            events.push(StoredEvent {
                id: row.get("id"),
                event_id: row.get("event_id"),
                event_type: row.get("event_type"),
                event_version: row.get("event_version"),
                routing_id: row.get("routing_id"),
                destination_topic: row.get("destination_topic"),
                failure_reason: row.get("failure_reason"),
                stored_at: row.get("stored_at"),
                event_data: row.get("event_data"),
            });
        }
        
        Ok(events)
    }

    async fn get_events_by_routing(&self, routing_id: Uuid, limit: i64, offset: i64) -> Result<Vec<StoredEvent>, StorageError> {
        let client = self.pool.get().await.map_err(|e| StorageError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        
        let stmt = client.prepare_cached(
            "SELECT id, event_id, event_type, event_version, routing_id, destination_topic, failure_reason, stored_at, event_data 
             FROM events WHERE routing_id = $1 ORDER BY stored_at DESC LIMIT $2 OFFSET $3"
        ).await?;

        let rows = client.query(&stmt, &[&routing_id, &limit, &offset]).await?;
        
        let mut events = Vec::with_capacity(rows.len());
        for row in rows {
            events.push(StoredEvent {
                id: row.get("id"),
                event_id: row.get("event_id"),
                event_type: row.get("event_type"),
                event_version: row.get("event_version"),
                routing_id: row.get("routing_id"),
                destination_topic: row.get("destination_topic"),
                failure_reason: row.get("failure_reason"),
                stored_at: row.get("stored_at"),
                event_data: row.get("event_data"),
            });
        }
        
        Ok(events)
    }

    async fn get_sample_events(&self, limit: i64, offset: i64) -> Result<(Vec<Event>, i64), StorageError> {
        let client = self.pool.get().await.map_err(|e| StorageError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        
        // First get total count
        let count_stmt = client.prepare_cached("SELECT COUNT(*) FROM events").await?;
        let total: i64 = client.query_one(&count_stmt, &[]).await?.get(0);
        
        // Then get paginated events
        let stmt = client.prepare_cached(
            "SELECT id, event_id, event_type, event_version, event_data, metadata, transport_metadata, stored_at 
             FROM events 
             ORDER BY stored_at DESC 
             LIMIT $1 OFFSET $2"
        ).await?;

        let rows = client.query(&stmt, &[&limit, &offset]).await?;
        
        let mut events = Vec::with_capacity(rows.len());
        for row in rows {
            let event_data: Value = row.get("event_data");
            let metadata: Value = row.get("metadata");
            let transport_metadata: Value = row.get("transport_metadata");
            let stored_at: DateTime<Utc> = row.get("stored_at");

            // Convert the event_data Value into a HashMap
            let event_data_map = match event_data {
                Value::Object(map) => map.into_iter().collect::<HashMap<String, Value>>(),
                _ => HashMap::new(), // If not an object, create an empty map
            };

            events.push(Event {
                id: row.get("event_id"),
                event_type: row.get("event_type"),
                event_version: row.get("event_version"),
                metadata: serde_json::from_value(metadata).unwrap_or_default(),
                transport_metadata: serde_json::from_value(transport_metadata).ok(),
                data_type: None,
                data: crate::model::event::Data::Json(event_data_map),
                timestamp: Some(stored_at),
                origin: None,
            });
        }
        
        Ok((events, total))
    }
} 