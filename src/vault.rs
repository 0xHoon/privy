use crate::db::connect_or_create_encrypted_database;
use crate::getpass::get_or_prompt_secret;
use crate::secret::Secret;
use age::secrecy::{ExposeSecret, SecretString};
use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use log::{debug, info};
use sqlx::{Error, Row, SqliteConnection, query};
use std::env;
use std::path::PathBuf;
use arboard::Clipboard;

pub struct Vault {
    conn: SqliteConnection,
}

impl Vault {
    /// Creates a vault at the specified directory, and run any pending migrations.
    pub async fn init(path: Option<PathBuf>) -> Result<Self> {
        let vault_path = path
            .or_else(|| env::home_dir().map(|h| h.join(".privy.db")))
            .ok_or_else(|| anyhow!("Could not resolve home directory"))?;
        let conn = connect_or_create_encrypted_database(
            &vault_path,
            get_or_prompt_secret("PRIVY_VAULT_PASSWORD", "Enter vault password > ")?,
        )
        .await?;
        debug!("Initialized vault at {}", vault_path.display());
        Ok(Self { conn })
    }

    /// Creates a secret in the specified vault
    pub async fn create_secret(
        &mut self,
        name: &str,
        value: &SecretString,
        description: Option<&str>,
        frozen: bool,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<()> {
        let result = // language=sqlite
            query("INSERT INTO secret (name, value, description, frozen) VALUES (?, ?, ?, ?)")
        .bind(name)
        .bind(value.expose_secret())
        .bind(description)
        .bind(frozen)
        .execute(&mut self.conn)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(Error::Database(err)) => {
                // Check if name collision
                if let Some(code) = err.code()
                    && code == "1555"
                {
                    Err(anyhow!("Secret name already in use"))
                } else {
                    Err(anyhow!("Unexpected error: {}", err))
                }
            }
            Err(error) => Err(anyhow!("Unexpected error: {}", error)),
        }
    }

    /// Lists secrets currently stored in the vault
    pub async fn list_secrets(&mut self) -> Result<Vec<Secret>> {
        let rows = query(
            // language=sqlite
            "SELECT name, description, frozen, created_at, updated_at, expires_at FROM secret",
        )
        .fetch_all(&mut self.conn)
        .await?;

        let mut secrets = vec![];
        for row in rows {
            secrets.push(Secret::new(
                row.get("name"),
                row.get("description"),
                row.get("frozen"),
                row.get("created_at"),
                row.get("updated_at"),
                row.get("expires_at"),
            ));
        }
        Ok(secrets)
    }

    /// Describes a secret without revealing its value
    pub async fn describe_secret(&mut self, name: &str) -> Result<Secret> {
        let row = // language=sqlite
            query("SELECT name, description, frozen, created_at, updated_at, expires_at FROM secret WHERE name = ?")
                .bind(name)
                .fetch_one(&mut self.conn)
                .await?;

        let secret = Secret::new(
            row.get("name"),
            row.get("description"),
            row.get("frozen"),
            row.get("created_at"),
            row.get("updated_at"),
            row.get("expires_at"),
        );

        Ok(secret)
    }

    /// Returns the decrypted secret
    pub async fn decrypt_secret(&mut self, name: &str) -> Result<SecretString> {
        let result = // language=sqlite
            query("SELECT value FROM secret WHERE name = ?")
            .bind(name)
            .fetch_one(&mut self.conn)
            .await;

        match result {
            Ok(row) => {
                let value: String = row.get("value");
                Ok(SecretString::from(value))
            },
            Err(Error::RowNotFound) => {
                Err(anyhow!("No secret named '{}'", name))
            },
            Err(error) => {
                Err(anyhow!("Failed to decrypt secret: {}", error))
            }
        }

    }

    /// Copies a secret to the clipboard
    pub async fn decrypt_and_copy_secret(&mut self, name: &str) -> Result<()> {
        let mut clipboard = Clipboard::new()?;
        let value = self.decrypt_secret(name).await?;
        clipboard.set_text(value.expose_secret())?;
        Ok(())
    }

    /// Uploads the secret to a supported remote secret store
    pub async fn upload_secret(vault: &str, name: &str, backend: &str) {
        todo!("Damn this is tough");
    }
}
