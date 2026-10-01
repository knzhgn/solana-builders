//! Обработчики команд. Каждый возвращает текст для вывода или ToolError.

use crate::cli::Command;
use crate::errors::ToolResult;
use crate::instruction::InstructionType;
use crate::pubkey::Pubkey;
use crate::units::{base_fee, format_sol, lamports_to_sol, sol_to_lamports};
use crate::wallet::Wallet;

/// Принимает `&Command` — заимствует команду, не забирая владение.
pub fn run(cmd: &Command) -> ToolResult<String> {
    match cmd {
        Command::Lamports { sol } => lamports(sol),
        Command::Sol { lamports } => Ok(sol(*lamports)),
        Command::Validate { pubkey } => validate(pubkey),
        Command::Fee {
            signatures,
            balance,
            payer,
        } => fee(*signatures, balance.as_deref(), payer.as_deref()),
        Command::ParseIx { kind } => parse_ix(kind),
    }
}

/// 1234567 -> "1 234 567"
pub fn group(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(' ');
        }
        out.push(c);
    }
    out
}

fn lamports(sol: &str) -> ToolResult<String> {
    let l = sol_to_lamports(sol)?; // `?` пробрасывает ToolError наверх
    Ok(format!("{} SOL = {} lamports", sol.trim(), group(l)))
}

fn sol(lamports: u64) -> String {
    let approx: f64 = lamports_to_sol(lamports);
    format!(
        "{} lamports = {} SOL (f64: {approx})",
        group(lamports),
        format_sol(lamports)
    )
}

fn validate(s: &str) -> ToolResult<String> {
    let pk = Pubkey::from_base58(s)?;
    let mut out = format!(
        "OK: корректный публичный ключ, длина {} байта\n{pk}",
        pk.as_bytes().len()
    );
    if pk.is_system_program() {
        out.push_str("\nэто System Program");
    }
    Ok(out)
}

fn fee(signatures: u64, balance: Option<&str>, payer: Option<&str>) -> ToolResult<String> {
    let fee = base_fee(signatures)?;
    let mut out = format!(
        "подписей: {signatures}\nбазовая комиссия: {} lamports = {} SOL",
        group(fee),
        format_sol(fee)
    );
    // Option: баланс передан не всегда
    if let Some(b) = balance {
        let mut wallet = Wallet::new(String::from("fee payer"), 0);
        wallet.deposit(sol_to_lamports(b)?)?;
        if let Some(p) = payer {
            // with_pubkey забирает wallet по значению и возвращает обратно
            wallet = wallet.with_pubkey(Pubkey::from_base58(p)?);
        }
        let before = wallet.lamports();
        wallet.pay_fee(signatures)?; // &mut заимствование внутри метода
        let after = wallet.balance_sol();
        let who = match wallet.pubkey {
            Some(pk) => format!("{} {pk}", wallet.into_label()),
            None => wallet.into_label(), // кошелёк больше не нужен — забираем строку
        };
        out.push_str(&format!(
            "\n{who}: {} SOL -> после комиссии {after} SOL",
            format_sol(before)
        ));
    }
    Ok(out)
}

fn parse_ix(kind: &str) -> ToolResult<String> {
    let ix: InstructionType = kind.parse()?;
    Ok(format!(
        "{ix} (индекс {} в System Program)\n{}\naccounts: {}",
        ix.index(),
        ix.description(),
        ix.accounts().join("; ")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ToolError;

    #[test]
    fn group_digits() {
        assert_eq!(group(0), "0");
        assert_eq!(group(5_000), "5 000");
        assert_eq!(group(1_000_000_000), "1 000 000 000");
    }

    #[test]
    fn run_dispatches_commands() {
        let out = run(&Command::Lamports { sol: "0.1".into() }).unwrap();
        assert_eq!(out, "0.1 SOL = 100 000 000 lamports");
        let out = run(&Command::ParseIx {
            kind: "transfer".into(),
        })
        .unwrap();
        assert!(out.starts_with("Transfer (индекс 2"));
    }

    #[test]
    fn fee_with_balance_uses_wallet() {
        let cmd = Command::Fee {
            signatures: 2,
            balance: Some("1".into()),
            payer: None,
        };
        assert!(run(&cmd).unwrap().ends_with("после комиссии 0.99999 SOL"));
        let poor = Command::Fee {
            signatures: 1,
            balance: Some("0.000001".into()),
            payer: None,
        };
        assert_eq!(
            run(&poor),
            Err(ToolError::InsufficientFunds {
                needed: 5_000,
                available: 1_000
            })
        );
    }

    #[test]
    fn errors_bubble_up() {
        assert!(matches!(
            run(&Command::Validate {
                pubkey: "xyz0".into()
            }),
            Err(ToolError::InvalidBase58(_))
        ));
        assert!(matches!(
            run(&Command::ParseIx {
                kind: "mint".into()
            }),
            Err(ToolError::UnknownInstruction(_))
        ));
    }
}
