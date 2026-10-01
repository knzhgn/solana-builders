// Клиент для программы Counter в devnet.
// Запуск: npx tsx counter.ts            — создать новый счётчик, Initialize, 3× Increment
//         npx tsx counter.ts <COUNTER>  — только прочитать count существующего счётчика
import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import { deserialize } from "borsh";
import { readFileSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";

const RPC = "https://api.devnet.solana.com";
const PROGRAM_ID = new PublicKey("5iWJKJ8S5mewS8JiMPbhAZjgRrPn53eW44ea8WqfbTfH");
const WALLET = `${homedir()}/solana-builders/w1-wallet/keys/main.json`;

// Тот же формат, что struct Counter в программе (borsh): u64 LE + 32 байта ключа.
const COUNTER_SCHEMA = { struct: { count: "u64", authority: { array: { type: "u8", len: 32 } } } };
const COUNTER_LEN = 40;
const IX = { Initialize: 0, Increment: 1 } as const;

const explorer = (kind: "tx" | "address", id: string) =>
  `https://explorer.solana.com/${kind}/${id}?cluster=devnet`;

function counterIx(tag: number, counter: PublicKey, authority: PublicKey) {
  return new TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: counter, isSigner: false, isWritable: true },
      { pubkey: authority, isSigner: true, isWritable: false },
    ],
    data: Buffer.from([tag]),
  });
}

async function readCounter(conn: Connection, counter: PublicKey) {
  const info = await conn.getAccountInfo(counter, "confirmed");
  if (!info) throw new Error(`аккаунт ${counter} не найден`);
  if (!info.owner.equals(PROGRAM_ID)) throw new Error(`владелец ${info.owner}, а не программа`);
  const s = deserialize(COUNTER_SCHEMA, info.data) as { count: bigint; authority: number[] };
  return { count: s.count, authority: new PublicKey(Uint8Array.from(s.authority)), lamports: info.lamports };
}

async function main() {
  const conn = new Connection(RPC, "confirmed");

  if (process.argv[2]) {
    const c = await readCounter(conn, new PublicKey(process.argv[2]));
    console.log(`count = ${c.count}, authority = ${c.authority.toBase58()}`);
    return;
  }

  // Секретный ключ только читается из файла и никуда не выводится.
  const payer = Keypair.fromSecretKey(Uint8Array.from(JSON.parse(readFileSync(WALLET, "utf8"))));
  const counter = Keypair.generate(); // новый адрес под state; нужен только для подписи createAccount
  const log: { step: string; signature: string; result: string }[] = [];
  const startBalance = await conn.getBalance(payer.publicKey);
  console.log(`payer:   ${payer.publicKey.toBase58()}`);
  console.log(`counter: ${counter.publicKey.toBase58()}`);

  // 1. Аккаунт под state: 40 байт, rent-exempt, владелец — наша программа.
  const rent = await conn.getMinimumBalanceForRentExemption(COUNTER_LEN);
  const create = SystemProgram.createAccount({
    fromPubkey: payer.publicKey,
    newAccountPubkey: counter.publicKey,
    lamports: rent,
    space: COUNTER_LEN,
    programId: PROGRAM_ID,
  });
  let sig = await sendAndConfirmTransaction(conn, new Transaction().add(create), [payer, counter]);
  log.push({ step: "CreateAccount (System Program)", signature: sig, result: `40 байт, rent ${rent} lamports` });

  // 2. Initialize
  sig = await sendAndConfirmTransaction(
    conn,
    new Transaction().add(counterIx(IX.Initialize, counter.publicKey, payer.publicKey)),
    [payer],
  );
  log.push({ step: "Initialize", signature: sig, result: `count = ${(await readCounter(conn, counter.publicKey)).count}` });

  // 3. Increment ×3
  for (let i = 0; i < 3; i++) {
    sig = await sendAndConfirmTransaction(
      conn,
      new Transaction().add(counterIx(IX.Increment, counter.publicKey, payer.publicKey)),
      [payer],
    );
    log.push({ step: `Increment #${i + 1}`, signature: sig, result: `count = ${(await readCounter(conn, counter.publicKey)).count}` });
  }

  const final = await readCounter(conn, counter.publicKey);
  const spent = startBalance - (await conn.getBalance(payer.publicKey));
  for (const l of log) console.log(`${l.step.padEnd(32)} ${l.result.padEnd(32)} ${explorer("tx", l.signature)}`);
  console.log(`\nитог: count = ${final.count}, authority = ${final.authority.toBase58()}`);
  console.log(`клиент потратил: ${spent} lamports (rent счётчика ${rent} + комиссии)`);

  writeFileSync(
    "run.log",
    [
      `program: ${PROGRAM_ID.toBase58()}`,
      `counter: ${counter.publicKey.toBase58()}`,
      ...log.map((l) => `${l.step}\t${l.signature}\t${l.result}`),
      `final count: ${final.count}`,
      `client spent lamports: ${spent}`,
    ].join("\n") + "\n",
  );
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
