//! Конвертация SOL <-> lamports и расчёт базовой комиссии.

use crate::errors::{ToolError, ToolResult};

/// 1 SOL = 1 000 000 000 lamports.
pub const LAMPORTS_PER_SOL: u64 = 1_000_000_000;
/// Базовая комиссия за одну подпись.
pub const LAMPORTS_PER_SIGNATURE: u64 = 5_000;
const SOL_DECIMALS: usize = 9;

/// Точный разбор строки "1.5" в lamports без f64: деньги не считают во float.
/// Принимает `&str` — только читает строку, владение не забирает.
pub fn sol_to_lamports(input: &str) -> ToolResult<u64> {
    let s = input.trim();
    let (whole, frac) = match s.split_once('.') {
        Some((w, f)) => (w, f),
        None => (s, ""),
    };
    let digits_only = |part: &str| part.chars().all(|c| c.is_ascii_digit());
    if (whole.is_empty() && frac.is_empty()) || !digits_only(whole) || !digits_only(frac) {
        return Err(ToolError::InvalidAmount(input.to_string()));
    }
    if frac.len() > SOL_DECIMALS {
        return Err(ToolError::TooManyDecimals(frac.len()));
    }

    let whole: u64 = if whole.is_empty() {
        0
    } else {
        whole.parse().map_err(|_| ToolError::Overflow)?
    };
    // "5" -> "500000000": дополняем дробную часть нулями до 9 знаков.
    let frac: u64 = format!("{frac:0<width$}", width = SOL_DECIMALS)
        .parse()
        .map_err(|_| ToolError::InvalidAmount(input.to_string()))?;

    whole
        .checked_mul(LAMPORTS_PER_SOL)
        .and_then(|l| l.checked_add(frac))
        .ok_or(ToolError::Overflow)
}

/// lamports -> SOL как f64 (только для отображения, точность ~15 знаков).
pub fn lamports_to_sol(lamports: u64) -> f64 {
    lamports as f64 / LAMPORTS_PER_SOL as f64
}

/// Точная строка SOL без потерь: 1500000000 -> "1.5".
pub fn format_sol(lamports: u64) -> String {
    let whole = lamports / LAMPORTS_PER_SOL;
    let frac = lamports % LAMPORTS_PER_SOL;
    if frac == 0 {
        return whole.to_string();
    }
    let frac = format!("{frac:09}");
    format!("{whole}.{}", frac.trim_end_matches('0'))
}

/// Базовая комиссия: 5000 lamports × число подписей.
pub fn base_fee(signatures: u64) -> ToolResult<u64> {
    if signatures == 0 {
        return Err(ToolError::NoSignatures);
    }
    signatures
        .checked_mul(LAMPORTS_PER_SIGNATURE)
        .ok_or(ToolError::Overflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_and_fractional_sol() {
        assert_eq!(sol_to_lamports("1"), Ok(1_000_000_000));
        assert_eq!(sol_to_lamports("1.5"), Ok(1_500_000_000));
        assert_eq!(sol_to_lamports("0.000000001"), Ok(1));
        assert_eq!(sol_to_lamports(".25"), Ok(250_000_000));
    }

    #[test]
    fn float_trap_is_avoided() {
        // 0.1 + 0.2 во float = 0.30000000000000004, а у нас ровно 300 000 000.
        assert_eq!(sol_to_lamports("0.3"), Ok(300_000_000));
    }

    #[test]
    fn rejects_bad_amounts() {
        assert_eq!(
            sol_to_lamports("abc"),
            Err(ToolError::InvalidAmount("abc".into()))
        );
        assert_eq!(
            sol_to_lamports("-1"),
            Err(ToolError::InvalidAmount("-1".into()))
        );
        assert_eq!(
            sol_to_lamports("."),
            Err(ToolError::InvalidAmount(".".into()))
        );
        assert_eq!(
            sol_to_lamports("0.0000000001"),
            Err(ToolError::TooManyDecimals(10))
        );
    }

    #[test]
    fn overflow_is_an_error_not_a_panic() {
        // u64::MAX lamports ≈ 18.4 млрд SOL
        assert_eq!(sol_to_lamports("18446744074"), Err(ToolError::Overflow));
        assert_eq!(base_fee(u64::MAX), Err(ToolError::Overflow));
    }

    #[test]
    fn lamports_back_to_sol() {
        assert_eq!(format_sol(1_500_000_000), "1.5");
        assert_eq!(format_sol(5_000), "0.000005");
        assert_eq!(format_sol(2_000_000_000), "2");
        assert_eq!(lamports_to_sol(100_000_000), 0.1);
    }

    #[test]
    fn fee_per_signature() {
        assert_eq!(base_fee(1), Ok(5_000));
        assert_eq!(base_fee(3), Ok(15_000));
        assert_eq!(base_fee(0), Err(ToolError::NoSignatures));
    }
}
