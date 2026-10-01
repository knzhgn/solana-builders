use anchor_lang::prelude::*;

/// Коды ошибок Anchor начинаются с 6000.
#[error_code]
pub enum CounterError {
    #[msg("Счётчик не может быть меньше нуля")]
    CounterUnderflow,
    #[msg("Счётчик достиг максимального значения u64")]
    CounterOverflow,
}
