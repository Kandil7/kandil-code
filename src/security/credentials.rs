use crate::utils::config::SecureKey;
use anyhow::{Context, Result};
use keyring::Entry;
use secrecy::{ExposeSecret, SecretString};

/// Represents the presence of a credential in the OS keyring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialState {
    Present,
    Missing,
}

/// Centralized credential manager.
///
/// Wraps the lower-level `SecureKey` helper and provides richer diagnostics
/// plus convenience helpers for storing and retrieving provider secrets.
pub struct CredentialManager;

impl CredentialManager {
    /// Returns `CredentialState::Present` if the provider has a key in the keyring.
    pub fn ensure(provider: &str) -> Result<CredentialState> {
        let entry = Entry::new("kandil", provider)?;
        match entry.get_password() {
            Ok(value) => {
                if value.is_empty() {
                    anyhow::bail!("Key for provider {provider} exists but is empty");
                }
                Ok(CredentialState::Present)
            }
            Err(keyring::Error::NoEntry) => Ok(CredentialState::Missing),
            Err(err) => Err(err.into()),
        }
    }

    /// Retrieves a provider secret from the OS keyring.
    pub fn get(provider: &str) -> Result<SecretString> {
        let key = SecureKey::load(provider)
            .with_context(|| format!("Missing API key for provider '{provider}' in keyring"))?;
        Ok(SecretString::new(key.expose().to_string()))
    }

    /// Stores/updates a provider secret.
    pub fn set(provider: &str, secret: &SecretString) -> Result<()> {
        SecureKey::save(provider, secret.expose_secret())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_credentials_reported() {
        // Generate a random provider name to guarantee absence.
        let provider = format!("test-provider-{}", uuid::Uuid::new_v4());
        let state = CredentialManager::ensure(&provider).unwrap();
        assert_eq!(state, CredentialState::Missing);
    }
}
