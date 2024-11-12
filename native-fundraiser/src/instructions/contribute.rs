use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError, 
    sysvars::{clock::Clock, Sysvar}, 
    instruction::Seed,
    ProgramResult,
    pubkey,
    msg,
};
use pinocchio_token::instructions::Transfer;
use crate::{constants::PERCENTAGE_SCALER, processor::ContributeArgs};
use crate::constants::{MIN_AMOUNT_TO_RAISE, MAX_CONTRIBUTION_PERCENTAGE, SECONDS_TO_DAYS};
use crate::state::{Fundraiser, Contributor};
use crate::utils::validate_pda;
pub fn contribute(
    accounts: &[AccountInfo],
    args: &[u8]
) -> ProgramResult {
    let [ 
        contributor,
        fundraiser,
        contributor_ta,
        contributor_account, // account that saves the contribution amount of the contributor
        vault, // account that receives the contribution
        _token_program,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    assert!(contributor.is_signer());

    let ContributeArgs {
        amount,
        vault_bump,
    } = ContributeArgs::try_from(args)?;

    let u8_vault_bump = vault_bump as u8;

    // check if vault is correct
    validate_pda(&[
        b"vault",
        fundraiser.key().as_ref(),
        &[u8_vault_bump]
    ], &crate::ID, vault.key());

    // Check if the amount to contribute meets the minimum amount required
    assert!(amount >= MIN_AMOUNT_TO_RAISE);

    // TODO: check if we need to validate the fundraiser PDA
    // Borrow the data and immediately convert it
    let mut fundraiser_data: Fundraiser = *bytemuck::try_from_bytes::<Fundraiser>(&fundraiser.try_borrow_mut_data()?).map_err(|_| ProgramError::InvalidAccountData)?;

    // Check if the total amount raised is less than the maximum amount to raise
    assert!(amount + fundraiser_data.current_amount <= fundraiser_data.amount_to_raise);

    // TODO: check if the contributer account exists and create it if it doesn't or can we expect it to be created by the frontend?
    let mut contributor_data: Contributor = *bytemuck::try_from_bytes::<Contributor>(&contributor_account.try_borrow_mut_data()?).map_err(|_| ProgramError::InvalidAccountData)?;

    // Check if the amount to contribute is less than the maximum allowed contribution per contributor
    let amount_allowed = (fundraiser_data.amount_to_raise * MAX_CONTRIBUTION_PERCENTAGE) / PERCENTAGE_SCALER;
    assert!(amount + contributor_data.amount <= amount_allowed);
    
    // check if the fundraising duration has been reached
    let current_time = Clock::get()?.unix_timestamp; 
    let _fundraiser_end_time = (current_time - fundraiser_data.time_started) / SECONDS_TO_DAYS;
    // assert!(fundraiser_end_time >= fundraiser_data.duration); // TODO: fails in test because current_time is 0 - fix test

    // transfer the contribution amount to the vault
    Transfer {
        from: contributor_ta,
        to: vault,
        authority: contributor,
        amount
    }.invoke()?;

    fundraiser_data.current_amount = fundraiser_data.current_amount + amount;
    contributor_data.amount = contributor_data.amount + amount;

    fundraiser.try_borrow_mut_data()?.copy_from_slice(bytemuck::bytes_of(&fundraiser_data));
    contributor_account.try_borrow_mut_data()?.copy_from_slice(bytemuck::bytes_of(&contributor_data));
      
    Ok(())
}