use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use tokio::time;
use uuid::Uuid;
use log::{info, warn, debug};
use async_trait::async_trait;

use crate::model::routing::{DataSchema, TopicRoutingRule, TopicValidationConfig};
use crate::store::storage::{Storage, StorageError};
use crate::store::postgres_storage::PostgresStorage;

/// A cached version of PostgresStorage that keeps routing rules in memory
/// and reloads them periodically to reduce database reads.
pub struct CachedPostgresStorage {
    /// The underlying PostgreSQL storage
    postgres: Arc<PostgresStorage>,
    
    /// In-memory cache of routing rules
    rules_cache: Arc<RwLock<Vec<TopicRoutingRule>>>,
    
    /// In-memory cache of topic validations
    validations_cache: Arc<RwLock<HashMap<String, Vec<TopicValidationConfig>>>>,
    
    /// Last time the cache was refreshed
    last_refresh: Arc<Mutex<Instant>>,
    
    /// Cache refresh interval
    refresh_interval: Duration,
    
    /// Whether the cache is currently being refreshed
    is_refreshing: Arc<Mutex<bool>>,
}

impl CachedPostgresStorage {
    /// Create a new CachedPostgresStorage with the given refresh interval
    pub async fn new(
        postgres: PostgresStorage,
        refresh_interval_secs: u64,
    ) -> Result<Self, StorageError> {
        // Initialize with empty caches
        let rules_cache = Arc::new(RwLock::new(Vec::new()));
        let validations_cache = Arc::new(RwLock::new(HashMap::new()));
        let last_refresh = Arc::new(Mutex::new(Instant::now()));
        let is_refreshing = Arc::new(Mutex::new(false));
        
        let storage = CachedPostgresStorage {
            postgres: Arc::new(postgres),
            rules_cache,
            validations_cache,
            last_refresh,
            refresh_interval: Duration::from_secs(refresh_interval_secs),
            is_refreshing,
        };
        
        // Initial cache load
        storage.refresh_cache().await?;
        
        // Start background refresh task
        let storage_clone = storage.clone();
        tokio::spawn(async move {
            storage_clone.background_refresh().await;
        });
        
        Ok(storage)
    }
    
    /// Clone the storage for use in the background task
    fn clone(&self) -> Self {
        CachedPostgresStorage {
            postgres: Arc::clone(&self.postgres),
            rules_cache: Arc::clone(&self.rules_cache),
            validations_cache: Arc::clone(&self.validations_cache),
            last_refresh: Arc::clone(&self.last_refresh),
            refresh_interval: self.refresh_interval,
            is_refreshing: Arc::clone(&self.is_refreshing),
        }
    }
    
    /// Background task to refresh the cache periodically
    async fn background_refresh(&self) {
        let mut interval = time::interval(self.refresh_interval);
        
        loop {
            interval.tick().await;
            
            // Check if we're already refreshing
            {
                let mut is_refreshing = self.is_refreshing.lock().unwrap();
                if *is_refreshing {
                    continue;
                }
                *is_refreshing = true;
            }
            
            // Refresh the cache
            match self.refresh_cache().await {
                Ok(_) => {
                    debug!("Cache refreshed successfully");
                }
                Err(e) => {
                    warn!("Failed to refresh cache: {:?}", e);
                }
            }
            
            // Mark as no longer refreshing
            {
                let mut is_refreshing = self.is_refreshing.lock().unwrap();
                *is_refreshing = false;
            }
        }
    }
    
    /// Refresh the cache from the database
    async fn refresh_cache(&self) -> Result<(), StorageError> {
        // Load rules from database
        let rules = self.postgres.get_all_rules().await?;
        
        // Load validations from database
        let validations = self.postgres.get_all_topic_validations().await?;
        
        // Update the caches
        {
            let mut rules_cache = self.rules_cache.write().unwrap();
            *rules_cache = rules;
        }
        
        {
            let mut validations_cache = self.validations_cache.write().unwrap();
            *validations_cache = validations;
        }
        
        // Update last refresh time
        {
            let mut last_refresh = self.last_refresh.lock().unwrap();
            *last_refresh = Instant::now();
        }
        
        info!("Cache refreshed with {} rules and {} topic validations", 
            self.rules_cache.read().unwrap().len(),
            self.validations_cache.read().unwrap().len());
        
        Ok(())
    }
    
    /// Force a refresh of the cache
    pub async fn force_refresh(&self) -> Result<(), StorageError> {
        // Check if we're already refreshing
        {
            let mut is_refreshing = self.is_refreshing.lock().unwrap();
            if *is_refreshing {
                return Ok(());
            }
            *is_refreshing = true;
        }
        
        // Refresh the cache
        let result = self.refresh_cache().await;
        
        // Mark as no longer refreshing
        {
            let mut is_refreshing = self.is_refreshing.lock().unwrap();
            *is_refreshing = false;
        }
        
        result
    }
}

#[async_trait]
impl Storage for CachedPostgresStorage {
    async fn add_rule(&self, rule: &TopicRoutingRule) -> Result<(), StorageError> {
        // Add to database
        self.postgres.add_rule(rule).await?;
        
        // Force refresh cache to include the new rule
        self.force_refresh().await
    }
    
    async fn get_rule(&self, id: Uuid) -> Result<Option<TopicRoutingRule>, StorageError> {
        // Check if cache needs refresh
        let needs_refresh = {
            let last_refresh = self.last_refresh.lock().unwrap();
            last_refresh.elapsed() >= self.refresh_interval
        };
        
        if needs_refresh {
            // Try to refresh in the background
            let storage_clone = self.clone();
            tokio::spawn(async move {
                if let Err(e) = storage_clone.force_refresh().await {
                    warn!("Background cache refresh failed: {:?}", e);
                }
            });
        }
        
        // Get from cache
        let rules = self.rules_cache.read().unwrap();
        Ok(rules.iter().find(|r| r.id == id).cloned())
    }
    
    async fn get_all_rules(&self) -> Result<Vec<TopicRoutingRule>, StorageError> {
        // Check if cache needs refresh
        let needs_refresh = {
            let last_refresh = self.last_refresh.lock().unwrap();
            last_refresh.elapsed() >= self.refresh_interval
        };
        
        if needs_refresh {
            // Try to refresh in the background
            let storage_clone = self.clone();
            tokio::spawn(async move {
                if let Err(e) = storage_clone.force_refresh().await {
                    warn!("Background cache refresh failed: {:?}", e);
                }
            });
        }
        
        // Get from cache
        let rules = self.rules_cache.read().unwrap();
        Ok(rules.clone())
    }
    
    async fn update_rule(&self, id: Uuid, rule: &TopicRoutingRule) -> Result<(), StorageError> {
        // Update in database
        self.postgres.update_rule(id, rule).await?;
        
        // Force refresh cache to include the updated rule
        self.force_refresh().await
    }
    
    async fn delete_rule(&self, id: Uuid) -> Result<(), StorageError> {
        // Delete from database
        self.postgres.delete_rule(id).await?;
        
        // Force refresh cache to remove the deleted rule
        self.force_refresh().await
    }
    
    async fn add_topic_validation(&self, v: &TopicValidationConfig) -> Result<(), StorageError> {
        // Add to database
        self.postgres.add_topic_validation(v).await?;
        
        // Force refresh cache to include the new validation
        self.force_refresh().await
    }
    
    async fn get_all_topic_validations(&self) -> Result<HashMap<String, Vec<TopicValidationConfig>>, StorageError> {
        // Check if cache needs refresh
        let needs_refresh = {
            let last_refresh = self.last_refresh.lock().unwrap();
            last_refresh.elapsed() >= self.refresh_interval
        };
        
        if needs_refresh {
            // Try to refresh in the background
            let storage_clone = self.clone();
            tokio::spawn(async move {
                if let Err(e) = storage_clone.force_refresh().await {
                    warn!("Background cache refresh failed: {:?}", e);
                }
            });
        }
        
        // Get from cache
        let validations = self.validations_cache.read().unwrap();
        Ok(validations.clone())
    }
    
    async fn delete_topic_validation(&self, id: &Uuid) -> Result<(), StorageError> {
        // Delete from database
        self.postgres.delete_topic_validation(id).await?;
        
        // Force refresh cache to remove the deleted validation
        self.force_refresh().await
    }

    async fn store_event(&self, event: &crate::model::event::Event, routing_id: Option<Uuid>, destination_topic: Option<String>, failure_reason: Option<String>) -> Result<(), StorageError> {
        // Delegate to the underlying PostgresStorage
        self.postgres.store_event(event, routing_id, destination_topic, failure_reason).await
    }

    async fn get_event(&self, id: Uuid) -> Result<Option<crate::store::storage::StoredEvent>, StorageError> {
        // Delegate to the underlying PostgresStorage
        self.postgres.get_event(id).await
    }

    async fn get_events_by_type(&self, event_type: &str, limit: i64, offset: i64) -> Result<Vec<crate::store::storage::StoredEvent>, StorageError> {
        // Delegate to the underlying PostgresStorage
        self.postgres.get_events_by_type(event_type, limit, offset).await
    }

    async fn get_events_by_routing(&self, routing_id: Uuid, limit: i64, offset: i64) -> Result<Vec<crate::store::storage::StoredEvent>, StorageError> {
        // Delegate to the underlying PostgresStorage
        self.postgres.get_events_by_routing(routing_id, limit, offset).await
    }
} 