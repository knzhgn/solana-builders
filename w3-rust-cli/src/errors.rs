//! Собственный тип ошибок утилиты.

use std::fmt;

/// Все ошибки sol-tools. Варианты несут данные, чтобы сообщение было конкретным.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolError {
    /// Строку не удалось разобрать как сумму SOL.
    InvalidAmount(String),
    /// У суммы больше 9 знаков после точки (меньше 1 lamport).
    TooManyDecimals(usize),
    /// Результат не помещается в u64.
    Overflow,
    /// Строка не является корректным base58.
    InvalidBase58(String),
    /// base58 корректен, но длина не 32 байта.
    InvalidPubkeyLength(usize),
    /// Неизвестный тип инструкции.
    UnknownInstruction(String),
    /// Количество подписей должно быть не меньше 1.
    NoSignatures,
    /// Не хватает lamports для списания.
    InsufficientFunds { needed: u64, available: u64 },
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ToolError::InvalidAmount(s) => write!(f, "некорректная сумма SOL: '{s}'"),
            ToolError::TooManyDecimals(n) => {
                write!(
                    f,
                    "слишком точная сумма: {n} знаков после точки, максимум 9 (1 lamport = 0.000000001 SOL)"
                )
            }
            ToolError::Overflow => write!(f, "переполнение: значение не помещается в u64"),
            ToolError::InvalidBase58(s) => write!(f, "строка не является base58: '{s}'"),
            ToolError::InvalidPubkeyLength(n) => {
                write!(
                    f,
                    "неверная длина публичного ключа: получено байт — {n}, нужно 32"
                )
            }
            ToolError::UnknownInstruction(s) => write!(
                f,
                "неизвестный тип инструкции '{s}'. Доступно: {}",
                crate::instruction::InstructionType::ALL_NAMES.join(", ")
            ),
            ToolError::NoSignatures => write!(f, "в транзакции должна быть хотя бы одна подпись"),
            ToolError::InsufficientFunds { needed, available } => write!(
                f,
                "недостаточно средств: нужно {needed} lamports, доступно {available}"
            ),
        }
    }
}

impl std::error::Error for ToolError {}

/// Короткий псевдоним, как `ProgramResult` в Solana.
pub type ToolResult<T> = Result<T, ToolError>;
