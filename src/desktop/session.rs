use crate::client::Client;

use super::keyring::TokenKeyring;

/// Glues an SDK [`Client`] to an OS keychain entry — apps call `hydrate` at
/// boot, `persist` after `verify_otp` / oauth exchange, and `clear` on logout.
#[derive(Clone)]
pub struct SessionStore {
    keyring: TokenKeyring,
}

impl SessionStore {
    pub fn new(keyring: TokenKeyring) -> Self {
        Self { keyring }
    }

    /// Pull a saved token into the given SDK client. Returns true when a token
    /// was found and applied.
    pub fn hydrate(&self, client: &mut Client) -> Result<bool, String> {
        if let Some(token) = self.keyring.get()? {
            client.set_customer_token(token);
            return Ok(true);
        }
        Ok(false)
    }

    pub fn persist(&self, client: &mut Client, token: &str) -> Result<(), String> {
        self.keyring.set(token)?;
        client.set_customer_token(token.to_string());
        Ok(())
    }

    pub fn clear(&self, client: &mut Client) -> Result<(), String> {
        self.keyring.delete()?;
        client.clear_customer_token();
        Ok(())
    }

    pub fn has_token(&self) -> bool {
        self.keyring.get().ok().flatten().is_some()
    }
}
