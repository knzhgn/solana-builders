# Counter — первая Solana Program (devnet, нативная, без Anchor)

Программа хранит в аккаунте счётчик `struct Counter { count: u64, authority: Pubkey }` (40 байт, формат borsh).
Инструкции: `Initialize` (count = 0, authority = signer) и `Increment` (count += 1, только для authority).

- Сеть: **devnet**
- Program ID: `5iWJKJ8S5mewS8JiMPbhAZjgRrPn53eW44ea8WqfbTfH` — [Explorer](https://explorer.solana.com/address/5iWJKJ8S5mewS8JiMPbhAZjgRrPn53eW44ea8WqfbTfH?cluster=devnet)
- Размер .so: 18 312 байт; upgrade authority: `2KWU4aTFFwChdU2YxYj1LD8b6eqC7Kum7hHsKDsaFXv9`
- Signature деплоя: `2HNQpFMQxQxAj4BcrtL8DcqVtJMmaqq6zwMRuL89oJj3PfNnvioVqq3WVmPsiDgb5Mm8UeAih8t31pPqqgQYbVfY` — [Explorer](https://explorer.solana.com/tx/2HNQpFMQxQxAj4BcrtL8DcqVtJMmaqq6zwMRuL89oJj3PfNnvioVqq3WVmPsiDgb5Mm8UeAih8t31pPqqgQYbVfY?cluster=devnet)

## Структура

```
src/lib.rs          модули, no-entrypoint feature
src/entrypoint.rs   точка входа (entrypoint_no_alloc!) и panic-handler
src/instruction.rs  enum CounterInstruction, unpack/pack
src/state.rs        struct Counter, unpack/pack_into (borsh-совместимый формат)
src/error.rs        enum CounterError + impl From<CounterError> for ProgramError
src/processor.rs    Initialize / Increment и проверки
tests/counter.rs    solana-program-test (настоящий .so)
client/counter.ts   TypeScript-клиент (@solana/web3.js)
```

## Проверки в программе

владелец счётчика == program_id · счётчик writable · authority подписал · размер данных == 40 · authority совпадает с записанным · `checked_add` · повторный Initialize запрещён.

## Сборка, тесты, деплой

```bash
cargo build-sbf                                   # target/deploy/counter.so
SBF_OUT_DIR=$PWD/target/deploy cargo test         # 12 тестов: 7 unit + 5 solana-program-test
solana program deploy target/deploy/counter.so --max-len 18312
solana program show 5iWJKJ8S5mewS8JiMPbhAZjgRrPn53eW44ea8WqfbTfH
```

## Клиент

```bash
cd client && npm install
npx tsx counter.ts                 # создаёт счётчик, Initialize, 3× Increment (подписывает w1-wallet/keys/main.json)
npx tsx counter.ts <COUNTER>       # прочитать count
```

Клиент на TypeScript: `@solana/web3.js` проще Rust-клиента (не нужен второй крейт, быстрый запуск), а формат state читается библиотекой borsh.

## Результат запуска клиента

Счётчик: `BvmnsYB2dKWZjSr3tDo3p9c4J9WBtPKNtYiphMv1D5no` ([Explorer](https://explorer.solana.com/address/BvmnsYB2dKWZjSr3tDo3p9c4J9WBtPKNtYiphMv1D5no?cluster=devnet)). Итоговый count = **3**.

| Действие | Signature | Результат |
|---|---|---|
| CreateAccount | [4wHKZGiq…TccdX](https://explorer.solana.com/tx/4wHKZGiqpG8dKzQvHwnwNvpnUnc6bDYZ9X6KYx12dPdtJDhM94DtfZZGkVLh1j2oPgJzHiTEEEuWPG6Q4KfTccdX?cluster=devnet) | 40 байт, rent 853 440 lamports |
| Initialize | [5qMv1HQk…z7zKyg](https://explorer.solana.com/tx/5qMv1HQkbsjGYvMJZhQRZ9CkWYaywFSgrnb3jk9Dvz75S8UfC6Mx6EoRU6m71C8QRorSRigHtNKBvAcfANz7zKyg?cluster=devnet) | count = 0 |
| Increment #1 | [VBurv5GY…KsY5yQ](https://explorer.solana.com/tx/VBurv5GYhsBSt1PvNGoyAqLy1Dnvy57nXxVLx7BnbhnHjYe3Jjy6uk7Kasna8rBu4NZuf7zfmoxnbKCCMKsY5yQ?cluster=devnet) | count = 1 |
| Increment #2 | [27SFoj2J…H1fPrn](https://explorer.solana.com/tx/27SFoj2JN4oThD332Zkg7dg8tbQthS1mRJjfUJpxpHWhxTT2Efnbsu3kmk8cdXBk2Sy6JEerAdqUNch38NH1fPrn?cluster=devnet) | count = 2 |
| Increment #3 | [L8d1B6ER…Jogoup](https://explorer.solana.com/tx/L8d1B6ERPFDY4hNpJjgw5B68eY62NouiUasKNdhL4u2ZJ9WMQ8Bmg7GXFR1BE8EeHEbzKiVeg81Uw3Cg7Jogoup?cluster=devnet) | count = 3 |

Потрачено: деплой 0.09485192 SOL (из них 0.0939 SOL — rent, возвращается при закрытии программы) + клиент 0.00088344 SOL.
Ключ программы `keys/counter-program-keypair.json` в git не попадает (`keys/` в `.gitignore`).
