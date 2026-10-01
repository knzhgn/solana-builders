// Клиент для Counter. IDL лежит рядом (client/idl/counter.json, копия target/idl/counter.json).
// Запуск:  CLUSTER=localnet npm run client     (локальный валидатор, бесплатно)
//          CLUSTER=devnet   npm run client     (devnet, нужна задеплоенная программа и SOL на кошельке)
import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { Connection, Keypair, clusterApiUrl } from "@solana/web3.js";
import { readFileSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import idl from "./idl/counter.json";

const CLUSTER = process.env.CLUSTER ?? "devnet";
const RPC = CLUSTER === "localnet" ? "http://127.0.0.1:8899" : clusterApiUrl("devnet");
const WALLET = process.env.WALLET ?? `${homedir()}/solana-builders/w1-wallet/keys/main.json`;

const link = (kind: "tx" | "address", id: string) =>
  CLUSTER === "devnet"
    ? `https://explorer.solana.com/${kind}/${id}?cluster=devnet`
    : `https://explorer.solana.com/${kind}/${id}?cluster=custom&customUrl=${encodeURIComponent(RPC)}`;

async function main() {
  // Секретный ключ читается из файла и нигде не печатается.
  const payer = Keypair.fromSecretKey(Uint8Array.from(JSON.parse(readFileSync(WALLET, "utf8"))));
  const provider = new anchor.AnchorProvider(new Connection(RPC, "confirmed"), new anchor.Wallet(payer), {
    commitment: "confirmed",
    preflightCommitment: "confirmed",
  });
  const program = new Program(idl as anchor.Idl, provider);
  const read = async (k: Keypair) => (program.account as any).counter.fetch(k.publicKey);

  const counter = Keypair.generate();
  const log: { step: string; signature: string; count: string }[] = [];
  const startBalance = await provider.connection.getBalance(payer.publicKey);
  console.log(`cluster: ${CLUSTER}\nprogram: ${program.programId.toBase58()}\npayer:   ${payer.publicKey.toBase58()}\ncounter: ${counter.publicKey.toBase58()}`);

  let sig = await program.methods
    .initialize()
    .accounts({ counter: counter.publicKey, authority: payer.publicKey })
    .signers([counter])
    .rpc();
  log.push({ step: "initialize", signature: sig, count: (await read(counter)).count.toString() });

  for (const step of ["increment", "increment", "increment", "decrement"] as const) {
    sig = await program.methods[step]().accounts({ counter: counter.publicKey, authority: payer.publicKey }).rpc();
    log.push({ step, signature: sig, count: (await read(counter)).count.toString() });
  }

  const state = await read(counter);
  for (const l of log) console.log(`${l.step.padEnd(11)} count=${l.count}  ${link("tx", l.signature)}`);
  console.log(`\nитог: count = ${state.count.toString()}, authority = ${state.authority.toBase58()}`);
  console.log(`счётчик: ${link("address", counter.publicKey.toBase58())}`);
  const spent = startBalance - (await provider.connection.getBalance(payer.publicKey));
  console.log(`потрачено кошельком: ${spent} lamports`);

  writeFileSync(
    `run-${CLUSTER}.log`,
    [
      `cluster: ${CLUSTER}`,
      `program: ${program.programId.toBase58()}`,
      `counter: ${counter.publicKey.toBase58()}`,
      ...log.map((l) => `${l.step}\t${l.signature}\tcount=${l.count}`),
      `final count: ${state.count.toString()}`,
      `authority: ${state.authority.toBase58()}`,
    ].join("\n") + "\n",
  );
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
