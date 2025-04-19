use std::fmt;

use crate::publisher::kafka_publisher::KafkaPublisherConfig;
use crate::publisher::mqtt_publisher::MqttPublisherConfig;
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
    Mqtt(MqttPublisherConfig),
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
    pub username: String,
    pub password: String,
    pub endpoint: String,
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
