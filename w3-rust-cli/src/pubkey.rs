//! Публичный ключ Solana: 32 байта, в тексте — base58.

use std::fmt;

use crate::errors::{ToolError, ToolResult};

pub const PUBKEY_BYTES: usize = 32;

/// Tuple-struct: внутри фиксированный массив из 32 байт, как `solana_pubkey::Pubkey`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pubkey([u8; PUBKEY_BYTES]);

impl Pubkey {
    /// Разбор base58-строки. Берёт `&str` (заимствование), возвращает новый `Pubkey`.
    pub fn from_base58(s: &str) -> ToolResult<Pubkey> {
        let bytes = bs58::decode(s)
            .into_vec()
            .map_err(|_| ToolError::InvalidBase58(s.to_string()))?;
        let arr: [u8; PUBKEY_BYTES] = bytes
            .as_slice()
            .try_into()
            .map_err(|_| ToolError::InvalidPubkeyLength(bytes.len()))?;
        Ok(Pubkey(arr))
    }

    /// Заимствование байтов без копирования.
    pub fn as_bytes(&self) -> &[u8; PUBKEY_BYTES] {
        &self.0
    }

    /// System Program — это ключ из 32 нулевых байт ("1111…1111" в base58).
    pub fn is_system_program(&self) -> bool {
        self.0 == [0u8; PUBKEY_BYTES]
    }
}

impl fmt::Display for Pubkey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", bs58::encode(self.0).into_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PHANTOM: &str = "6gAYJ35qhM1WznJz5d2kRa3mG5HdC2VwvhQGvoSY1sgF";

    #[test]
    fn valid_pubkey_roundtrip() {
        let pk = Pubkey::from_base58(PHANTOM).unwrap();
        assert_eq!(pk.as_bytes().len(), 32);
        assert_eq!(pk.to_string(), PHANTOM);
    }

    #[test]
    fn system_program_is_all_zeros() {
        let pk = Pubkey::from_base58("11111111111111111111111111111111").unwrap();
        assert!(pk.is_system_program());
    }

    #[test]
    fn invalid_base58_chars() {
        // '0', 'O', 'I', 'l' не входят в алфавит base58
        let err = Pubkey::from_base58("0OIl").unwrap_err();
        assert_eq!(err, ToolError::InvalidBase58("0OIl".into()));
    }

    #[test]
    fn wrong_length() {
        // корректный base58, но 3 байта вместо 32
        assert_eq!(
            Pubkey::from_base58("abc"),
            Err(ToolError::InvalidPubkeyLength(3))
        );
        // 64 байта — это уже signature, а не ключ
        let sig = "nzorVo4VUwfuLdrTgfgcmsnuMiL92BBvTxZrkfYiK9Dsuz2orWoXxL5WoS7EEM9D2gmTYVQ8kwZcjc9Q1C1sMY9";
        assert_eq!(
            Pubkey::from_base58(sig),
            Err(ToolError::InvalidPubkeyLength(64))
        );
    }
}
