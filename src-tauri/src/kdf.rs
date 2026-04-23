use argon2::{Argon2, Algorithm, Params, Version};
use rand::RngCore;

#[derive(Debug, thiserror::Error)]
pub enum KdfError {
    #[error("argon2 failed: {0}")]
    Argon2(String),
}

pub const SALT_LEN: usize = 16;
pub const KEY_LEN: usize = 32;

pub fn random_salt() -> [u8; SALT_LEN] {
    let mut salt = [0u8; SALT_LEN];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; KEY_LEN], KdfError> {
    // OWASP 2023 recommended defaults for interactive login:
    // m = 19 MiB, t = 2, p = 1
    let params = Params::new(19 * 1024, 2, 1, Some(KEY_LEN))
        .map_err(|e| KdfError::Argon2(e.to_string()))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut key = [0u8; KEY_LEN];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| KdfError::Argon2(e.to_string()))?;
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_key_is_deterministic() {
        let salt = [7u8; SALT_LEN];
        let k1 = derive_key("hunter2", &salt).unwrap();
        let k2 = derive_key("hunter2", &salt).unwrap();
        assert_eq!(k1, k2);
    }

    #[test]
    fn different_passwords_produce_different_keys() {
        let salt = [0u8; SALT_LEN];
        let k1 = derive_key("alpha", &salt).unwrap();
        let k2 = derive_key("beta", &salt).unwrap();
        assert_ne!(k1, k2);
    }

    #[test]
    fn different_salts_produce_different_keys() {
        let k1 = derive_key("same-password", &[1u8; SALT_LEN]).unwrap();
        let k2 = derive_key("same-password", &[2u8; SALT_LEN]).unwrap();
        assert_ne!(k1, k2);
    }

    #[test]
    fn random_salt_has_correct_length() {
        assert_eq!(random_salt().len(), SALT_LEN);
    }
}
