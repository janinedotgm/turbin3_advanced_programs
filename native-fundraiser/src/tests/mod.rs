#![cfg(test)]
mod tests {
    use pinocchio::program_error::ProgramError;
    use mollusk_svm::{
        program,
        Mollusk,
        result::ProgramResult
    };
    use solana_sdk::{
        account::{AccountSharedData, ReadableAccount, WritableAccount}, 
        instruction::{AccountMeta, Instruction}, 
        program_option::COption, 
        program_pack::Pack, 
        pubkey::Pubkey as SolanaPubkey
    };
    use spl_token::state::AccountState;
    use bytemuck::bytes_of;
    use crate::state::{Fundraiser, Contributor};

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

    fn create_mint_account(mollusk: &Mollusk, _mint: &SolanaPubkey) -> AccountSharedData{
        let (token_program, _token_program_account) = (
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

        
        let (system_program, system_program_account) = program::keyed_account_for_system_program();
        let (token_program, _token_program_account) = (
            spl_token::ID,
            program::create_program_account_loader_v3(&spl_token::ID),
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
    fn contribute(){
        let mollusk = setup_mollusk();

        // the maker is the one raising the funds
        let maker = SolanaPubkey::new_unique();

        // the mint that the fundraiser wants to raise
        let mint_to_raise = SolanaPubkey::new_unique();

        // create the fundraiser account that holds the state of the fundraiser
        let (fundraiser, bump) = SolanaPubkey::find_program_address(&[b"fundraiser", maker.as_ref()], &PROGRAM_ID);
        let mut fundraiser_account = AccountSharedData::new(
            mollusk.sysvars.rent.minimum_balance(Fundraiser::LEN),
            Fundraiser::LEN,
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

        // The person who wants to contribute to the fundraiser
        let contributor = SolanaPubkey::new_unique();
        let contributor_account_data = AccountSharedData::new(1_000_000_000, 0, &contributor);

        // The contributor's token account
        let contributor_ata = SolanaPubkey::new_unique();
        
        let mut contributor_ata_account = AccountSharedData::new(
            mollusk
                .sysvars
                .rent
                .minimum_balance(spl_token::state::Account::LEN),
            spl_token::state::Account::LEN,
            &spl_token::id(),
        );
        spl_token::state::Account::pack(
            spl_token::state::Account {
                mint: mint_to_raise,
                owner: contributor,
                amount: 1_000_000_000,
                delegate: COption::None,
                state: AccountState::Initialized,
                is_native: COption::None,
                delegated_amount: 0,
                close_authority: COption::None,
            },
            contributor_ata_account.data_as_mut_slice(),
        ).unwrap();

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
        let vault_size = spl_token::state::Account::LEN;
        let mut vault_account = AccountSharedData::new(
            mollusk.sysvars.rent.minimum_balance(vault_size),
            vault_size,
            &spl_token::ID,
        );
        spl_token::state::Account::pack(
            spl_token::state::Account {
                mint: mint_to_raise,
                owner: vault,
                amount: 0,
                delegate: COption::None,
                state: AccountState::Initialized,
                is_native: COption::None,
                delegated_amount: 0,
                close_authority: COption::None,
            },
            vault_account.data_as_mut_slice(),
        ).unwrap();

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

    #[test]
    fn checker(){
        let mollusk = setup_mollusk();
        let maker = SolanaPubkey::new_unique();
        let (fundraiser, fundraiser_bump) = SolanaPubkey::find_program_address(&[b"fundraiser", maker.as_ref()], &PROGRAM_ID);
        let mint_to_raise = SolanaPubkey::new_unique();
        let (token_program, token_program_account) = (
            spl_token::ID,
            program::create_program_account_loader_v3(&spl_token::ID),
        );
        let (vault, vault_bump) = SolanaPubkey::find_program_address(&[b"vault", fundraiser.as_ref()], &PROGRAM_ID);
        let maker_ata = SolanaPubkey::new_unique();

        let mut fundraiser_account = AccountSharedData::new(
            mollusk.sysvars.rent.minimum_balance(Fundraiser::LEN),
            Fundraiser::LEN,
            &PROGRAM_ID,
        );
        fundraiser_account.set_data_from_slice(bytes_of::<Fundraiser>(&Fundraiser {
            maker: maker.to_bytes(),
            mint_to_raise: mint_to_raise.to_bytes(),
            amount_to_raise: 1_000_000,
            current_amount: 1_000_000,
            time_started: 0,
            duration: 30,
            bump: fundraiser_bump as u64,
        }));

        let vault_size = spl_token::state::Account::LEN;
        let mut vault_account = AccountSharedData::new(
            mollusk.sysvars.rent.minimum_balance(vault_size),
            vault_size,
            &spl_token::ID,
        );
        spl_token::state::Account::pack(
            spl_token::state::Account {
                mint: mint_to_raise,
                owner: vault,
                amount: 1_000_000,
                delegate: COption::None,
                state: AccountState::Initialized,
                is_native: COption::None,
                delegated_amount: 0,
                close_authority: COption::None,
            },
            vault_account.data_as_mut_slice(),
        ).unwrap();
        
        let mut maker_ata_account = AccountSharedData::new(
            mollusk
                .sysvars
                .rent
                .minimum_balance(spl_token::state::Account::LEN),
            spl_token::state::Account::LEN,
            &spl_token::id(),
        );
        spl_token::state::Account::pack(
            spl_token::state::Account {
                mint: mint_to_raise,
                owner: maker,
                amount: 123,
                delegate: COption::None,
                state: AccountState::Initialized,
                is_native: COption::None,
                delegated_amount: 0,
                close_authority: COption::None,
            },
            maker_ata_account.data_as_mut_slice(),
        ).unwrap();

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
                AccountMeta::new(vault, true),
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

}