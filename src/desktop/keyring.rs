use keyring::Entry;

#[derive(Debug, Clone)]
pub struct TokenKeyring {
    service: String,
    account: String,
}

impl TokenKeyring {
    pub fn new(service: impl Into<String>, account: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            account: account.into(),
        }
    }

    fn entry(&self) -> Result<Entry, String> {
        Entry::new(&self.service, &self.account).map_err(|e| format!("keyring entry: {e}"))
    }

    pub fn get(&self) -> Result<Option<String>, String> {
        match self.entry()?.get_password() {
            Ok(v) => Ok(Some(v)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(format!("keyring get: {e}")),
        }
    }

    pub fn set(&self, value: &str) -> Result<(), String> {
        self.entry()?
            .set_password(value)
            .map_err(|e| format!("keyring set: {e}"))
    }

    pub fn delete(&self) -> Result<(), String> {
        match self.entry()?.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(format!("keyring delete: {e}")),
        }
    }
}
