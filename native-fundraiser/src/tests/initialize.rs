use mollusk_svm::{
    program,
    result::ProgramResult,
};
use solana_sdk::{
    account::AccountSharedData, 
    instruction::{AccountMeta, Instruction}, 
    pubkey::Pubkey as SolanaPubkey
};

use crate::tests::{
    setup_mollusk,
    create_mint_account,
    PROGRAM_ID
};


#[test]
fn initialize(){

    let mollusk = setup_mollusk();

    let maker = SolanaPubkey::new_unique();
    let (fundraiser, bump) = SolanaPubkey::find_program_address(&[b"fundraiser", maker.as_ref()], &PROGRAM_ID);

    let mint_to_raise = SolanaPubkey::new_unique();
    let mint_account = create_mint_account(&mollusk, &mint_to_raise);
    
    let (system_program, system_program_account) = program::keyed_account_for_system_program();
    
    let bump_u64 = bump as u64;

    let data = [
        vec![0],
        1_000_000u64.to_le_bytes().to_vec(), // amount
        30u64.to_le_bytes().to_vec(), // duration
        bump_u64.to_le_bytes().to_vec(), // fundraiser_bump
    ]
    .concat();

    let instruction = Instruction::new_with_bytes(
        PROGRAM_ID,
        &data,
        vec![
            AccountMeta::new(maker, true),
            AccountMeta::new(mint_to_raise, false),
            AccountMeta::new(fundraiser, false),
            AccountMeta::new(system_program, false),
        ],
    );

    let result: mollusk_svm::result::InstructionResult = mollusk.process_instruction(
        &instruction,
        &vec![
            (
                maker,
                AccountSharedData::new(1_000_000_000, 0, &SolanaPubkey::default()),
            ),
            (mint_to_raise, mint_account),
            (fundraiser, AccountSharedData::new(0, 0, &SolanaPubkey::default())),
            (system_program, system_program_account),
        ],
    );

    assert!(matches!(result.program_result, ProgramResult::Success))
}