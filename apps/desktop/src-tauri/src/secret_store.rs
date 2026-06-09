//! Local secret storage for cloud-provider API keys.
//!
//! Keys are kept in the OS keyring (Windows Credential Manager, macOS Keychain,
//! or the Linux Secret Service), never in the repo, never in `settings.json`, and
//! never in the SQLite store. The bridge only ever reports whether a key is set;
//! the secret value is read back solely by the model-execution path, never
//! returned to the UI.

const SERVICE: &str = "com.geaux.delyxnext";

/// Backend-agnostic secret storage so command logic can be tested with an
/// in-memory fake instead of touching the real OS keyring.
pub trait SecretStore: Send + Sync {
    fn set(&self, account: &str, value: &str) -> Result<(), String>;
    fn get(&self, account: &str) -> Result<Option<String>, String>;
    fn delete(&self, account: &str) -> Result<(), String>;
}

/// Production store backed by the OS keyring.
pub struct KeyringSecretStore;

impl SecretStore for KeyringSecretStore {
    fn set(&self, account: &str, value: &str) -> Result<(), String> {
        entry(account)?.set_password(value).map_err(keyring_error)
    }

    fn get(&self, account: &str) -> Result<Option<String>, String> {
        match entry(account)?.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(keyring_error(error)),
        }
    }

    fn delete(&self, account: &str) -> Result<(), String> {
        match entry(account)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(keyring_error(error)),
        }
    }
}

/// In-memory store for deterministic tests; never persists.
#[derive(Default)]
pub struct MemorySecretStore {
    entries: std::sync::Mutex<std::collections::HashMap<String, String>>,
}

impl SecretStore for MemorySecretStore {
    fn set(&self, account: &str, value: &str) -> Result<(), String> {
        self.lock()?.insert(account.to_string(), value.to_string());
        Ok(())
    }

    fn get(&self, account: &str) -> Result<Option<String>, String> {
        Ok(self.lock()?.get(account).cloned())
    }

    fn delete(&self, account: &str) -> Result<(), String> {
        self.lock()?.remove(account);
        Ok(())
    }
}

impl MemorySecretStore {
    fn lock(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, std::collections::HashMap<String, String>>, String> {
        self.entries
            .lock()
            .map_err(|_| "Secret store lock failed.".to_string())
    }
}

fn entry(account: &str) -> Result<keyring::Entry, String> {
    keyring::Entry::new(SERVICE, account).map_err(keyring_error)
}

fn keyring_error(error: keyring::Error) -> String {
    format!("OS keyring error: {error}")
}
