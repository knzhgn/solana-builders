use anchor_lang::prelude::*;

use crate::state::Counter;

#[derive(Accounts)]
pub struct Initialize<'info> {
    /// Новый аккаунт: Anchor сам вызывает System Program (create_account) с нужным
    /// размером и rent-exempt балансом, а владельцем назначает эту программу.
    /// Адрес — обычный keypair, поэтому новый аккаунт тоже должен подписать транзакцию.
    #[account(
        init,
        payer = authority,
        space = Counter::DISCRIMINATOR.len() + Counter::INIT_SPACE
    )]
    pub counter: Account<'info, Counter>,

    /// Платит за создание и становится владельцем счётчика. `mut`: с него списываются lamports.
    #[account(mut)]
    pub authority: Signer<'info>,

    /// Нужна для CPI в System Program при `init`.
    pub system_program: Program<'info, System>,
}

pub fn handle_initialize(ctx: Context<Initialize>) -> Result<()> {
    let counter = &mut ctx.accounts.counter;
    counter.authority = ctx.accounts.authority.key();
    counter.count = 0;
    Ok(())
}
