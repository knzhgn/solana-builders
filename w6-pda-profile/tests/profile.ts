import * as anchor from "@anchor-lang/core";
import { AnchorError, Program } from "@anchor-lang/core";
import { Keypair, LAMPORTS_PER_SOL, PublicKey, SystemProgram, Transaction } from "@solana/web3.js";
import { expect } from "chai";
import { Profile } from "../target/types/profile";

describe("profile (PDA)", () => {
  const env = anchor.AnchorProvider.env();
  const provider = new anchor.AnchorProvider(env.connection, env.wallet, {
    commitment: "confirmed",
    preflightCommitment: "confirmed",
  });
  anchor.setProvider(provider);
  const program = anchor.workspace.profile as Program<Profile>;
  const conn = provider.connection;

  // PDA профиля: ровно те же seeds, что в программе.
  const pdaOf = (owner: PublicKey) =>
    PublicKey.findProgramAddressSync([Buffer.from("profile"), owner.toBuffer()], program.programId);

  // Каждый тест работает с новым кошельком: на один кошелёк приходится один профиль.
  // Кошелёк получает немного SOL на rent профиля (платит provider.wallet).
  async function newUser(): Promise<Keypair> {
    const user = Keypair.generate();
    await provider.sendAndConfirm(
      new Transaction().add(
        SystemProgram.transfer({ fromPubkey: provider.wallet.publicKey, toPubkey: user.publicKey, lamports: 0.05 * LAMPORTS_PER_SOL }),
      ),
    );
    return user;
  }
  const create = (user: Keypair, username: string, bio: string) =>
    program.methods
      .createProfile(username, bio)
      .accountsPartial({ profile: pdaOf(user.publicKey)[0], authority: user.publicKey })
      .signers([user])
      .rpc();
  const update = (user: Keypair, profile: PublicKey, username: string, bio: string) =>
    program.methods.updateProfile(username, bio).accountsPartial({ profile, authority: user.publicKey }).signers([user]).rpc();
  const remove = (user: Keypair) =>
    program.methods
      .deleteProfile()
      .accountsPartial({ profile: pdaOf(user.publicKey)[0], authority: user.publicKey })
      .signers([user])
      .rpc();

  // Ожидаем AnchorError с одним из указанных кодов; возвращаем фактический код.
  async function expectCode(p: Promise<unknown>, ...codes: string[]): Promise<string> {
    try {
      await p;
    } catch (e) {
      expect(e).to.be.instanceOf(AnchorError);
      const code = (e as AnchorError).error.errorCode.code;
      expect(codes).to.include(code);
      return code;
    }
    expect.fail(`ожидали ошибку ${codes.join("/")}`);
  }

  it("create + fetch: поля, canonical bump, PDA детерминирован", async () => {
    const user = await newUser();
    await create(user, "valentin", "Solana builder");
    const [pda, bump] = pdaOf(user.publicKey);
    const p = await program.account.userProfile.fetch(pda);
    expect(p.authority.toBase58()).to.equal(user.publicKey.toBase58());
    expect(p.username).to.equal("valentin");
    expect(p.bio).to.equal("Solana builder");
    expect(p.bump).to.equal(bump); // bump в аккаунте совпал с вычисленным на клиенте
    expect(p.createdAt.toNumber()).to.be.greaterThan(0);
    expect(p.updatedAt.toNumber()).to.equal(p.createdAt.toNumber());
    // PDA лежит вне кривой ed25519, у адреса нет приватного ключа
    expect(PublicKey.isOnCurve(pda.toBytes())).to.equal(false);
    // владелец аккаунта — наша программа, размер 8 + 249
    const info = await conn.getAccountInfo(pda);
    expect(info!.owner.toBase58()).to.equal(program.programId.toBase58());
    expect(info!.data.length).to.equal(257);
  });

  it("update: меняются username и bio, authority и created_at остаются", async () => {
    const user = await newUser();
    await create(user, "old_name", "old bio");
    const [pda] = pdaOf(user.publicKey);
    const before = await program.account.userProfile.fetch(pda);
    await update(user, pda, "new_name", "new bio");
    const after = await program.account.userProfile.fetch(pda);
    expect(after.username).to.equal("new_name");
    expect(after.bio).to.equal("new bio");
    expect(after.authority.toBase58()).to.equal(before.authority.toBase58());
    expect(after.createdAt.toNumber()).to.equal(before.createdAt.toNumber());
    expect(after.updatedAt.toNumber()).to.be.gte(before.updatedAt.toNumber());
    expect(after.bump).to.equal(before.bump);
  });

  it("delete: аккаунт закрыт, rent вернулся владельцу", async () => {
    const user = await newUser();
    await create(user, "temp", "to be deleted");
    const [pda] = pdaOf(user.publicKey);
    const rent = (await conn.getAccountInfo(pda))!.lamports;
    const balBefore = await conn.getBalance(user.publicKey);
    await remove(user);
    expect(await conn.getAccountInfo(pda)).to.equal(null);
    // комиссию платит provider.wallet, поэтому у владельца баланс вырос ровно на rent
    expect((await conn.getBalance(user.publicKey)) - balBefore).to.equal(rent);
    // после удаления профиль можно создать заново
    await create(user, "again", "");
    expect((await program.account.userProfile.fetch(pda)).username).to.equal("again");
  });

  it("✗ второй create тем же кошельком падает", async () => {
    const user = await newUser();
    await create(user, "first", "");
    let failed = false;
    try {
      await create(user, "second", "");
    } catch {
      failed = true; // адрес PDA уже занят: System Program не создаёт аккаунт повторно
    }
    expect(failed).to.equal(true);
    expect((await program.account.userProfile.fetch(pdaOf(user.publicKey)[0])).username).to.equal("first");
  });

  it("✗ update чужим signer: seeds не совпадают", async () => {
    const owner = await newUser();
    await create(owner, "owner", "mine");
    const stranger = await newUser();
    const code = await expectCode(update(stranger, pdaOf(owner.publicKey)[0], "hacked", "x"), "ConstraintSeeds");
    expect(code).to.equal("ConstraintSeeds");
    expect((await program.account.userProfile.fetch(pdaOf(owner.publicKey)[0])).username).to.equal("owner");
  });

  it("✗ подмена PDA: профиль с другими seeds или обычный аккаунт", async () => {
    const a = await newUser();
    const b = await newUser();
    await create(a, "alice", "");
    await create(b, "bob", "");
    // A подписывает, но подставляет PDA профиля B
    const code = await expectCode(update(a, pdaOf(b.publicKey)[0], "alice", "stolen"), "ConstraintSeeds");
    expect(code).to.equal("ConstraintSeeds");
    // подставляем обычный (не PDA, не принадлежащий программе) аккаунт
    const fake = Keypair.generate().publicKey;
    await expectCode(update(a, fake, "alice", "fake"), "AccountNotInitialized", "ConstraintSeeds", "AccountOwnedByWrongProgram");
    // подставляем кошелёк самого A (системный аккаунт) вместо профиля
    await expectCode(update(a, a.publicKey, "alice", "wallet"), "AccountOwnedByWrongProgram", "ConstraintSeeds");
    expect((await program.account.userProfile.fetch(pdaOf(b.publicKey)[0])).username).to.equal("bob");
  });

  it("✗ чужой signer не может удалить профиль", async () => {
    const owner = await newUser();
    await create(owner, "owner", "");
    const stranger = await newUser();
    await expectCode(
      program.methods
        .deleteProfile()
        .accountsPartial({ profile: pdaOf(owner.publicKey)[0], authority: stranger.publicKey })
        .signers([stranger])
        .rpc(),
      "ConstraintSeeds",
    );
    expect(await conn.getAccountInfo(pdaOf(owner.publicKey)[0])).to.not.equal(null);
  });

  it("✗ валидация: username > 32 байт, пустой username, bio > 160 байт", async () => {
    const user = await newUser();
    await expectCode(create(user, "a".repeat(33), ""), "UsernameTooLong");
    await expectCode(create(user, "я".repeat(17), ""), "UsernameTooLong"); // 17 букв = 34 байта
    await expectCode(create(user, "   ", ""), "EmptyUsername");
    await expectCode(create(user, "ok", "b".repeat(161)), "BioTooLong");
    // граничные значения проходят: 32 байта и 160 байт
    await create(user, "я".repeat(16), "b".repeat(160));
    const [pda] = pdaOf(user.publicKey);
    await expectCode(update(user, pda, "ok", "b".repeat(161)), "BioTooLong");
    await expectCode(update(user, pda, "", "x"), "EmptyUsername");
    expect((await program.account.userProfile.fetch(pda)).username).to.equal("я".repeat(16));
  });
});
