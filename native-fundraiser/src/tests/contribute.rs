use mollusk_svm::program;
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
fn contribute(){
    let mollusk = setup_mollusk();

    // the maker is the one raising the funds
    let maker = SolanaPubkey::new_unique();

    // the mint that the fundraiser wants to raise
    let mint_to_raise = SolanaPubkey::new_unique();

    // create the fundraiser account that holds the state of the fundraiser
    let (fundraiser, bump) = SolanaPubkey::find_program_address(&[b"fundraiser", maker.as_ref()], &PROGRAM_ID);
    let fundraiser_account = create_initialized_fundraiser_account(maker, &mollusk, mint_to_raise, 0, 0, bump as u64);

    // The person who wants to contribute to the fundraiser
    let contributor = SolanaPubkey::new_unique();
    let contributor_account_data = AccountSharedData::new(1_000_000_000, 0, &contributor);

    // The contributor's token account
    let contributor_ata = SolanaPubkey::new_unique();
    let contributor_ata_account = create_initialized_ata_account(contributor, &mollusk, mint_to_raise, 1_000_000_000);

    // The PDA that holds the state of the contributor
    let (contributor_pda, contributor_bump) = SolanaPubkey::find_program_address(&[b"contributor", contributor.as_ref()], &PROGRAM_ID);
    let contributor_bump_u64 = contributor_bump as u64;
    let mut contributor_account = AccountSharedData::new(
        mollusk.sysvars.rent.minimum_balance(Contributor::LEN),
        Contributor::LEN,
        &PROGRAM_ID,
    );

    contributor_account.set_data_from_slice(bytes_of::<Contributor>(&Contributor {
        amount: 0,
        contributor_bump: contributor_bump_u64,
    }));

    // The vault that holds the funds raised
    let (vault, vault_bump) = SolanaPubkey::find_program_address(&[b"vault", fundraiser.as_ref()], &PROGRAM_ID);
    let vault_account = create_initialized_vault_account(fundraiser, mint_to_raise, &mollusk,0);

    // The token program
    let (token_program, token_program_account) = (
        spl_token::ID,
        program::create_program_account_loader_v3(&spl_token::ID),
    );        

    let bump_u64 = vault_bump as u64;
    let data = [
        vec![1],
        1_000u64.to_le_bytes().to_vec(), // amount
        bump_u64.to_le_bytes().to_vec(), // vault bump
    ]
    .concat();


    let instruction = Instruction::new_with_bytes(
        PROGRAM_ID,
        &data,
        vec![
            AccountMeta::new(contributor, true),
            AccountMeta::new(fundraiser, false),
            AccountMeta::new(contributor_ata, false),
            AccountMeta::new(contributor_pda, false),
            AccountMeta::new(vault, false),
            AccountMeta::new(token_program, false),
        ],
    );

    let result = mollusk.process_instruction(
        &instruction,
        &vec![
            (contributor, contributor_account_data),
            (fundraiser, fundraiser_account),
            (contributor_ata, contributor_ata_account),
            (contributor_pda, contributor_account),
            (vault, vault_account),
            (token_program, token_program_account),
        ],
    );

    let vault = result.get_account(&vault).expect("Failed to get vault account");

    let vault_data = vault.data();
    let vault_account_info = spl_token::state::Account::unpack(&vault_data).unwrap();

    assert_eq!(vault_account_info.amount, 1_000);

    // TODO: check if contributer account safed correct amount

}