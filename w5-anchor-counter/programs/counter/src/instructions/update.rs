use anchor_lang::prelude::*;

use crate::{error::CounterError, state::Counter};

/// Общий набор аккаунтов для increment / decrement / reset.
#[derive(Accounts)]
pub struct Update<'info> {
    /// `mut` — меняем данные. `has_one = authority` — поле counter.authority
    /// должно совпадать с ключом аккаунта `authority` ниже (иначе ошибка ConstraintHasOne).
    /// Тип Account<Counter> сам проверяет владельца (owner == program id) и discriminator.
    #[account(mut, has_one = authority)]
    pub counter: Account<'info, Counter>,

    /// Signer — транзакцию подписал именно этот ключ.
    pub authority: Signer<'info>,
}

pub fn handle_increment(ctx: Context<Update>) -> Result<()> {
    let counter = &mut ctx.accounts.counter;
    counter.count = counter
        .count
        .checked_add(1)
        .ok_or(CounterError::CounterOverflow)?;
    Ok(())
}

pub fn handle_decrement(ctx: Context<Update>) -> Result<()> {
    let counter = &mut ctx.accounts.counter;
    counter.count = counter
        .count
        .checked_sub(1)
        .ok_or(CounterError::CounterUnderflow)?;
    Ok(())
}

pub fn handle_reset(ctx: Context<Update>) -> Result<()> {
    ctx.accounts.counter.count = 0;
    Ok(())
}
