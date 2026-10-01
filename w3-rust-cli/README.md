# sol-tools — неделя 3, Rust mini-project

CLI-утилита на Rust с мини-инструментами для Solana: конвертация SOL ↔ lamports, проверка публичного ключа, расчёт базовой комиссии, разбор типа инструкции System Program. Работает локально, без сети.

## Запуск

```bash
cargo build --release
./target/release/sol-tools --help
# или без сборки release:
cargo run -- lamports 1.5
```

Проверки: `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (22 теста).

## Команды

| Команда | Что делает |
|---|---|
| `lamports <SOL>` | SOL → lamports, точно, без float (до 9 знаков после точки) |
| `sol <lamports>` | lamports → SOL |
| `validate <pubkey>` | проверка base58 и длины 32 байта |
| `fee <signatures> [--balance <SOL>] [--payer <pubkey>]` | базовая комиссия 5000 lamports × подписи; с `--balance` — остаток после оплаты |
| `parse-ix <тип>` | тип инструкции System Program → enum, индекс, accounts |

## Структура

```
src/
  main.rs         точка входа: разбор аргументов, вывод, exit code
  cli.rs          enum Command (clap derive)
  commands.rs     обработчики команд -> Result<String, ToolError>
  errors.rs       enum ToolError + impl Display
  units.rs        SOL <-> lamports, базовая комиссия (u64, checked-арифметика)
  pubkey.rs       struct Pubkey([u8; 32]), base58
  instruction.rs  enum InstructionType + FromStr
  wallet.rs       struct Wallet: deposit / withdraw / pay_fee
```

## Примеры

```
$ sol-tools lamports 1.5
1.5 SOL = 1 500 000 000 lamports
$ sol-tools sol 899995000
899 995 000 lamports = 0.899995 SOL (f64: 0.899995)
$ sol-tools validate 6gAYJ35qhM1WznJz5d2kRa3mG5HdC2VwvhQGvoSY1sgF
OK: корректный публичный ключ, длина 32 байта
6gAYJ35qhM1WznJz5d2kRa3mG5HdC2VwvhQGvoSY1sgF
$ sol-tools validate 11111111111111111111111111111111
OK: корректный публичный ключ, длина 32 байта
11111111111111111111111111111111
это System Program
$ sol-tools fee 1
подписей: 1
базовая комиссия: 5 000 lamports = 0.000005 SOL
$ sol-tools fee 3 --balance 0.1 --payer 2KWU4aTFFwChdU2YxYj1LD8b6eqC7Kum7hHsKDsaFXv9
подписей: 3
базовая комиссия: 15 000 lamports = 0.000015 SOL
fee payer 2KWU4aTFFwChdU2YxYj1LD8b6eqC7Kum7hHsKDsaFXv9: 0.1 SOL -> после комиссии 0.099985 SOL
$ sol-tools parse-ix transfer
Transfer (индекс 2 в System Program)
перевести lamports с одного аккаунта на другой
accounts: from (signer, writable); to (writable)
$ sol-tools lamports 1.0000000001
ошибка: слишком точная сумма: 10 знаков после точки, максимум 9 (1 lamport = 0.000000001 SOL)
$ sol-tools validate abc
ошибка: неверная длина публичного ключа: получено байт — 3, нужно 32
$ sol-tools fee 1 --balance 0.000001
ошибка: недостаточно средств: нужно 5000 lamports, доступно 1000
$ sol-tools parse-ix swap
ошибка: неизвестный тип инструкции 'swap'. Доступно: CreateAccount, Assign, Transfer, CreateAccountWithSeed, AdvanceNonceAccount, Allocate
```
