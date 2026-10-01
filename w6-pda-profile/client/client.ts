// Клиент для программы profile. IDL лежит рядом (client/idl/profile.json = копия target/idl/profile.json),
// из сети он не читается.
//   CLUSTER=devnet npm run client     (по умолчанию)
//   CLUSTER=localnet npm run client   (локальный валидатор)
// Профиль в конце НЕ удаляется: он остаётся в devnet (delete проверен в тестах).
import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { Connection, Keypair, PublicKey, clusterApiUrl } from "@solana/web3.js";
import { readFileSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import idl from "./idl/profile.json";

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
  const fetchProfile = async (pda: PublicKey) => (program.account as any).userProfile.fetch(pda);
  const show = (tag: string, p: any) =>
    console.log(`${tag}: username=${JSON.stringify(p.username)} bio=${JSON.stringify(p.bio)} createdAt=${p.createdAt} updatedAt=${p.updatedAt} bump=${p.bump}`);

  // PDA = hash(seeds + program_id + bump), bump ищется от 255 вниз до адреса вне кривой ed25519.
  const [pda, bump] = PublicKey.findProgramAddressSync(
    [Buffer.from("profile"), payer.publicKey.toBuffer()],
    program.programId,
  );
  console.log(`cluster:   ${CLUSTER}\nprogram:   ${program.programId.toBase58()}\nauthority: ${payer.publicKey.toBase58()}`);
  console.log(`seeds:     ["profile", authority]\nPDA:       ${pda.toBase58()}\nbump:      ${bump}`);

  const startBalance = await provider.connection.getBalance(payer.publicKey);
  const log: { step: string; signature: string; result: string }[] = [];
  const accounts = { profile: pda, authority: payer.publicKey };

  let sig = await program.methods.createProfile("w6_builder", "Первый PDA-профиль в devnet").accountsPartial(accounts).rpc();
  let p = await fetchProfile(pda);
  show("после create", p);
  log.push({ step: "create_profile", signature: sig, result: `username=${p.username}` });

  sig = await program.methods.updateProfile("w6_builder_pda", "Обновлено: username и bio меняются одной инструкцией").accountsPartial(accounts).rpc();
  p = await fetchProfile(pda);
  show("после update", p);
  log.push({ step: "update_profile", signature: sig, result: `username=${p.username}` });

  const spent = startBalance - (await provider.connection.getBalance(payer.publicKey));
  console.log("");
  for (const l of log) console.log(`${l.step.padEnd(15)} ${l.result.padEnd(24)} ${link("tx", l.signature)}`);
  console.log(`профиль: ${link("address", pda.toBase58())}\nпотрачено кошельком: ${spent} lamports (rent профиля + комиссии)`);

  writeFileSync(
    `run-${CLUSTER}.log`,
    [
      `cluster: ${CLUSTER}`,
      `program: ${program.programId.toBase58()}`,
      `authority: ${payer.publicKey.toBase58()}`,
      `seeds: ["profile", authority]`,
      `PDA: ${pda.toBase58()}`,
      `bump: ${bump}`,
      ...log.map((l) => `${l.step}\t${l.signature}\t${l.result}`),
      `final: username=${p.username} bio=${p.bio} bump=${p.bump}`,
      `spent lamports: ${spent}`,
    ].join("\n") + "\n",
  );
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
