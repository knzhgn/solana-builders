# Неделя 1 — Вход в Web3: первая транзакция в Solana (Devnet)

## Что сделано

- Установлен и настроен Solana CLI 3.1.10, сеть — devnet.
- Создан кошелёк (keypair) `keys/main.json`, он назначен кошельком CLI по умолчанию. Папка `keys/` исключена из git.
- Airdrop упёрся в rate limit, поэтому кошелёк пополнен на 1 SOL переводом с Phantom (devnet).
- Отправлена первая транзакция: 0.1 SOL с CLI-кошелька на Phantom.
- Транзакция разобрана через `solana confirm -v` и JSON-RPC `getTransaction` (ответ в `tx.json`) и проверена в Explorer и Solscan.

## Адреса кошельков (только публичные)

| Кошелёк | Адрес |
|---|---|
| CLI-кошелёк (`keys/main.json`) | `2KWU4aTFFwChdU2YxYj1LD8b6eqC7Kum7hHsKDsaFXv9` |
| Phantom | `6gAYJ35qhM1WznJz5d2kRa3mG5HdC2VwvhQGvoSY1sgF` |

- Phantom в Solscan: https://solscan.io/account/6gAYJ35qhM1WznJz5d2kRa3mG5HdC2VwvhQGvoSY1sgF?cluster=devnet
- CLI-кошелёк в Solscan: https://solscan.io/account/2KWU4aTFFwChdU2YxYj1LD8b6eqC7Kum7hHsKDsaFXv9?cluster=devnet

## Использованные команды

```bash
solana --version
mkdir -p ~/solana-builders/w1-wallet/keys
cd ~/solana-builders && git init            # + .gitignore: keys/, **/keys/*.json, target/, node_modules/
solana config set --url devnet
solana-keygen new --silent --no-bip39-passphrase --outfile ~/solana-builders/w1-wallet/keys/main.json
solana config set --keypair ~/solana-builders/w1-wallet/keys/main.json
solana address
solana airdrop 2                            # rate limit, пополнение с Phantom (tx 5rB6J6dn…)
solana balance
solana transfer 6gAYJ35qhM1WznJz5d2kRa3mG5HdC2VwvhQGvoSY1sgF 0.1 --allow-unfunded-recipient
solana confirm -v <SIGNATURE>
curl -s https://api.devnet.solana.com -H 'Content-Type: application/json' -d '{
  "jsonrpc":"2.0","id":1,"method":"getTransaction",
  "params":["<SIGNATURE>",{"encoding":"jsonParsed","maxSupportedTransactionVersion":0}]
}' | python3 -m json.tool > tx.json
```

## Транзакция

Signature:

```
nzorVo4VUwfuLdrTgfgcmsnuMiL92BBvTxZrkfYiK9Dsuz2orWoXxL5WoS7EEM9D2gmTYVQ8kwZcjc9Q1C1sMY9
```

- Explorer: https://explorer.solana.com/tx/nzorVo4VUwfuLdrTgfgcmsnuMiL92BBvTxZrkfYiK9Dsuz2orWoXxL5WoS7EEM9D2gmTYVQ8kwZcjc9Q1C1sMY9?cluster=devnet
- Solscan: https://solscan.io/tx/nzorVo4VUwfuLdrTgfgcmsnuMiL92BBvTxZrkfYiK9Dsuz2orWoXxL5WoS7EEM9D2gmTYVQ8kwZcjc9Q1C1sMY9?cluster=devnet

Пополнение CLI-кошелька с Phantom (1 SOL): https://solscan.io/tx/5rB6J6dnaAvcztVzU7hJ9VkrqsMpbzp2dUcWfEruBN5C9XmCuuGddDAWjzPA6yvh9kXANmLbvQmGaMNaZvLDxMMt?cluster=devnet

## Разбор транзакции

| Поле | Значение |
|---|---|
| Статус | Ok (Finalized) |
| Fee payer | `2KWU…aFXv9` (CLI-кошелёк), он же единственный signer |
| Signers | 1 подпись: `2KWU…aFXv9` |
| Инструкция | одна, `transfer` |
| Программа | System Program `11111111111111111111111111111111` |
| Отправитель (source) | `2KWU4aTFFwChdU2YxYj1LD8b6eqC7Kum7hHsKDsaFXv9` |
| Получатель (destination) | `6gAYJ35qhM1WznJz5d2kRa3mG5HdC2VwvhQGvoSY1sgF` (Phantom) |
| Сумма | 100 000 000 lamports = 0.1 SOL |
| Комиссия | 5 000 lamports = 0.000005 SOL |
| Compute units | 150 |
| Slot | 506 290 625 |
| Время | 2026-10-01 13:39:37 UTC (blockTime 1790861977) |
| Версия | legacy |

Балансы (pre → post), в lamports:

| Адрес | До | После | Изменение |
|---|---|---|---|
| Отправитель `2KWU…aFXv9` | 1 000 000 000 | 899 995 000 | −100 005 000 (0.1 SOL + 0.000005 комиссия) |
| Получатель `6gAY…1sgF` | 3 999 920 066 | 4 099 920 066 | +100 000 000 (ровно 0.1 SOL) |

Простыми словами: CLI-кошелёк подписал своим приватным ключом сообщение «System Program, переведи 0.1 SOL с моего счёта на счёт 6gAY…». Отправитель платит и сумму перевода, и комиссию сети: 5 000 lamports за одну подпись. Получатель получает ровно 0.1 SOL. System Program — встроенная программа Solana, которая владеет обычными кошельками и умеет переводить SOL. В транзакции три аккаунта: отправитель (signer + writable), получатель (writable, баланс меняется) и сама программа (только чтение, исполняемая). 1 SOL = 1 000 000 000 lamports.

### Почему адрес Phantom виден на devnet, хотя Phantom не переключали

Адрес — это просто публичный ключ, производный от приватного ключа. Он один и тот же во всех сетях Solana: mainnet, devnet, testnet. Сети отличаются только набором валидаторов и своим реестром балансов. Переключатель сети в Phantom решает лишь то, к какому RPC приложение обращается, чтобы показать баланс. Отправить SOL на адрес в devnet можно, даже если кошелёк ни разу туда не «заходил»: аккаунт создаётся при первом поступлении средств (отсюда флаг `--allow-unfunded-recipient`). Поэтому в Explorer/Solscan с `?cluster=devnet` этот адрес виден с devnet-балансом, а в самом Phantom эти SOL появятся только после включения Devnet (Testnet Mode).

## Что я понял

Кошелёк в Solana — это пара ключей: публичный ключ служит адресом счёта, который можно давать всем, а приватный ключ (или seed phrase, из которой он выводится) даёт полное право распоряжаться средствами, поэтому его нельзя никому показывать и коммитить в git. Транзакция — подписанное приватным ключом сообщение со списком инструкций для программ (здесь одна инструкция `transfer` для System Program); сеть проверяет подпись, списывает комиссию с fee payer и навсегда записывает результат в блок (slot). Devnet — тестовая сеть Solana с бесплатными SOL, работающая так же, как mainnet, поэтому на ней можно безопасно учиться, не рискуя реальными деньгами.
