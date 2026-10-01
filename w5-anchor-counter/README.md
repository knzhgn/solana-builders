# Counter на Anchor (неделя 5)

Программа-счётчик на Anchor 1.x: аккаунт `Counter { authority: Pubkey, count: u64 }`, инструкции `initialize`, `increment`, `decrement`, `reset`. Менять счётчик может только `authority`.

> **Статус: задеплоено в devnet, клиент отработал на devnet.**

- Сеть: **devnet**
- Program ID: `H6sSYCfPuUZVSrZsoNnvRhgZf9EvFYsCKtV4wWJoMn6U` — [Explorer](https://explorer.solana.com/address/H6sSYCfPuUZVSrZsoNnvRhgZf9EvFYsCKtV4wWJoMn6U?cluster=devnet)
- Signature деплоя: `5cu4qsPCk5sAp4Xcvh1bcjCMd3ipgV7LZWqmV19nrAXBeqx9c2bancT8Kpt1eMqh4tYLaRF92YseVceGbp41Caqg` — [Explorer](https://explorer.solana.com/tx/5cu4qsPCk5sAp4Xcvh1bcjCMd3ipgV7LZWqmV19nrAXBeqx9c2bancT8Kpt1eMqh4tYLaRF92YseVceGbp41Caqg?cluster=devnet)
- Upgrade authority: `2KWU4aTFFwChdU2YxYj1LD8b6eqC7Kum7hHsKDsaFXv9`
- Размер `.so`: 117 168 байт
- Адрес Counter: `A8hchhjfBmc3f6RD8nGhAiG9nFgKwTbybh1JQ2WPkfvL` — [Explorer](https://explorer.solana.com/address/A8hchhjfBmc3f6RD8nGhAiG9nFgKwTbybh1JQ2WPkfvL?cluster=devnet), итоговый `count = 2`
- Потрачено SOL: деплой 0.605760 (ProgramData 0.596092 + program account 0.001141 + IDL-аккаунт 0.008199 + комиссии), клиент 0.000924; всего 0.606684. Остаток на кошельке 0.197576 SOL.
- IDL: `anchor deploy` в Anchor 1.x по умолчанию загружает IDL в сеть (metadata-аккаунт `4HtkQadYoUy4nJL7ENuBm1FoXW483ykmYTA2KibiZYEa`, 0.0082 SOL, возвращается при закрытии). Отключается флагом `--no-idl`.

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
anchor deploy --provider.cluster devnet --no-idl -- --max-len 117168
solana program show H6sSYCfPuUZVSrZsoNnvRhgZf9EvFYsCKtV4wWJoMn6U
CLUSTER=devnet npm run client
```

IDL в сеть не загружается (`anchor idl init` стоит SOL): клиент берёт его из `client/idl/counter.json`.

## Воспроизводимость сборки

Проверено: распаковка архива в чистую папку и `anchor build` дают тот же `counter.so` (117 168 байт, совпадает sha256 с задеплоенным), путь кошелька в `Anchor.toml` для сборки не нужен.
Флаги `-Zlocation-detail` и `-Zfmt-debug` принимает только rustc из platform-tools, который ставится вместе с Solana CLI 3.1.x (`RUSTC_BOOTSTRAP=1` включает их на стабильной сборке). Если на другой версии тулчейна сборка из-за них падает, удалите `.cargo/config.toml`: программа будет работать так же, но `.so` вырастет до 135 896 байт (+16%), а rent за деплой — на ≈ 0.095 SOL.

## Уменьшение размера .so

`opt-level = "s"`, `lto = "fat"`, `codegen-units = 1`, `strip`, `panic = "abort"`, `overflow-checks = true` (`Cargo.toml`), feature `no-log-ix-name`, а в `.cargo/config.toml` флаги `-Zlocation-detail=none -Zfmt-debug=none` (через `RUSTC_BOOTSTRAP=1`). Итог: 140 784 → 117 168 байт. Размер почти весь определяется самим Anchor (обработка ошибок, форматирование).

## Результат запуска клиента на devnet

`CLUSTER=devnet npm run client`, вывод и полные signatures — в `run-devnet.log`. Итоговый `count = 2`, authority `2KWU4aTFFwChdU2YxYj1LD8b6eqC7Kum7hHsKDsaFXv9`.

| Действие | Signature | count |
|---|---|---|
| initialize | [nAHSLLHh…nv4EQM](https://explorer.solana.com/tx/nAHSLLHhaxZwbVgnhSdEqBThDpAC2oW1WAhJFbXuVVCHtB93B9RGEnKs67k3x8Xtrc6mHSkLtxuJNz3LFnv4EQM?cluster=devnet) | count = 0 |
| increment | [3hzEysCv…FYFaED](https://explorer.solana.com/tx/3hzEysCvZteAb3q3d2jUSxNWKC2LMmgsskiaotvzPdGMem6GzUXRmEovQRzJVDMWZtS6LAxK6XTzPL3xqXFYFaED?cluster=devnet) | count = 1 |
| increment | [2RjRkq42…cuZuUx](https://explorer.solana.com/tx/2RjRkq42QPHp3rvuLcsnw8bCjGPiW1WmA3n9FRjgZpMesrm6XVvriuL1bf6VJ9vbwpvGigM9ZzRhzEd4urcuZuUx?cluster=devnet) | count = 2 |
| increment | [3xGFpY9b…fsgEeZ](https://explorer.solana.com/tx/3xGFpY9bTRRUqXJmS8p4kQkiraBfq9TMEozfZMm9vtgPs2QHMrt8d92bjo8WnZGCqEHf6mTANuDs5LRnXGfsgEeZ?cluster=devnet) | count = 3 |
| decrement | [5hh4W559…SEqwvy](https://explorer.solana.com/tx/5hh4W559qUi9PbpX75ibkmETFFH4UN92FWqT5vXLzoruytDfiH2hd3YeCP7DjjnGZMnZxFRkc4Ks6i9Ht8SEqwvy?cluster=devnet) | count = 2 |

Ранее клиент проверялся на локальном валидаторе (`run-localnet.log`).
