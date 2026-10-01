//! Counter — первая нативная Solana-программа (без Anchor).
//!
//! Program ID не зашит в код: рантайм передаёт его в entrypoint,
//! и все проверки владельца сравнивают с этим значением.

pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

// В тестах и при подключении как библиотеки точка входа не нужна.
#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;
