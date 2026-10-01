//! Логика инструкций и все проверки безопасности.

use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    log::sol_log_64,
    msg,
    pubkey::Pubkey,
};

use crate::{error::CounterError, instruction::CounterInstruction, state::Counter};

pub struct Processor;

impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
        match CounterInstruction::unpack(data)? {
            CounterInstruction::Initialize => {
                msg!("Instruction: Initialize");
                Self::initialize(program_id, accounts)
            }
            CounterInstruction::Increment => {
                msg!("Instruction: Increment");
                Self::increment(program_id, accounts)
            }
        }
    }

    /// Общие проверки аккаунта Counter: владелец, writable, размер.
    fn check_counter_account(program_id: &Pubkey, counter: &AccountInfo) -> ProgramResult {
        if counter.owner != program_id {
            return Err(CounterError::IncorrectOwner.into());
        }
        if !counter.is_writable {
            return Err(CounterError::AccountNotWritable.into());
        }
        if counter.data_len() != Counter::LEN {
            return Err(CounterError::InvalidAccountSize.into());
        }
        Ok(())
    }

    fn initialize(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let iter = &mut accounts.iter();
        let counter_ai = next_account_info(iter)?;
        let authority = next_account_info(iter)?;

        Self::check_counter_account(program_id, counter_ai)?;
        if !authority.is_signer {
            return Err(CounterError::MissingSignature.into());
        }
        // Rent отдельно не проверяем: рантайм сам отклонит транзакцию, после которой
        // у аккаунта с данными останется меньше rent-exempt минимума (Rent::is_exempt тянет f64-код).

        let mut state = Counter::unpack(&counter_ai.try_borrow_data()?)?;
        if state.is_initialized() {
            return Err(CounterError::AlreadyInitialized.into());
        }
        state.count = 0;
        state.authority = *authority.key;
        state.pack_into(&mut counter_ai.try_borrow_mut_data()?)?;

        authority.key.log();
        Ok(())
    }

    fn increment(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let iter = &mut accounts.iter();
        let counter_ai = next_account_info(iter)?;
        let authority = next_account_info(iter)?;

        Self::check_counter_account(program_id, counter_ai)?;
        if !authority.is_signer {
            return Err(CounterError::MissingSignature.into());
        }

        let mut state = Counter::unpack(&counter_ai.try_borrow_data()?)?;
        if !state.is_initialized() {
            return Err(CounterError::NotInitialized.into());
        }
        if state.authority != *authority.key {
            return Err(CounterError::Unauthorized.into());
        }
        state.count = state.count.checked_add(1).ok_or(CounterError::Overflow)?;
        state.pack_into(&mut counter_ai.try_borrow_mut_data()?)?;

        sol_log_64(state.count, 0, 0, 0, 0);
        Ok(())
    }
}
