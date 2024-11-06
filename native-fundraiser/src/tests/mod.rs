#![cfg(test)]
mod tests {
    use mollusk_svm::{program, Mollusk};
    use solana_sdk::{
        account::{AccountSharedData, WritableAccount},
        instruction::{AccountMeta, Instruction},
        program_option::COption,
        program_pack::Pack,
        pubkey::Pubkey as SolanaPubkey,
    };
    use spl_token::state::AccountState;


    #[test]
    fn initialize(){

        let program_id = SolanaPubkey::new_from_array(five8_const::decode_32_const(
            "22222222222222222222222222222222222222222222",
        ));

        let mut mollusk = Mollusk::new(&program_id, "target/deploy/native_fundraiser");

        // [x] maker, // person who is starting the fundraiser (signer)
        // [x] mint_to_raise, // the mint that the maker wants to receive
        // [x] fundraiser, // the PDA that will hold the fundraiser data (seeds: [b"fundraiser", maker.key().as_ref()] + bump, payer: maker, space)
        // [x] vault,  // the vault that will hold the raised funds (authority: fundraiser, mint: mint_to_raise, payer: maker)
        // [x] token_program, // the token program that will be used to transfer the funds 
        // [ ] associated_token_program, // the associated token program that will be used to create the vault
        // [x] system_program, 

        let maker = SolanaPubkey::new_unique();
        let (fundraiser, bump) = SolanaPubkey::find_program_address(&[b"fundraiser", maker.as_ref()], &program_id);
        let mint_to_raise = SolanaPubkey::new_unique();
        let vault = SolanaPubkey::find_program_address(&[b"vault", fundraiser.as_ref()], &program_id).0;

        mollusk.add_program(
            &spl_token::ID,
            "src/tests/spl_token-3.5.0",
            &mollusk_svm::program::loader_keys::LOADER_V3,
        );
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
       
        let bump_u64 = bump as u64;

        let data = [
            vec![0],
            1_000_000u64.to_le_bytes().to_vec(), // amount
            30u64.to_le_bytes().to_vec(), // duration
            bump_u64.to_le_bytes().to_vec(), // fundraiser_bump
            // maker_ta_b.to_bytes().to_vec(),
            // mint_a.to_bytes().to_vec(),
            // mint_b.to_bytes().to_vec(),
            // 1_000_000u64.to_le_bytes().to_vec(),
        ]
        .concat();

        let fundraiser_account = AccountSharedData::new(0, 0, &SolanaPubkey::default());

        let instruction = Instruction::new_with_bytes(
            program_id,
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

        assert!(!result.program_result.is_err());
    }

}