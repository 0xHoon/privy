use age::secrecy::SecretString;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tabled::Tabled;

fn display_datetime(p: &DateTime<Utc>) -> String {
    p.to_string()
}

fn display_secret(_p: &SecretString) -> String {
    "<redacted>".to_string()
}

#[derive(Debug, Deserialize, Serialize, Tabled)]
#[tabled(display(Option, "tabled::derive::display::option", "undefined"))]
#[tabled(display(DateTime<Utc>, "display_datetime"))]
#[tabled(display(SecretString, "display_secret"))]
pub struct Secret {
    name: String,

    #[serde(skip_serializing, skip_deserializing)]
    value: SecretString,

    /// ...
    description: String,

    /// TODO
    // tags: Vec<String>,

    /// A secret is frozen if its value is immutable. Defaults to `true`.
    frozen: bool,

    /// ...
    created_at: DateTime<Utc>,

    /// ...
    updated_at: DateTime<Utc>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// (Optional) The expiry time of the secret
    expires_at: Option<DateTime<Utc>>,
}

impl Secret {
    pub fn new(
        name: String,
        description: String,
        frozen: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            name,
            value: SecretString::default(),
            description,
            frozen,
            created_at,
            updated_at,
            expires_at,
        }
    }
}
