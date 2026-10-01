//! On-chain профиль на PDA: один профиль на кошелёк.
//!
//! Адрес профиля = PDA(["profile", authority], program_id). Приватного ключа у такого
//! адреса нет, создавать и менять аккаунт может только эта программа.

pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use instructions::*;

declare_id!("95m1n2nyxVMRKSNtJR9jCTGeLF7KasESR7zeyseH9hHu");

#[program]
pub mod profile {
    use super::*;

    /// Создаёт PDA-аккаунт профиля владельца.
    pub fn create_profile(
        ctx: Context<CreateProfile>,
        username: String,
        bio: String,
    ) -> Result<()> {
        instructions::create_profile::handle_create_profile(ctx, username, bio)
    }

    /// Меняет username и bio.
    pub fn update_profile(
        ctx: Context<UpdateProfile>,
        username: String,
        bio: String,
    ) -> Result<()> {
        instructions::update_profile::handle_update_profile(ctx, username, bio)
    }

    /// Закрывает профиль, rent возвращается владельцу.
    pub fn delete_profile(ctx: Context<DeleteProfile>) -> Result<()> {
        instructions::delete_profile::handle_delete_profile(ctx)
    }
}
