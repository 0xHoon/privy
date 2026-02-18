use age::secrecy::{ExposeSecret, SecretString};
use anyhow::{Result, anyhow};
use log::{LevelFilter, debug, error};
use sqlx::ConnectOptions;
use sqlx::SqliteConnection;
use sqlx::migrate;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous};
use std::fs::metadata;
use std::path::Path;
use std::str::FromStr;

pub async fn connect_or_create_encrypted_database(
    database_path: &Path,
    password: SecretString,
) -> Result<SqliteConnection> {
    let os_string = &database_path.to_string_lossy().to_string();

    let mut conn = SqliteConnectOptions::from_str(os_string)?
        .pragma("key", password.expose_secret().to_string())
        .journal_mode(SqliteJournalMode::Delete)
        .log_statements(LevelFilter::Trace)
        .create_if_missing(true)
        .connect()
        .await?;
    debug!("Connected to db at {}", database_path.display());

    if !database_path.is_file() || !metadata(database_path).is_ok() {
        return Err(anyhow!("Store corrupted"));
    }

    // Always run migrations
    match migrate!("./migrations").run(&mut conn).await {
        Ok(_) => (),
        Err(e) => {
            // We deliberately avoid failing here; hoping some operations
            // are still ok (e.g. reads)
            error!("[ERROR] Failed to migrate database: {}", e);
        }
    }

    Ok(conn)
}
