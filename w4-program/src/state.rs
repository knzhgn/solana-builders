//! Состояние, которое хранится в data аккаунта Counter.

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

use crate::error::CounterError;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Counter {
    pub count: u64,
    pub authority: Pubkey,
}

impl Counter {
    /// 8 байт u64 + 32 байта Pubkey. Borsh пишет их подряд без заголовков.
    pub const LEN: usize = 8 + 32;

    /// Новый аккаунт создаётся заполненным нулями, поэтому authority == 0…0 значит «не инициализирован».
    pub fn is_initialized(&self) -> bool {
        self.authority != Pubkey::default()
    }

    /// Чтение в программе. Байты те же, что у borsh (u64 little-endian + 32 байта ключа),
    /// но без borsh-кода ошибок, который тянет core::fmt и почти удваивает .so.
    pub fn unpack(data: &[u8]) -> Result<Self, CounterError> {
        let data: &[u8; Self::LEN] = data
            .try_into()
            .map_err(|_| CounterError::InvalidAccountSize)?;
        let (count, authority) = data.split_at(8);
        Ok(Counter {
            count: u64::from_le_bytes(
                count
                    .try_into()
                    .map_err(|_| CounterError::InvalidAccountSize)?,
            ),
            authority: Pubkey::new_from_array(
                authority
                    .try_into()
                    .map_err(|_| CounterError::InvalidAccountSize)?,
            ),
        })
    }

    /// Запись в программе в borsh-совместимом формате.
    pub fn pack_into(&self, dst: &mut [u8]) -> Result<(), CounterError> {
        if dst.len() != Self::LEN {
            return Err(CounterError::InvalidAccountSize);
        }
        dst[..8].copy_from_slice(&self.count.to_le_bytes());
        dst[8..].copy_from_slice(self.authority.as_ref());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn borsh_layout_is_40_bytes() {
        let c = Counter {
            count: 7,
            authority: Pubkey::new_from_array([9; 32]),
        };
        let bytes = borsh::to_vec(&c).unwrap();
        assert_eq!(bytes.len(), Counter::LEN);
        assert_eq!(&bytes[..8], &7u64.to_le_bytes());
        assert_eq!(&bytes[8..], &[9; 32]);
        assert_eq!(Counter::try_from_slice(&bytes).unwrap(), c);
    }

    #[test]
    fn zeroed_account_is_uninitialized() {
        let c = Counter::try_from_slice(&[0; Counter::LEN]).unwrap();
        assert!(!c.is_initialized());
        assert_eq!(c.count, 0);
    }

    #[test]
    fn manual_pack_matches_borsh() {
        let c = Counter {
            count: u64::MAX - 1,
            authority: Pubkey::new_unique(),
        };
        let mut buf = [0u8; Counter::LEN];
        c.pack_into(&mut buf).unwrap();
        assert_eq!(buf.to_vec(), borsh::to_vec(&c).unwrap());
        assert_eq!(Counter::unpack(&buf), Ok(c));
        assert_eq!(Counter::try_from_slice(&buf).unwrap(), c);
    }

    #[test]
    fn manual_unpack_checks_size() {
        assert_eq!(
            Counter::unpack(&[0; 39]),
            Err(CounterError::InvalidAccountSize)
        );
        assert_eq!(
            Counter::unpack(&[0; 41]),
            Err(CounterError::InvalidAccountSize)
        );
        assert_eq!(
            Counter::default().pack_into(&mut [0; 10]),
            Err(CounterError::InvalidAccountSize)
        );
    }

    #[test]
    fn wrong_size_fails_to_deserialize() {
        assert!(Counter::try_from_slice(&[0; Counter::LEN - 1]).is_err());
        assert!(Counter::try_from_slice(&[0; Counter::LEN + 1]).is_err());
    }
}
