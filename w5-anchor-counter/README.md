# Counter на Anchor (неделя 5)

Программа-счётчик на Anchor 1.x: аккаунт `Counter { authority: Pubkey, count: u64 }`, инструкции `initialize`, `increment`, `decrement`, `reset`. Менять счётчик может только `authority`.

> **Статус: протестировано локально, в devnet НЕ задеплоено.** Деплой стоит ≈ 0.597 SOL (на `main.json` 0.804 SOL, нужно оставить на неделю 6). Program ID зарезервирован, ключ в резервной копии вне проекта; деплой выполняется одной командой, см. ниже.

- Program ID (в `declare_id!` и `Anchor.toml`): `H6sSYCfPuUZVSrZsoNnvRhgZf9EvFYsCKtV4wWJoMn6U`
- Размер `.so`: 117 168 байт
- Стоимость деплоя в devnet: 596 092 280 lamports за ProgramData (ответ RPC `getMinimumBalanceForRentExemption`) + ≈ 0.001 SOL program account и комиссии

## Структура

```
programs/counter/src/
  lib.rs                   declare_id!, #[program] с 4 инструкциями
  state.rs                 #[account] #[derive(InitSpace)] struct Counter
  error.rs                 #[error_code] CounterError (CounterUnderflow = 6000, CounterOverflow = 6001)
  instructions/initialize.rs  Initialize (init, payer, space)
  instructions/update.rs      Update (mut, has_one = authority) + increment / decrement / reset
tests/counter.ts           TypeScript-тесты (mocha + chai), 7 тестов
client/client.ts           клиент; client/idl/counter.json — копия target/idl/counter.json
Anchor.toml  Cargo.toml  .cargo/config.toml  package.json  tsconfig.json
```

## Constraints

| Constraint | Что проверяет |
|---|---|
| `init, payer = authority, space = 8 + InitSpace` | создаёт аккаунт через System Program, rent-exempt, владелец — программа |
| `#[account(mut)]` | аккаунт изменяемый (writable) |
| `has_one = authority` | `counter.authority == authority.key()` |
| `Signer` | аккаунт подписал транзакцию |
| `Account<Counter>` | владелец == program id и discriminator совпадает |

## Команды

```bash
npm install
anchor build                                          # target/deploy/counter.so, target/idl, target/types
anchor test --provider.cluster localnet --validator legacy   # 7 тестов на локальном валидаторе, бесплатно
```

Клиент на локальном валидаторе:

```bash
solana-test-validator --reset --mint <PUBKEY_КОШЕЛЬКА> \
  --bpf-program H6sSYCfPuUZVSrZsoNnvRhgZf9EvFYsCKtV4wWJoMn6U target/deploy/counter.so
CLUSTER=localnet npm run client
```

Деплой в devnet (один раз; ~0.6 SOL):

```bash
anchor deploy --provider.cluster devnet -- --max-len 117168
solana program show H6sSYCfPuUZVSrZsoNnvRhgZf9EvFYsCKtV4wWJoMn6U
CLUSTER=devnet npm run client
```

IDL в сеть не загружается (`anchor idl init` стоит SOL): клиент берёт его из `client/idl/counter.json`.

## Уменьшение размера .so

`opt-level = "s"`, `lto = "fat"`, `codegen-units = 1`, `strip`, `panic = "abort"`, `overflow-checks = true` (`Cargo.toml`), feature `no-log-ix-name`, а в `.cargo/config.toml` флаги `-Zlocation-detail=none -Zfmt-debug=none` (через `RUSTC_BOOTSTRAP=1`). Итог: 140 784 → 117 168 байт. Размер почти весь определяется самим Anchor (обработка ошибок, форматирование).

## Результат локального прогона клиента (localnet)

Счётчик `J6cMdDafTp19kecxjkeeDEGG15oSp5x5xkzhjwTnGkNW`, итоговый `count = 2`, authority `2KWU4aTFFwChdU2YxYj1LD8b6eqC7Kum7hHsKDsaFXv9`. Подписи — в `run-localnet.log` (это localnet, в Explorer devnet их нет).

| Действие | Signature | count |
|---|---|---|
| initialize | `4dwN9rxn…hg1TLa` | 0 |
| increment | `5NUK75ju…cPYdZ` | 1 |
| increment | `7shssPZS…pXRs` | 2 |
| increment | `3x73C6iX…WdxEZ` | 3 |
| decrement | `H5CeC1KJ…GpxBh` | 2 |
