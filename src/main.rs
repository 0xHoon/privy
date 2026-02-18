mod db;
mod getpass;
mod secret;
mod vault;
mod clap_styles;
mod clap_banner;

use crate::clap_banner::BANNER;
use crate::clap_styles::CARGO_STYLING;
use crate::getpass::prompt_secret;
use crate::vault::Vault;
use age::secrecy::{ExposeSecret};
use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::{Subcommand};
use clap::{Parser, ValueEnum};
use color_eyre::owo_colors::OwoColorize;
use crossterm::style::Stylize;
use tabled::Table;
use tabled::settings::Style;

/// Privy - a secrets manager for linux users
///
/// privy create my-secret
/// privy list
/// privy get my-secret
/// privy grab my-secret
/// privy reveal my-secret
#[derive(Parser, Debug)]
#[command(name = "privy")]
#[command(about = BANNER, long_about = BANNER)]
#[clap(styles = CARGO_STYLING)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Default)]
enum OutputFormat {
    Json,
    Yaml,
    #[default]
    Table,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new secret
    Create {
        /// The (unique) name of the secret.
        name: String,

        #[arg(long, short)]
        /// A brief description of the secret.
        description: Option<String>,

        #[arg(long, short)]
        /// Whether the secret is immutable.
        frozen: Option<bool>,

        #[arg(long, short)]
        /// The expiry of the secret, if applicable.
        expires_at: Option<DateTime<Utc>>,
    },

    /// List all secrets
    List {
        #[arg(short, long, value_enum, default_value_t = OutputFormat::Json)]
        format: OutputFormat,
    },

    /// Get a secret (masked)
    Describe {
        /// Name of the secret
        name: String,

        #[arg(short, long, value_enum, default_value_t = OutputFormat::Json)]
        format: OutputFormat,
    },

    /// Copy a secret to clipboard
    Grab {
        /// Name of the secret
        name: String,
    },

    /// Reveal a secret (dangerous)
    Reveal {
        /// Name of the secret
        name: String,
    },
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    env_logger::init();

    let args = Cli::parse();

    let mut vault = Vault::init(None).await?;

    let result = match args.command {
        Commands::Create {
            name,
            description,
            frozen,
            expires_at,
        } => {
            let value = prompt_secret("Enter secret > ")?;
            vault
                .create_secret(
                    &name,
                    &value,
                    description.as_ref().map(String::as_str),
                    frozen.unwrap_or(false),
                    expires_at,
                )
                .await?;
        }
        Commands::List { format } => {
            let secrets = vault.list_secrets().await?;
            match format {
                OutputFormat::Yaml => println!("{}", serde_yaml::to_string(&secrets)?),
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&secrets)?),
                OutputFormat::Table => {
                    let mut table = Table::new(secrets);
                    table.with(Style::sharp());
                    println!("{}", table);
                }
            }
        }
        Commands::Describe { name, format } => {
            let secret = vault.describe_secret(&name).await?;
            match format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&secret)?),
                OutputFormat::Yaml => println!("{}", serde_yaml::to_string(&secret)?),
                OutputFormat::Table => {
                    let mut table = Table::new(vec![secret]);
                    table.with(Style::sharp());
                    println!("{}", table);
                }
            }
        }
        Commands::Grab { name } => {
            vault.decrypt_and_copy_secret(&name).await?;
            println!("Secret '{}' copied to clipboard!", name);
        }
        Commands::Reveal { name } => {
            let decrypted = vault.decrypt_secret(&name).await?;
            println!("{}", decrypted.expose_secret());
        }
    };

    Ok(())
}
