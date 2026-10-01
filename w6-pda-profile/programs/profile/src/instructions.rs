pub mod create_profile;
pub mod delete_profile;
pub mod update_profile;

pub use create_profile::*;
pub use delete_profile::*;
pub use update_profile::*;

use anchor_lang::prelude::*;

use crate::{
    error::ProfileError,
    state::{MAX_BIO_LEN, MAX_USERNAME_LEN},
};

/// Общая проверка длин строк (в байтах).
pub fn validate(username: &str, bio: &str) -> Result<()> {
    require!(!username.trim().is_empty(), ProfileError::EmptyUsername);
    require!(
        username.len() <= MAX_USERNAME_LEN,
        ProfileError::UsernameTooLong
    );
    require!(bio.len() <= MAX_BIO_LEN, ProfileError::BioTooLong);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn username_limits() {
        assert!(validate("valentin", "").is_ok());
        assert!(validate(&"a".repeat(32), "").is_ok());
        assert!(validate(&"a".repeat(33), "").is_err());
        assert!(validate("   ", "").is_err());
        // 16 кириллических букв = 32 байта, 17 уже нет
        assert!(validate(&"я".repeat(16), "").is_ok());
        assert!(validate(&"я".repeat(17), "").is_err());
    }

    #[test]
    fn bio_limits() {
        assert!(validate("u", &"b".repeat(160)).is_ok());
        assert!(validate("u", &"b".repeat(161)).is_err());
    }
}
