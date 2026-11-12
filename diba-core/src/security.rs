use sha2::{Sha256, Digest};

const SALT: &[u8] = b"DibaBlockerSecureSalt_2026";

pub struct SecurityManager {
    pin_hash: Option<String>,
    protection_active: bool,
    incognito_blocked: bool,
}

impl SecurityManager {
    pub fn new() -> Self {
        Self {
            pin_hash: None,
            protection_active: true,
            incognito_blocked: true,
        }
    }

    pub fn hash_pin(pin: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(SALT);
        hasher.update(pin.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn set_pin(&mut self, pin: &str) -> bool {
        if pin.len() >= 4 {
            self.pin_hash = Some(Self::hash_pin(pin));
            true
        } else {
            false
        }
    }

    pub fn verify_pin(&self, pin_attempt: &str) -> bool {
        match &self.pin_hash {
            Some(stored_hash) => {
                let attempt_hash = Self::hash_pin(pin_attempt);
                attempt_hash == *stored_hash
            }
            None => true, // No PIN configured yet
        }
    }

    pub fn is_pin_set(&self) -> bool {
        self.pin_hash.is_some()
    }

    pub fn is_active(&self) -> bool {
        self.protection_active
    }

    pub fn set_active(&mut self, active: bool) {
        self.protection_active = active;
    }

    pub fn is_incognito_blocked(&self) -> bool {
        self.incognito_blocked
    }

    pub fn set_incognito_blocked(&mut self, blocked: bool) {
        self.incognito_blocked = blocked;
    }
}
