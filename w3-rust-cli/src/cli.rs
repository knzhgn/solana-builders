//! Описание командной строки (clap derive).

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "sol-tools",
    version,
    about = "Мини-утилиты Solana: lamports, pubkey, fee, инструкции"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

/// Enum с данными: каждый вариант несёт свои аргументы.
#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// SOL -> lamports (точно, без float)
    Lamports {
        /// Сумма в SOL, например 1.5
        sol: String,
    },
    /// lamports -> SOL
    Sol {
        /// Сумма в lamports
        lamports: u64,
    },
    /// Проверить base58 публичный ключ (32 байта)
    Validate {
        /// Публичный ключ
        pubkey: String,
    },
    /// Базовая комиссия: 5000 lamports за подпись
    Fee {
        /// Количество подписей
        signatures: u64,
        /// Баланс кошелька в SOL: показать остаток после оплаты комиссии
        #[arg(long)]
        balance: Option<String>,
        /// Адрес плательщика комиссии (fee payer), проверяется как pubkey
        #[arg(long, requires = "balance")]
        payer: Option<String>,
    },
    /// Разобрать тип инструкции System Program
    ParseIx {
        /// Например: transfer, CreateAccount, allocate
        kind: String,
    },
}
