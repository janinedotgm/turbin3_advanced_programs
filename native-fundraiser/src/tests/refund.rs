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
use bytemuck::bytes_of;

use crate::state::Contributor;

use crate::tests::{
    setup_mollusk,
    create_initialized_fundraiser_account,
    create_initialized_ata_account,
    create_initialized_vault_account,
    PROGRAM_ID
};

#[test]
fn refund(){
    let mut mollusk = setup_mollusk();
    let contributor = SolanaPubkey::new_unique(); // The one who contributed to the fundraiser
    let maker = SolanaPubkey::new_unique(); // The one who created the fundraiser
    let mint_to_raise = SolanaPubkey::new_unique(); // Currency in which was raised
    let (fundraiser, fundraiser_bump) = SolanaPubkey::find_program_address(&[b"fundraiser", maker.as_ref()], &PROGRAM_ID);
    let fundraiser_account = create_initialized_fundraiser_account(maker, &mollusk, mint_to_raise, 100_000, 0, fundraiser_bump as u64);

    let (token_program, token_program_account) = (
        spl_token::ID,
        program::create_program_account_loader_v3(&spl_token::ID),
    );
    let (vault, vault_bump) = SolanaPubkey::find_program_address(&[b"vault", fundraiser.as_ref()], &PROGRAM_ID);
    let (contributor_account, contributer_account_bump) = SolanaPubkey::find_program_address(&[b"contributor", fundraiser.as_ref(), contributor.as_ref()], &PROGRAM_ID);
    let contributor_ata = SolanaPubkey::new_unique();

    let vault_account = create_initialized_vault_account(vault, mint_to_raise, &mollusk, 100_000);

    let mut contributor_account_data = AccountSharedData::new(
        mollusk.sysvars.rent.minimum_balance(Contributor::LEN),
        Contributor::LEN,
        &PROGRAM_ID,
    );
    contributor_account_data.set_data_from_slice(bytes_of::<Contributor>(&Contributor {
        amount: 100_000,
        contributor_bump: contributer_account_bump as u64
    }));

    let contributor_ata_account = create_initialized_ata_account(contributor, &mollusk, mint_to_raise, 123);

    // Mock the time to be past the duration
    mollusk.sysvars.clock.unix_timestamp = 31;

    let fundraiser_bump_u64 = fundraiser_bump as u64;
    let vault_bump_u64 = vault_bump as u64;
    let data = [
        vec![3],
        fundraiser_bump_u64.to_le_bytes().to_vec(),
        vault_bump_u64.to_le_bytes().to_vec(),
    ]
    .concat();

    let instruction = Instruction::new_with_bytes(
        PROGRAM_ID,
        &data,
        vec![
            AccountMeta::new(contributor, true),
            AccountMeta::new(fundraiser, false),
            AccountMeta::new(contributor_ata, false),
            AccountMeta::new(contributor_account, false),
            AccountMeta::new(vault, true),
            AccountMeta::new(token_program, false),
        ],
    );

    let result: mollusk_svm::result::InstructionResult = mollusk.process_instruction(
        &instruction,
        &vec![
            (contributor, AccountSharedData::new(1_000_000_000, 0, &SolanaPubkey::default())),
            (fundraiser, fundraiser_account),
            (contributor_ata, contributor_ata_account),
            (contributor_account, contributor_account_data),
            (vault, vault_account),
            (token_program, token_program_account),
        ],
    );
    assert!(matches!(result.program_result, ProgramResult::Success), "Processing instruction failed");

    let vault = result.get_account(&vault).expect("Failed to get vault account");
    let vault_data = vault.data();
    let vault_account_info = spl_token::state::Account::unpack(&vault_data).unwrap();

    assert_eq!(vault_account_info.amount, 0);
}
