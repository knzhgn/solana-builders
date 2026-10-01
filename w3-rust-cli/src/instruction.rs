//! Типы инструкций System Program.

use std::fmt;
use std::str::FromStr;

use crate::errors::ToolError;

/// Подмножество `SystemInstruction`. Номер варианта = индекс инструкции в System Program
/// (первые 4 байта data транзакции, u32 little-endian).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionType {
    CreateAccount,
    Assign,
    Transfer,
    CreateAccountWithSeed,
    AdvanceNonceAccount,
    Allocate,
}

impl InstructionType {
    pub const ALL_NAMES: [&'static str; 6] = [
        "CreateAccount",
        "Assign",
        "Transfer",
        "CreateAccountWithSeed",
        "AdvanceNonceAccount",
        "Allocate",
    ];

    /// Поиск по имени без учёта регистра и разделителей: "create-account" == "CreateAccount".
    /// Возвращает `Option`: «нашли / не нашли», без объяснения причины.
    pub fn from_name(name: &str) -> Option<InstructionType> {
        let key: String = name
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect::<String>()
            .to_lowercase();
        match key.as_str() {
            "createaccount" => Some(Self::CreateAccount),
            "assign" => Some(Self::Assign),
            "transfer" => Some(Self::Transfer),
            "createaccountwithseed" => Some(Self::CreateAccountWithSeed),
            "advancenonceaccount" => Some(Self::AdvanceNonceAccount),
            "allocate" => Some(Self::Allocate),
            _ => None,
        }
    }

    /// Индекс инструкции в System Program.
    pub fn index(&self) -> u32 {
        match self {
            Self::CreateAccount => 0,
            Self::Assign => 1,
            Self::Transfer => 2,
            Self::CreateAccountWithSeed => 3,
            Self::AdvanceNonceAccount => 4,
            Self::Allocate => 8,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::CreateAccount => {
                "создать новый аккаунт, внести lamports и назначить программу-владельца"
            }
            Self::Assign => "сменить программу-владельца аккаунта",
            Self::Transfer => "перевести lamports с одного аккаунта на другой",
            Self::CreateAccountWithSeed => "создать аккаунт по адресу, производному от base + seed",
            Self::AdvanceNonceAccount => "обновить durable nonce",
            Self::Allocate => "выделить место под данные аккаунта",
        }
    }

    /// Какие аккаунты нужны инструкции и с какими правами.
    pub fn accounts(&self) -> &'static [&'static str] {
        match self {
            Self::CreateAccount => &[
                "funding (signer, writable)",
                "new account (signer, writable)",
            ],
            Self::Assign | Self::Allocate => &["account (signer, writable)"],
            Self::Transfer => &["from (signer, writable)", "to (writable)"],
            Self::CreateAccountWithSeed => &[
                "funding (signer, writable)",
                "created (writable)",
                "base (signer)",
            ],
            Self::AdvanceNonceAccount => &[
                "nonce account (writable)",
                "RecentBlockhashes sysvar",
                "nonce authority (signer)",
            ],
        }
    }
}

/// `FromStr` даёт `"transfer".parse::<InstructionType>()`.
/// Здесь `Option` превращается в `Result` с понятной ошибкой.
impl FromStr for InstructionType {
    type Err = ToolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        InstructionType::from_name(s).ok_or_else(|| ToolError::UnknownInstruction(s.to_string()))
    }
}

impl fmt::Display for InstructionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_types_any_case() {
        assert_eq!("transfer".parse(), Ok(InstructionType::Transfer));
        assert_eq!("CreateAccount".parse(), Ok(InstructionType::CreateAccount));
        assert_eq!("create-account".parse(), Ok(InstructionType::CreateAccount));
        assert_eq!("ALLOCATE".parse(), Ok(InstructionType::Allocate));
    }

    #[test]
    fn indices_match_system_program() {
        assert_eq!(InstructionType::CreateAccount.index(), 0);
        assert_eq!(InstructionType::Transfer.index(), 2);
        assert_eq!(InstructionType::Allocate.index(), 8);
    }

    #[test]
    fn unknown_type_is_an_error() {
        let err = "swap".parse::<InstructionType>().unwrap_err();
        assert_eq!(err, ToolError::UnknownInstruction("swap".into()));
        assert!(err.to_string().contains("Transfer"));
        assert_eq!(InstructionType::from_name(""), None);
    }

    #[test]
    fn all_names_parse() {
        for name in InstructionType::ALL_NAMES {
            assert_eq!(InstructionType::from_name(name).unwrap().to_string(), name);
        }
    }
}
