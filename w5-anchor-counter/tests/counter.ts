import * as anchor from "@anchor-lang/core";
import { AnchorError, Program } from "@anchor-lang/core";
import { Keypair } from "@solana/web3.js";
import { expect } from "chai";
import { Counter } from "../target/types/counter";

describe("counter", () => {
  // Провайдер берёт кластер и кошелёк из Anchor.toml / переменных, которые ставит `anchor test`.
  // Один уровень подтверждения, чтобы следующий вызов видел аккаунт из предыдущего.
  const env = anchor.AnchorProvider.env();
  const provider = new anchor.AnchorProvider(env.connection, env.wallet, {
    commitment: "confirmed",
    preflightCommitment: "confirmed",
  });
  anchor.setProvider(provider);

  // Program<Counter> собран из IDL (target/idl/counter.json): методы и аккаунты типизированы.
  const program = anchor.workspace.counter as Program<Counter>;
  const authority = provider.wallet.publicKey;

  const create = async () => {
    const counter = Keypair.generate();
    await program.methods
      .initialize()
      .accounts({ counter: counter.publicKey, authority })
      .signers([counter]) // новый аккаунт подписывает своё создание
      .rpc();
    return counter;
  };
  const update = (method: "increment" | "decrement" | "reset", counter: Keypair, signer?: Keypair) => {
    const b = program.methods[method]().accounts({
      counter: counter.publicKey,
      authority: signer ? signer.publicKey : authority,
    });
    return (signer ? b.signers([signer]) : b).rpc();
  };
  const read = (counter: Keypair) => program.account.counter.fetch(counter.publicKey);

  // Ожидаем AnchorError с заданным именем ошибки.
  const expectAnchorError = async (p: Promise<unknown>, code: string) => {
    try {
      await p;
    } catch (e) {
      expect(e).to.be.instanceOf(AnchorError);
      expect((e as AnchorError).error.errorCode.code).to.equal(code);
      return;
    }
    expect.fail(`ожидали ошибку ${code}`);
  };

  it("initialize: count = 0, authority = подписант", async () => {
    const counter = await create();
    const state = await read(counter);
    expect(state.count.toNumber()).to.equal(0);
    expect(state.authority.toBase58()).to.equal(authority.toBase58());
  });

  it("increment ×2 → decrement → reset", async () => {
    const counter = await create();
    await update("increment", counter);
    await update("increment", counter);
    expect((await read(counter)).count.toNumber()).to.equal(2);
    await update("decrement", counter);
    expect((await read(counter)).count.toNumber()).to.equal(1);
    await update("reset", counter);
    expect((await read(counter)).count.toNumber()).to.equal(0);
  });

  it("decrement ниже нуля → CounterUnderflow (6000)", async () => {
    const counter = await create();
    try {
      await update("decrement", counter);
      expect.fail("ожидали CounterUnderflow");
    } catch (e) {
      expect(e).to.be.instanceOf(AnchorError);
      const err = (e as AnchorError).error;
      expect(err.errorCode.code).to.equal("CounterUnderflow");
      expect(err.errorCode.number).to.equal(6000);
    }
    expect((await read(counter)).count.toNumber()).to.equal(0);
  });

  it("increment чужим signer → ConstraintHasOne", async () => {
    const counter = await create();
    const stranger = Keypair.generate();
    await expectAnchorError(update("increment", counter, stranger), "ConstraintHasOne");
    expect((await read(counter)).count.toNumber()).to.equal(0);
  });

  it("reset и decrement чужим signer тоже запрещены", async () => {
    const counter = await create();
    await update("increment", counter);
    const stranger = Keypair.generate();
    await expectAnchorError(update("reset", counter, stranger), "ConstraintHasOne");
    await expectAnchorError(update("decrement", counter, stranger), "ConstraintHasOne");
    expect((await read(counter)).count.toNumber()).to.equal(1);
  });

  it("authority не подписал → ошибка подписи", async () => {
    const counter = await create();
    const stranger = Keypair.generate();
    // authority указан, но подписи его ключа нет: клиент не соберёт транзакцию
    let failed = false;
    try {
      await program.methods
        .increment()
        .accounts({ counter: counter.publicKey, authority: stranger.publicKey })
        .rpc();
    } catch {
      failed = true;
    }
    expect(failed).to.equal(true);
    expect((await read(counter)).count.toNumber()).to.equal(0);
  });

  it("повторный initialize того же аккаунта невозможен", async () => {
    const counter = await create();
    let failed = false;
    try {
      await program.methods
        .initialize()
        .accounts({ counter: counter.publicKey, authority })
        .signers([counter])
        .rpc();
    } catch {
      failed = true; // аккаунт уже существует: System Program не даёт создать его снова
    }
    expect(failed).to.equal(true);
  });
});
