use std::path::Path;
use log::{info, error};
use tokio_postgres::NoTls;

use crate::configuration::PostgresDatabaseConfig;
use crate::store::storage::StorageError;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("migrations");
}

pub async fn run_migrations(config: &PostgresDatabaseConfig) -> Result<(), StorageError> {
    info!("Running database migrations...");
    
    // Parse endpoint which contains both host and port
    let (host, port) = parse_endpoint(&config.endpoint);
    
    // Connect to the database
    let (mut client, connection) = tokio_postgres::connect(
        &format!(
            "host={} port={} user={} password={} dbname={}",
            host, port, config.username, config.password, config.dbname
        ),
        NoTls,
    )
    .await
    .map_err(|e| StorageError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("Database connection error: {}", e);
        }
    });

    // Run the migrations
    embedded::migrations::runner()
        .run_async(&mut client)
        .await
        .map_err(|e| StorageError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Migration error: {}", e),
        )))?;
    
    info!("Database migrations completed successfully");
    Ok(())
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