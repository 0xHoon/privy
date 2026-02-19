mod clap_banner;
mod clap_styles;
mod db;
mod getpass;
mod secret;
mod vault;

use crate::clap_banner::BANNER;
use crate::clap_styles::CARGO_STYLING;
use crate::getpass::prompt_secret;
use crate::vault::Vault;
use age::secrecy::ExposeSecret;
use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Subcommand;
use clap::{Parser, ValueEnum};
use tabled::Table;
use tabled::settings::Style;

/// Secretly - a secrets manager for linux users
///
/// secretly create my-secret
/// secretly list
/// secretly get my-secret
/// secretly grab my-secret
/// secretly reveal my-secret
#[derive(Parser, Debug)]
#[command(name = "secretly")]
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
    ///
    /// Stores the encrypted value in the local vault. Optional flags let you
    /// set a description, mark as frozen (immutable), and attach an expiry.
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

    /// Describe a single secret
    ///
    /// Shows details about the specified secret, such as description,
    /// creation/update timestamps, expiry, and whether it's frozen. The secret
    /// value is intentionally redacted. For the actual value, use `grab` to copy
    /// it to the clipboard or `reveal` to print it.
    ///
    /// Output format can be selected via `--format` and supports `json`, `yaml`,
    /// and `table`.
    Describe {
        /// Name of the secret to describe
        name: String,

        #[arg(short, long, value_enum, default_value_t = OutputFormat::Json)]
        /// Output format (json|yaml|table)
        format: OutputFormat,
    },

    /// Copy a secret's value to the system clipboard
    ///
    /// Decrypts the secret and pushes it onto your system clipboard (if supported)
    /// for pasting into other applications. The value is never printed to stdout.
    /// Be aware that clipboard contents may persist and be accessible to other
    /// apps/users.
    Grab {
        /// Name of the secret to copy
        name: String,
    },

    /// Print a secret's value as plaintext to stdout
    ///
    /// Decrypts the secret and writes the value to standard output. This is
    /// convenient for scripting but risky on shared machines or when shells and
    /// CI systems log command output. Prefer `grab` when possible.
    Reveal {
        /// Name of the secret to reveal
        name: String,
    },
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    env_logger::init();

    let args = Cli::parse();

    let mut vault = Vault::init(None).await?;

    match args.command {
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
