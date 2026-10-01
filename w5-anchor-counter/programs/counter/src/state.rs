use anchor_lang::prelude::*;

/// Данные аккаунта. Anchor добавляет в начало 8-байтовый discriminator,
/// поэтому на диске аккаунт занимает 8 + INIT_SPACE = 8 + 32 + 8 = 48 байт.
#[account]
#[derive(InitSpace)]
pub struct Counter {
    pub authority: Pubkey,
    pub count: u64,
}
