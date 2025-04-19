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
    
    // Connect to the database
    let (mut client, connection) = tokio_postgres::connect(
        &format!(
            "host={} user={} password={} dbname=event_gateway",
            config.endpoint, config.username, config.password
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