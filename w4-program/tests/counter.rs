//! Интеграционные тесты: настоящий target/deploy/counter.so в локальном банке solana-program-test.
//! Перед запуском: cargo build-sbf (cargo test-sbf делает это сам).
// ProgramTest в 3.x помечен deprecated как «unstable API» — для тестов это нормально.
#![allow(deprecated)]

use solana_program_test::{BanksClientError, ProgramTest, ProgramTestContext};
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction, InstructionError},
    pubkey::Pubkey,
    rent::Rent,
    signature::{Keypair, Signer},
    transaction::{Transaction, TransactionError},
};
use solana_system_interface::{instruction as system_instruction, program as system_program};

const LEN: usize = 40;
const INITIALIZE: u8 = 0;
const INCREMENT: u8 = 1;

async fn setup() -> (ProgramTestContext, Pubkey) {
    let program_id = Pubkey::new_unique();
    let mut pt = ProgramTest::new("counter", program_id, None);
    pt.prefer_bpf(true);
    (pt.start_with_context().await, program_id)
}

fn ix(
    program_id: Pubkey,
    tag: u8,
    counter: Pubkey,
    authority: Pubkey,
    signer: bool,
) -> Instruction {
    Instruction::new_with_bytes(
        program_id,
        &[tag],
        vec![
            AccountMeta::new(counter, false),
            AccountMeta::new_readonly(authority, signer),
        ],
    )
}

async fn send(
    ctx: &mut ProgramTestContext,
    ixs: &[Instruction],
    extra: &[&Keypair],
) -> Result<(), BanksClientError> {
    // новый blockhash на каждую транзакцию: одинаковые tx с одним blockhash банк отбросит как дубликат
    let bh = ctx.get_new_latest_blockhash().await.unwrap();
    let mut signers: Vec<&Keypair> = vec![&ctx.payer];
    signers.extend_from_slice(extra);
    let tx = Transaction::new_signed_with_payer(ixs, Some(&ctx.payer.pubkey()), &signers, bh);
    ctx.banks_client.process_transaction(tx).await
}

/// Создаёт аккаунт счётчика (40 байт, rent-exempt, owner = программа) и инициализирует его.
async fn create_and_init(ctx: &mut ProgramTestContext, program_id: Pubkey) -> Keypair {
    let counter = Keypair::new();
    let lamports = Rent::default().minimum_balance(LEN);
    let payer = ctx.payer.pubkey();
    let create = system_instruction::create_account(
        &payer,
        &counter.pubkey(),
        lamports,
        LEN as u64,
        &program_id,
    );
    let init = ix(program_id, INITIALIZE, counter.pubkey(), payer, true);
    send(ctx, &[create, init], &[&counter]).await.unwrap();
    counter
}

async fn read(ctx: &mut ProgramTestContext, key: Pubkey) -> (u64, [u8; 32]) {
    let acc = ctx.banks_client.get_account(key).await.unwrap().unwrap();
    assert_eq!(acc.data.len(), LEN);
    (
        u64::from_le_bytes(acc.data[..8].try_into().unwrap()),
        acc.data[8..].try_into().unwrap(),
    )
}

fn custom_code(err: BanksClientError) -> u32 {
    match err.unwrap() {
        TransactionError::InstructionError(_, InstructionError::Custom(code)) => code,
        other => panic!("ожидали custom error, получили {other:?}"),
    }
}

#[tokio::test]
async fn initialize_then_increment() {
    let (mut ctx, program_id) = setup().await;
    let counter = create_and_init(&mut ctx, program_id).await;
    let (count, authority) = read(&mut ctx, counter.pubkey()).await;
    assert_eq!(count, 0);
    assert_eq!(authority, ctx.payer.pubkey().to_bytes());

    let payer = ctx.payer.pubkey();
    for expected in 1..=3u64 {
        let inc = ix(program_id, INCREMENT, counter.pubkey(), payer, true);
        send(&mut ctx, &[inc], &[]).await.unwrap();
        assert_eq!(read(&mut ctx, counter.pubkey()).await.0, expected);
    }
}

#[tokio::test]
async fn increment_by_stranger_is_unauthorized() {
    let (mut ctx, program_id) = setup().await;
    let counter = create_and_init(&mut ctx, program_id).await;
    let stranger = Keypair::new();
    let inc = ix(
        program_id,
        INCREMENT,
        counter.pubkey(),
        stranger.pubkey(),
        true,
    );
    let err = send(&mut ctx, &[inc], &[&stranger]).await.unwrap_err();
    assert_eq!(custom_code(err), 7); // Unauthorized
    assert_eq!(read(&mut ctx, counter.pubkey()).await.0, 0);
}

#[tokio::test]
async fn increment_without_authority_signature_fails() {
    let (mut ctx, program_id) = setup().await;
    let counter = create_and_init(&mut ctx, program_id).await;
    // authority передан, но не подписывает (is_signer = false)
    let authority = Keypair::new();
    let inc = ix(
        program_id,
        INCREMENT,
        counter.pubkey(),
        authority.pubkey(),
        false,
    );
    let err = send(&mut ctx, &[inc], &[]).await.unwrap_err();
    assert_eq!(custom_code(err), 3); // MissingSignature
}

#[tokio::test]
async fn increment_foreign_account_is_rejected() {
    let program_id = Pubkey::new_unique();
    let mut pt = ProgramTest::new("counter", program_id, None);
    pt.prefer_bpf(true);
    // аккаунт нужного размера, но принадлежит System Program, а не нашей программе
    let foreign = Pubkey::new_unique();
    pt.add_account(
        foreign,
        Account {
            lamports: Rent::default().minimum_balance(LEN),
            data: vec![0; LEN],
            owner: system_program::id(),
            executable: false,
            rent_epoch: 0,
        },
    );
    let mut ctx = pt.start_with_context().await;
    let payer = ctx.payer.pubkey();
    let inc = ix(program_id, INCREMENT, foreign, payer, true);
    let err = send(&mut ctx, &[inc], &[]).await.unwrap_err();
    assert_eq!(custom_code(err), 1); // IncorrectOwner
}

#[tokio::test]
async fn double_initialize_and_bad_data_fail() {
    let (mut ctx, program_id) = setup().await;
    let counter = create_and_init(&mut ctx, program_id).await;
    let payer = ctx.payer.pubkey();

    let again = ix(program_id, INITIALIZE, counter.pubkey(), payer, true);
    assert_eq!(
        custom_code(send(&mut ctx, &[again], &[]).await.unwrap_err()),
        5
    ); // AlreadyInitialized

    let mut bad = ix(program_id, INCREMENT, counter.pubkey(), payer, true);
    bad.data = vec![7];
    assert_eq!(
        custom_code(send(&mut ctx, &[bad], &[]).await.unwrap_err()),
        0
    ); // InvalidInstruction
}
