//! Точка входа: рантайм Solana вызывает её для каждой инструкции, адресованной программе.

use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::processor::Processor;

solana_program::entrypoint_no_alloc!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    Processor::process(program_id, accounts, data)
}

/// Свой panic-handler: стандартный форматирует сообщение паники и тянет в .so весь core::fmt.
#[cfg(all(feature = "custom-panic", target_os = "solana"))]
#[unsafe(no_mangle)]
fn custom_panic(_info: &core::panic::PanicInfo<'_>) {
    solana_program::msg!("panic");
}
