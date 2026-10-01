//! sol-tools — мини-утилиты для Solana-разработчика.

mod cli;
mod commands;
mod errors;
mod instruction;
mod pubkey;
mod units;
mod wallet;

use std::process::ExitCode;

use clap::Parser;

fn main() -> ExitCode {
    let args = cli::Cli::parse();
    match commands::run(&args.command) {
        Ok(text) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("ошибка: {e}");
            ExitCode::FAILURE
        }
    }
}
