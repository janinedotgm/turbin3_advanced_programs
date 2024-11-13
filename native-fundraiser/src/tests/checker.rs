use mollusk_svm::{
    program,
    result::ProgramResult,
};
use solana_sdk::{
    account::{AccountSharedData, ReadableAccount}, 
    instruction::{AccountMeta, Instruction}, 
    program_pack::Pack, 
    pubkey::Pubkey as SolanaPubkey
};

use crate::tests::{
    setup_mollusk,
    create_initialized_fundraiser_account,
    create_initialized_ata_account,
    create_initialized_vault_account,
    PROGRAM_ID
};

#[test]
fn checker(){
    let mollusk = setup_mollusk();
    let maker = SolanaPubkey::new_unique();
    let mint_to_raise = SolanaPubkey::new_unique();
    let (fundraiser, fundraiser_bump) = SolanaPubkey::find_program_address(&[b"fundraiser", maker.as_ref()], &PROGRAM_ID);
    let (token_program, token_program_account) = (
        spl_token::ID,
        program::create_program_account_loader_v3(&spl_token::ID),
    );
    let (vault, vault_bump) = SolanaPubkey::find_program_address(&[b"vault", fundraiser.as_ref()], &PROGRAM_ID);
    let maker_ata = SolanaPubkey::new_unique();
    let fundraiser_account = create_initialized_fundraiser_account(maker, &mollusk, mint_to_raise, 1_000_000, 0, fundraiser_bump as u64);

    let vault_account = create_initialized_vault_account(vault, mint_to_raise, &mollusk,1_000_000);
    let maker_ata_account = create_initialized_ata_account(maker, &mollusk, mint_to_raise, 1_000_000_000);

    let fundraiser_bump_u64 = fundraiser_bump as u64;
    let vault_bump_u64 = vault_bump as u64;
    let data = [
        vec![2],
        fundraiser_bump_u64.to_le_bytes().to_vec(),
        vault_bump_u64.to_le_bytes().to_vec(),
    ]
    .concat();

    let instruction = Instruction::new_with_bytes(
        PROGRAM_ID,
        &data,
        vec![
            AccountMeta::new(maker, true),
            AccountMeta::new(fundraiser, false),
            AccountMeta::new(vault, false),
            AccountMeta::new(maker_ata, false),
            AccountMeta::new(token_program, false),
        ],
    );

    let result: mollusk_svm::result::InstructionResult = mollusk.process_instruction(
        &instruction,
        &vec![
            (maker, AccountSharedData::new(1_000_000_000, 0, &SolanaPubkey::default())),
            (fundraiser, fundraiser_account),
            (vault, vault_account),
            (maker_ata, maker_ata_account),
            (token_program, token_program_account),
        ],
    );
    assert!(matches!(result.program_result, ProgramResult::Success), "Processing instruction failed");

    let vault = result.get_account(&vault).expect("Failed to get vault account");

    let vault_data = vault.data();
    let vault_account_info = spl_token::state::Account::unpack(&vault_data).unwrap();

    assert_eq!(vault_account_info.amount, 0);

    // TODO: check if maker_ata has the correct amount
}