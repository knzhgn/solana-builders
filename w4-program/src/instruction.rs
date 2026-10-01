//! Инструкции программы. Первый байт data — номер варианта enum (borsh).

use borsh::BorshSerialize;

use crate::error::CounterError;

#[derive(BorshSerialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CounterInstruction {
    /// count = 0, authority = подписант.
    /// Accounts: [0] counter (writable, owner = program), [1] authority (signer).
    Initialize,
    /// count += 1, только если подписал authority.
    /// Accounts: [0] counter (writable, owner = program), [1] authority (signer).
    Increment,
}

impl CounterInstruction {
    /// Разбор вручную, а не через derive(BorshDeserialize): сгенерированный borsh-код
    /// на неизвестный вариант собирает сообщение через format!, что раздувает .so на ~25 КБ.
    /// Формат тот же, что у borsh: один байт — номер варианта.
    pub fn unpack(data: &[u8]) -> Result<Self, CounterError> {
        match data {
            [0] => Ok(Self::Initialize),
            [1] => Ok(Self::Increment),
            _ => Err(CounterError::InvalidInstruction),
        }
    }

    pub fn pack(&self) -> Vec<u8> {
        borsh::to_vec(self).expect("enum без полей всегда сериализуется")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tags_are_stable() {
        assert_eq!(CounterInstruction::Initialize.pack(), vec![0]);
        assert_eq!(CounterInstruction::Increment.pack(), vec![1]);
        // unpack понимает ровно то, что пишет borsh-сериализация (pack)
        for ix in [
            CounterInstruction::Initialize,
            CounterInstruction::Increment,
        ] {
            assert_eq!(CounterInstruction::unpack(&ix.pack()), Ok(ix));
        }
    }

    #[test]
    fn bad_data_is_invalid_instruction() {
        assert_eq!(
            CounterInstruction::unpack(&[]),
            Err(CounterError::InvalidInstruction)
        );
        assert_eq!(
            CounterInstruction::unpack(&[2]),
            Err(CounterError::InvalidInstruction)
        );
        assert_eq!(
            CounterInstruction::unpack(&[0, 0]),
            Err(CounterError::InvalidInstruction)
        );
    }
}
