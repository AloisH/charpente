//! Argon2id hashing with explicit parameters (OWASP 2024 baseline:
//! 19 MiB memory, 2 iterations, 1 lane). Explicit so an argon2 crate upgrade
//! cannot silently change the work factor.

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::{Algorithm, Argon2, Params, Version};

fn hasher() -> anyhow::Result<Argon2<'static>> {
    let params = Params::new(19_456, 2, 1, None)
        .map_err(|e| anyhow::anyhow!("invalid argon2 params: {e}"))?;
    Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
}

pub fn hash(password: &str) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = hasher()?
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("password hashing failed: {e}"))?;
    Ok(hash.to_string())
}

pub fn verify(stored_hash: &str, password: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(stored_hash) else {
        return false;
    };
    let Ok(argon2) = hasher() else {
        return false;
    };
    argon2.verify_password(password.as_bytes(), &parsed).is_ok()
}

/// Burn the same CPU as a real verification when the account does not exist,
/// so login timing does not reveal which emails are registered.
pub fn verify_dummy(password: &str) {
    // Any valid argon2id hash with the production parameters works here.
    const DUMMY: &str = "$argon2id$v=19$m=19456,t=2,p=1$q0DzsJTNfB9zHledqPmC2A$\
                         xw2C9zkTfWyl1uFAHy4TCJcYA/M9zA1ZctD9m3TXvJc";
    let _ = verify(DUMMY, password);
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn roundtrip() {
        let h = hash("hunter2secret").unwrap();
        assert!(verify(&h, "hunter2secret"));
        assert!(!verify(&h, "wrong-password"));
    }

    #[test]
    fn bad_hash_is_false_not_panic() {
        assert!(!verify("not-a-hash", "whatever"));
    }
}
