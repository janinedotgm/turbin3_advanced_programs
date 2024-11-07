#![cfg(test)]
mod tests {
    use mollusk_svm::{
        program, 
        Mollusk,
        result::ProgramResult
    };
    use solana_sdk::{
        account::{AccountSharedData, WritableAccount},
        instruction::{AccountMeta, Instruction},
        program_option::COption,
        program_pack::Pack,
        pubkey::Pubkey as SolanaPubkey,
    };
    use spl_token::state::AccountState;
    use bytemuck::bytes_of;
    use crate::state::Fundraiser;

    const PROGRAM_ID: SolanaPubkey = SolanaPubkey::new_from_array(five8_const::decode_32_const(
        "22222222222222222222222222222222222222222222",
    ));

    fn setup_mollusk() -> Mollusk{
        let mut mollusk = Mollusk::new(&PROGRAM_ID, "target/deploy/native_fundraiser");

        mollusk.add_program(
            &spl_token::ID,
            "src/tests/spl_token-3.5.0",
            &mollusk_svm::program::loader_keys::LOADER_V3,
        );

        mollusk
    }

    fn create_mint_account(mollusk: &Mollusk, mint: &SolanaPubkey) -> AccountSharedData{
        let (token_program, token_program_account) = (
            spl_token::ID,
            program::create_program_account_loader_v3(&spl_token::ID),
        );

        let mut mint_account = AccountSharedData::new(
            mollusk
                .sysvars
                .rent
                .minimum_balance(spl_token::state::Mint::LEN),
            spl_token::state::Mint::LEN,
            &token_program,
        );
        Pack::pack(
            spl_token::state::Mint {
                mint_authority: COption::Some(SolanaPubkey::new_from_array([0x05; 32])),
                supply: 100_000_000_000,
                decimals: 6,
                is_initialized: true,
                freeze_authority: COption::None,
            },
            mint_account.data_as_mut_slice(),
        ).unwrap();

        mint_account
    }


    #[test]
    fn initialize(){

        let mollusk = setup_mollusk();

        let maker = SolanaPubkey::new_unique();
        let (fundraiser, bump) = SolanaPubkey::find_program_address(&[b"fundraiser", maker.as_ref()], &PROGRAM_ID);
        let mint_to_raise = SolanaPubkey::new_unique();
        let mint_account = create_mint_account(&mollusk, &mint_to_raise);
        let vault = SolanaPubkey::find_program_address(&[b"vault", fundraiser.as_ref()], &PROGRAM_ID).0;

        
        let (system_program, system_program_account) = program::keyed_account_for_system_program();
        let (token_program, token_program_account) = (
            spl_token::ID,
            program::create_program_account_loader_v3(&spl_token::ID),
        );

        let (rent_sysvar, rent_sysvar_account) = (
            solana_sdk::sysvar::rent::ID,
            program::create_program_account_loader_v3(&solana_sdk::sysvar::rent::ID),
        );

        let mut vault_account = AccountSharedData::new(
            mollusk
                .sysvars
                .rent
                .minimum_balance(spl_token::state::Account::LEN),
            spl_token::state::Account::LEN,
            &token_program,
        );

        Pack::pack(
            spl_token::state::Account {
                mint: mint_to_raise,
                owner: fundraiser,
                amount: 0,
                delegate: COption::None,
                state: AccountState::Initialized,
                is_native: COption::None,
                delegated_amount: 0,
                close_authority: COption::None,
            },
            vault_account.data_as_mut_slice(),
        ).unwrap();
       
        let bump_u64 = bump as u64;

        let data = [
            vec![0],
            1_000_000u64.to_le_bytes().to_vec(), // amount
            30u64.to_le_bytes().to_vec(), // duration
            bump_u64.to_le_bytes().to_vec(), // fundraiser_bump
        ]
        .concat();

        let fundraiser_account = AccountSharedData::new(0, 0, &SolanaPubkey::default());

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
                (fundraiser, fundraiser_account),
                (system_program, system_program_account),
            ],
        );

        assert!(matches!(result.program_result, ProgramResult::Success))
    }

    #[test]
    fn contribution_too_small(){
        let mollusk = setup_mollusk();
        let maker = SolanaPubkey::new_unique();
        let (fundraiser, bump) = SolanaPubkey::find_program_address(&[b"fundraiser", maker.as_ref()], &PROGRAM_ID);
        let fundraiser_account = AccountSharedData::new(0, 0, &SolanaPubkey::default());

        let contributor = SolanaPubkey::new_unique();
        let mint_to_raise = SolanaPubkey::new_unique();
        let mint_account = create_mint_account(&mollusk, &mint_to_raise);

        let data = [
            vec![1],
            1u64.to_le_bytes().to_vec(), // amount
            123u64.to_le_bytes().to_vec(), // vault_bump
        ]
        .concat();

        let instruction = Instruction::new_with_bytes(
            PROGRAM_ID,
            &data,
            vec![
                AccountMeta::new(contributor, true),
                AccountMeta::new(mint_to_raise, false),
                AccountMeta::new(fundraiser, false),
            ],
        );

        let result: mollusk_svm::result::InstructionResult = mollusk.process_instruction(
            &instruction,
            &vec![
                (contributor, AccountSharedData::new(1_000_000_000, 0, &SolanaPubkey::default())),
                (mint_to_raise, mint_account),
                (fundraiser, fundraiser_account),

            ],
        );

        assert!(result.program_result.is_err());
    }

    #[test]
    fn contribute(){

        let mollusk = setup_mollusk();
        let maker = SolanaPubkey::new_unique();
        let (fundraiser, bump) = SolanaPubkey::find_program_address(&[b"fundraiser", maker.as_ref()], &PROGRAM_ID);
        let space = std::mem::size_of::<Fundraiser>();

        let contributor = SolanaPubkey::new_unique();
        let mint_to_raise = SolanaPubkey::new_unique();
        let mint_account = create_mint_account(&mollusk, &mint_to_raise);

        let mut fundraiser_account = AccountSharedData::new(
            mollusk
                .sysvars
                .rent
                .minimum_balance(std::mem::size_of::<Fundraiser>()),
            std::mem::size_of::<Fundraiser>(),
            &PROGRAM_ID,
        );
        fundraiser_account.set_data_from_slice(bytes_of::<Fundraiser>(&Fundraiser {
            maker: maker.to_bytes(),
            mint_to_raise: mint_to_raise.to_bytes(),
            amount_to_raise: 1_000_000,
            current_amount: 0,
            time_started: 0,
            duration: 30,
            bump: bump as u64,
        }));

        let data = [
            vec![1],
            1000u64.to_le_bytes().to_vec(), // amount
            123u64.to_le_bytes().to_vec(), // vault_bump
        ]
        .concat();

        let instruction = Instruction::new_with_bytes(
            PROGRAM_ID,
            &data,
            vec![
                AccountMeta::new(contributor, true),
                AccountMeta::new(mint_to_raise, false),
                AccountMeta::new(fundraiser, false),
            ],
        );

        let result: mollusk_svm::result::InstructionResult = mollusk.process_instruction(
            &instruction,
            &vec![
                (contributor, AccountSharedData::new(1_000_000_000, 0, &SolanaPubkey::default())),
                (mint_to_raise, mint_account),
                (fundraiser, fundraiser_account),

            ],
        );

        assert!(matches!(result.program_result, ProgramResult::Success))
    }
}