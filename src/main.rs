#![allow(clippy::module_inception)]

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

use axum::Router;

use config::Config;
use configuration::{AppConfig, DatabaseConfig, PublisherConfig};
use log::{error, info};
use model::event::Event;
use publisher::kafka_publisher::KafkaPublisher;
use publisher::publisher::{NoOpPublisher, Publisher};
use store::{
    cached_postgres_storage::CachedPostgresStorage,
    file_storage::FileStorage,
    postgres_storage::PostgresStorage,
    storage::{InMemoryStorage, Storage},
};

use crate::gateway::gateway::{EventGateway, GateWay};
use crate::gateway::metered::MeteredEventGateway;
use crate::publisher::mqtt_publisher::MqttPublisher;
use crate::publisher::pgmq_publisher::PgmqPublisher;
use crate::ui::static_handler;

async fn load_storage(
    config: DatabaseConfig,
) -> Result<Box<dyn Storage>, store::storage::StorageError> {
    Ok(match config {
        DatabaseConfig::File(file_config) => {
            let path = file_config.path;
            let path_buf = PathBuf::from(path);
            Box::new(FileStorage::new(path_buf))
        }
        DatabaseConfig::InMemory(config) => {
            let initial_data: InMemoryStorage = match config.initial_data_json {
                Some(json) => serde_json::from_str(&json)?,
                None => InMemoryStorage::new(),
            };
            Box::new(initial_data)
        }
        DatabaseConfig::Postgres(postgres_config) => {
            let postgres = PostgresStorage::new(&postgres_config).await?;
            let cached_postgres =
                CachedPostgresStorage::new(postgres, postgres_config.cache_refresh_interval_secs)
                    .await?;
            Box::new(cached_postgres)
        }
    })
}

async fn load_publisher(
    config: PublisherConfig,
) -> Result<Box<dyn Publisher<Event> + Send + Sync>, Box<dyn std::error::Error>> {
    Ok(match config {
        PublisherConfig::NoOp => Box::new(NoOpPublisher),
        PublisherConfig::Kafka(kafka_config) => Box::new(KafkaPublisher::new(kafka_config)?),
        PublisherConfig::Mqtt(mqtt_config) => Box::new(MqttPublisher::new(mqtt_config)),
        PublisherConfig::Pgmq(pgmq_config) => Box::new(PgmqPublisher::new(pgmq_config).await?),
    })
}

fn load_configuration() -> Result<AppConfig, config::ConfigError> {
    let config_path = std::env::var("APP_CONFIG_PATH").unwrap_or_else(|_| "config".to_string());
    info!("Loading config from {config_path}");

    let cfg = Config::builder()
        .add_source(config::File::with_name(config_path.as_str()))
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__")
                .list_separator(",")
                .with_list_parse_key("gateway.publisher.brokers")
                .try_parsing(true),
        )
        .build()?;

    cfg.try_deserialize::<AppConfig>()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let app_config = load_configuration()?;
    info!("Loaded config: {app_config}");
    let storage = load_storage(app_config.database.clone()).await?;
    let publisher = load_publisher(app_config.gateway.publisher.clone()).await?;
    let base_gateway = EventGateway::new(publisher, storage);

    let service: Arc<dyn GateWay + Send + Sync> = if app_config.gateway.metrics_enabled {
        info!("Metrics enabled - creating MeteredEventGateway");
        let metered_gateway = MeteredEventGateway::new(base_gateway).map_err(|e| {
            error!("Failed to create metered gateway: {e}");
            e
        })?;
        info!("Metrics registered successfully");
        Arc::new(metered_gateway)
    } else {
        Arc::new(base_gateway)
    };
    info!("Loaded Gateway");

    let base_router =
        app_router(service, &app_config.api, app_config.gateway.metrics_enabled).await?;
    let app = Router::new().merge(base_router).fallback(static_handler);

    let ip = app_config.server.host.parse::<IpAddr>()?;
    let addr = SocketAddr::from((ip, app_config.server.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!(
        "🚀 Started Server at {}:{}",
        app_config.server.host, app_config.server.port
    );
    axum::serve(listener, app).await?;
    Ok(())
}
