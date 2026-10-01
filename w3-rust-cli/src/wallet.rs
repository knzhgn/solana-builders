//! Простая модель кошелька: адрес + баланс в lamports.

use crate::errors::{ToolError, ToolResult};
use crate::pubkey::Pubkey;
use crate::units::{base_fee, format_sol};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wallet {
    pub label: String,
    pub pubkey: Option<Pubkey>,
    lamports: u64,
}

impl Wallet {
    /// Забирает владение `label: String` — строка переезжает внутрь Wallet.
    pub fn new(label: String, lamports: u64) -> Wallet {
        Wallet {
            label,
            pubkey: None,
            lamports,
        }
    }

    /// Builder-метод: принимает `self` по значению и возвращает его обратно (передача владения туда и обратно).
    pub fn with_pubkey(mut self, pubkey: Pubkey) -> Wallet {
        self.pubkey = Some(pubkey);
        self
    }

    /// `&self` — только чтение.
    pub fn lamports(&self) -> u64 {
        self.lamports
    }

    pub fn balance_sol(&self) -> String {
        format_sol(self.lamports)
    }

    /// `&mut self` — меняем баланс; переполнение — ошибка, а не паника.
    pub fn deposit(&mut self, amount: u64) -> ToolResult<()> {
        self.lamports = self
            .lamports
            .checked_add(amount)
            .ok_or(ToolError::Overflow)?;
        Ok(())
    }

    /// Списание с проверкой баланса, как System Program при transfer.
    pub fn withdraw(&mut self, amount: u64) -> ToolResult<()> {
        self.lamports = self
            .lamports
            .checked_sub(amount)
            .ok_or(ToolError::InsufficientFunds {
                needed: amount,
                available: self.lamports,
            })?;
        Ok(())
    }

    /// Списать базовую комиссию за `signatures` подписей; вернуть размер комиссии.
    pub fn pay_fee(&mut self, signatures: u64) -> ToolResult<u64> {
        let fee = base_fee(signatures)?;
        self.withdraw(fee)?;
        Ok(fee)
    }

    /// Поглощает Wallet и отдаёт владение строкой наружу.
    pub fn into_label(self) -> String {
        self.label
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deposit_and_withdraw() {
        let mut w = Wallet::new(String::from("main"), 1_000_000_000);
        w.deposit(500_000_000).unwrap();
        w.withdraw(100_000_000).unwrap();
        assert_eq!(w.lamports(), 1_400_000_000);
        assert_eq!(w.balance_sol(), "1.4");
    }

    #[test]
    fn withdraw_more_than_balance_fails_and_keeps_balance() {
        let mut w = Wallet::new("empty".to_string(), 4_000);
        assert_eq!(
            w.pay_fee(1),
            Err(ToolError::InsufficientFunds {
                needed: 5_000,
                available: 4_000
            })
        );
        assert_eq!(w.lamports(), 4_000);
    }

    #[test]
    fn pay_fee_like_week1_tx() {
        // неделя 1: было 1 SOL, перевод 0.1 SOL + комиссия за 1 подпись
        let mut w = Wallet::new("cli".into(), 1_000_000_000);
        w.withdraw(100_000_000).unwrap();
        assert_eq!(w.pay_fee(1), Ok(5_000));
        assert_eq!(w.lamports(), 899_995_000);
    }

    #[test]
    fn ownership_moves_in_and_out() {
        let label = String::from("phantom");
        let w = Wallet::new(label, 0); // `label` перемещена, дальше её использовать нельзя
        let pk = Pubkey::from_base58("6gAYJ35qhM1WznJz5d2kRa3mG5HdC2VwvhQGvoSY1sgF").unwrap();
        let w = w.with_pubkey(pk); // pk — Copy, остаётся доступен
        assert_eq!(w.pubkey, Some(pk));
        assert_eq!(w.into_label(), "phantom"); // w поглощён, строка вернулась наружу
    }
}
