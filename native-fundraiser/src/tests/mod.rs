#![cfg(test)]
mod initialize;
mod contribute;
mod checker;
mod refund;

/**
 * Setup functions for tests
 */
use mollusk_svm::{
    program,
    Mollusk,
};
use solana_sdk::{
    account::{AccountSharedData, WritableAccount}, 
    program_option::COption, 
    program_pack::Pack, 
    pubkey::Pubkey as SolanaPubkey
};
use spl_token::state::AccountState;
use bytemuck::bytes_of;
use crate::state::Fundraiser;


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
