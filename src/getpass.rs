use age::secrecy::SecretString;
use anyhow::Result;
use std::env;

/// Reads a configuration value from env var and fallback to prompting for it if it's not set.
pub fn get_or_prompt_secret(env_var: &str, prompt: &str) -> Result<SecretString> {
    // TODO: use keychain / passkey
    env::var(env_var).map(SecretString::from).or_else(|_| {
        let secret = rpassword::prompt_password(prompt)?;
        Ok(SecretString::from(secret))
    })
}

pub fn prompt_secret(prompt: &str) -> Result<SecretString> {
    Ok(SecretString::from(rpassword::prompt_password(prompt)?))
}
