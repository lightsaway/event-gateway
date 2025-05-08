#![allow(warnings)]
mod configuration;
mod gateway;
mod http;
mod model;
mod publisher;
mod router;
mod store;
mod ui;

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
    postgres_storage::PostgresStorage,
    cached_postgres_storage::CachedPostgresStorage,
    storage::{InMemoryStorage, Storage},
};

use crate::gateway::gateway::EventGateway;
use crate::gateway::gateway::GateWay;
use crate::publisher::mqtt_publisher::MqttPublisher;
use crate::ui::static_handler;

async fn load_storage(config: DatabaseConfig) -> Box<dyn Storage> {
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
            let postgres = PostgresStorage::new(&postgres_config).await.unwrap();
            let cached_postgres = CachedPostgresStorage::new(
                postgres,
                postgres_config.cache_refresh_interval_secs,
            ).await.unwrap();
            Box::new(cached_postgres)
        }
    }
}

fn load_publisher(config: PublisherConfig) -> Box<dyn Publisher<Event> + Send + Sync> {
    match config {
        PublisherConfig::NoOp => Box::new(NoOpPublisher),
        PublisherConfig::Kafka(kafka_config) => Box::new(KafkaPublisher::new(kafka_config)),
        PublisherConfig::Mqtt(mqtt_config) => Box::new(MqttPublisher::new(mqtt_config)),
    }
}

fn load_configuration() -> Result<AppConfig, String> {
    let config_path = std::env::var("APP_CONFIG_PATH").unwrap_or("config".to_string());
    info!("Loading config from {}", config_path);
    
    let mut cfg = Config::builder()
        .add_source(config::File::with_name(config_path.as_str()))
        .add_source(config::Environment::with_prefix("APP")
            .separator("_")
            .list_separator(",")
            .with_list_parse_key("gateway.publisher.brokers")
            .with_list_parse_key("GATEWAY_PUBLISHER_BROKERS")
            .try_parsing(true))
            .build()
            .unwrap();
    
    let config = cfg.try_deserialize::<AppConfig>()
        .map_err(|e| e.to_string())?;
    
    info!("Loaded database config: {:?}", config.database);
    info!("Loaded gateway config: {:?}", config.gateway);
    info!("Loaded publisher config: {:?}", config.gateway.publisher);
    Ok(config)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let app_config = load_configuration().unwrap();
    info!("Loaded config: {}", app_config);
    let storage = load_storage(app_config.database).await;
    let publisher = load_publisher(app_config.gateway.publisher);
    let gateway = EventGateway::new(
        publisher,
        storage,
        app_config.gateway.sampling_enabled,
        app_config.gateway.sampling_threshold,
    ).map_err(|e| {
        error!("Failed to create gateway: {}", e);
        e
    })?;
    let service = Arc::new(gateway);
    info!("Loaded Gateway");

    let base_router = app_router(service, &app_config.api).await;
    let app = Router::new()
        .merge(base_router)
        .fallback(static_handler);

    let ip = app_config.server.host.parse::<IpAddr>().unwrap();
    let addr = SocketAddr::from((ip, app_config.server.port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!(
        "🚀 Started Server at {}:{}",
        app_config.server.host, app_config.server.port
    );
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
