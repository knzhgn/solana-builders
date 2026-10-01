use anchor_lang::prelude::*;

/// Коды начинаются с 6000.
#[error_code]
pub enum ProfileError {
    #[msg("Username длиннее 32 байт")]
    UsernameTooLong,
    #[msg("Bio длиннее 160 байт")]
    BioTooLong,
    #[msg("Username не может быть пустым")]
    EmptyUsername,
    #[msg("Только владелец может менять профиль")]
    Unauthorized,
}
