#![cfg(test)]
mod tests {
    use mollusk_svm::{
        program,
        Mollusk,
        result::ProgramResult,
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

    const AMOUNT_TO_RAISE: u64 = 1_000_000;
    const DURATION: i64 = 30;

    fn setup_mollusk() -> Mollusk{
        let mut mollusk = Mollusk::new(&PROGRAM_ID, "target/deploy/native_fundraiser");

        mollusk.add_program(
            &spl_token::ID,
            "src/tests/spl_token-3.5.0",
            &mollusk_svm::program::loader_keys::LOADER_V3,
        );

        mollusk
    }

    /**
     * Creates an initialized fundraiser account
     * 
     * @param maker: The maker of the fundraiser
     * @param mollusk: The mollusk instance
     * @param mint_to_raise: The mint that the fundraiser is raising
     * @param current_amount: The current amount of the fundraiser
     * @param time_started: The time the fundraiser started
     * @param fundraiser_bump: The bump of the fundraiser
     * @returns: An initialized fundraiser account
     */
    fn create_initialized_fundraiser_account(
        maker: SolanaPubkey,
        mollusk: &Mollusk,
        mint_to_raise: SolanaPubkey,
        current_amount: u64,
        time_started: i64,
        fundraiser_bump: u64,
    ) -> AccountSharedData {

        let mut fundraiser_account = AccountSharedData::new(
            mollusk.sysvars.rent.minimum_balance(Fundraiser::LEN),
            Fundraiser::LEN,
            &PROGRAM_ID,
        );
        fundraiser_account.set_data_from_slice(bytes_of::<Fundraiser>(&Fundraiser {
            maker: maker.to_bytes(),
            mint_to_raise: mint_to_raise.to_bytes(),
            amount_to_raise: AMOUNT_TO_RAISE,
            current_amount,
            time_started,
            duration: DURATION,
            bump: fundraiser_bump,
        }));

        fundraiser_account
    }

    /**
     * Creates an initialized ata account
     * 
     * @param owner: The owner of the ata
     * @param mollusk: The mollusk instance
     * @param mint_to_raise: The mint that the ata is for
     * @param amount: The amount of the ata
     * @returns: An initialized ata account
     */
    fn create_initialized_ata_account(
        owner: SolanaPubkey,
        mollusk: &Mollusk,
        mint_to_raise: SolanaPubkey,
        amount: u64,
    ) -> AccountSharedData {

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
                owner,
                amount,
                delegate: COption::None,
                state: AccountState::Initialized,
                is_native: COption::None,
                delegated_amount: 0,
                close_authority: COption::None,
            },
            contributor_ata_account.data_as_mut_slice(),
        ).unwrap();

        contributor_ata_account
    }


    /**
     * Creates an initialized vault account
     * 
     * @param fundraiser: The fundraiser pda account that holds the fundraiser state
     * @param mint_to_raise: The mint that the vault is for
     * @param mollusk: The mollusk instance
     * @param amount: The amount of the vault
     * @returns: An initialized vault account
     */
    fn create_initialized_vault_account(
        fundraiser: SolanaPubkey,
        mint_to_raise: SolanaPubkey,
        mollusk: &Mollusk,
        amount: u64,
    ) -> AccountSharedData {
        let vault_size = spl_token::state::Account::LEN;
        let mut account_data = AccountSharedData::new(
            mollusk
                .sysvars
                .rent
                .minimum_balance(spl_token::state::Account::LEN),
            vault_size,
            &spl_token::ID,
        );
    
        spl_token::state::Account::pack(
            spl_token::state::Account {
                mint: mint_to_raise,
                owner: fundraiser,
                amount: amount,
                delegate: COption::None,
                state: AccountState::Initialized,
                is_native: COption::None,
                delegated_amount: 0,
                close_authority: COption::None,
            },
            account_data.data_as_mut_slice(),
        )
        .unwrap();
    
        account_data
    }

    /**
     * Creates an initialized mint account
     * 
     * @param mollusk: The mollusk instance
     * @param mint: The mint that the account is for
     * @returns: An initialized mint account
     */
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

}