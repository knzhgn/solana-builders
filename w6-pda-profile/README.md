# On-chain профиль на PDA (неделя 6)

Anchor-программа `profile`: у каждого кошелька один профиль `UserProfile` (username до 32 байт, bio до 160 байт, время создания и обновления, bump). Аккаунт профиля — **PDA** с seeds `["profile", authority]`. Инструкции: `create_profile`, `update_profile` (username и bio), `delete_profile` (rent возвращается владельцу).

- Сеть: **devnet**
- Program ID: `95m1n2nyxVMRKSNtJR9jCTGeLF7KasESR7zeyseH9hHu` — [Explorer](https://explorer.solana.com/address/95m1n2nyxVMRKSNtJR9jCTGeLF7KasESR7zeyseH9hHu?cluster=devnet)
- Signature деплоя: `61j7pCzUsfHerzvYMQdEv2Lt5RXJEFPixz3ZoErK6JMJczpojbL7RmYQXJ1e259gnaYksRfDc2dpUJ58K69qVRA3` — [Explorer](https://explorer.solana.com/tx/61j7pCzUsfHerzvYMQdEv2Lt5RXJEFPixz3ZoErK6JMJczpojbL7RmYQXJ1e259gnaYksRfDc2dpUJ58K69qVRA3?cluster=devnet)
- Upgrade authority: `2KWU4aTFFwChdU2YxYj1LD8b6eqC7Kum7hHsKDsaFXv9`
- Размер `.so`: 160 360 байт; IDL в сеть не загружался (`--no-idl`)
- PDA профиля: `CS9tZotJGKo9zP1u4aXXtneNr42Ht3e1sxzrHgbYkchk` — [Explorer](https://explorer.solana.com/address/CS9tZotJGKo9zP1u4aXXtneNr42Ht3e1sxzrHgbYkchk?cluster=devnet); seeds `["profile", authority]`, bump `255`
- Потрачено SOL: деплой 0.817186, клиент 0.001966; всего 0.819152. Остаток на кошельке 0.304462 SOL.

## Как работает PDA простыми словами

PDA (Program Derived Address) — адрес, который вычисляется из «семян» (seeds) и Program ID: `hash(seeds + bump + program_id)`. Один и тот же кошелёк всегда даёт один и тот же адрес, поэтому профиль находится без запросов к сети и без таблицы соответствий. Bump — число, которое подбирается от 255 вниз, пока адрес не окажется вне кривой ed25519; первое подходящее называется canonical bump и сохраняется в аккаунте. У такого адреса нет приватного ключа, поэтому подписать за него может только программа (`invoke_signed`): Anchor делает это при `init`. Кошелёк с другим адресом получит другой PDA, поэтому чужой профиль подменить нельзя: seeds не сойдутся.

## Структура

```
programs/profile/src/
  lib.rs                      declare_id!, #[program] с 3 инструкциями
  state.rs                    UserProfile (InitSpace), PROFILE_SEED, лимиты
  error.rs                    ProfileError (6000–6003)
  instructions/               create_profile.rs, update_profile.rs, delete_profile.rs
tests/profile.ts              8 TypeScript-тестов (anchor test); Rust unit-тесты в state.rs и instructions.rs
client/client.ts              клиент; client/idl/profile.json — копия target/idl/profile.json
Anchor.toml  Cargo.toml  package.json  tsconfig.json
```

## Constraints

| Constraint | Что проверяет |
|---|---|
| `init, payer, space` | создаёт аккаунт 8 + 249 = 257 байт, rent-exempt, владелец — программа |
| `seeds = ["profile", authority]`, `bump` / `bump = profile.bump` | адрес аккаунта совпадает с PDA этого authority |
| `has_one = authority` | `profile.authority == authority.key()` |
| `Signer` | authority подписал транзакцию |
| `close = authority` | данные обнуляются, rent возвращается владельцу |

## Команды

```bash
npm install
anchor build                                                   # target/deploy/profile.so, target/idl
anchor test --provider.cluster localnet --validator legacy     # 8 тестов на локальном валидаторе, бесплатно
cargo test                                                     # Rust unit-тесты
anchor deploy --provider.cluster devnet --no-idl -- --max-len 160360   # ~0.82 SOL
solana program show 95m1n2nyxVMRKSNtJR9jCTGeLF7KasESR7zeyseH9hHu
CLUSTER=devnet npm run client                                  # create_profile + update_profile (профиль не удаляется)
```

Нестандартные флаги компилятора не используются (`.cargo/config.toml` нет), сборка обычная.

## Результат запуска клиента на devnet

Вывод и полные signatures — в `run-devnet.log`. Итог: `username = w6_builder_pda`, `bump = 255`.

| Действие | Signature | Результат |
|---|---|---|
| create_profile | [5pfhd4cw…84eKHF](https://explorer.solana.com/tx/5pfhd4cwKLdidTCpT6ZYmnQftfLH9tttz2fyWFZQZzRKyjH6Wi5BGnTebYcGjzQdNkazzB6ngkY4fZQEdT84eKHF?cluster=devnet) | username=w6_builder |
| update_profile | [4vz9Fhp8…vruUHJ](https://explorer.solana.com/tx/4vz9Fhp89unbK9BgezBsHEqXCs2MeH6HbURB1u8wG9XEqkjHSc228jYcYLY2yeWZtWWsCkS6toGS9Kz13WvruUHJ?cluster=devnet) | username=w6_builder_pda |
