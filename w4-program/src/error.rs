//! Собственные ошибки программы. В транзакции видны как `custom program error: 0x…`.

use solana_program::program_error::ProgramError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum CounterError {
    /// Неизвестная инструкция или битые данные инструкции.
    InvalidInstruction = 0,
    /// Аккаунт Counter принадлежит не этой программе.
    IncorrectOwner = 1,
    /// Аккаунт Counter не помечен как writable.
    AccountNotWritable = 2,
    /// Authority не подписал транзакцию.
    MissingSignature = 3,
    /// Размер данных аккаунта не равен Counter::LEN.
    InvalidAccountSize = 4,
    /// Initialize для уже инициализированного счётчика.
    AlreadyInitialized = 5,
    /// Increment для неинициализированного счётчика.
    NotInitialized = 6,
    /// Подписант не совпадает с authority счётчика.
    Unauthorized = 7,
    /// count достиг u64::MAX.
    Overflow = 8,
}

impl From<CounterError> for ProgramError {
    fn from(e: CounterError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
