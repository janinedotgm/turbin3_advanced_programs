use pinocchio::{
    account_info::AccountInfo, 
    program_error::ProgramError, 
    sysvars::{clock::Clock, Sysvar}, 
    ProgramResult
};
use crate::state::{ Fundraiser, Contributor };
use pinocchio_token::instructions::Transfer;

pub fn refund(accounts: &[AccountInfo], _data: &[u8]) -> ProgramResult {

    let [ 
        _contributor,
        fundraiser,
        contributor_ta,
        contributor_account, // account that saves the contribution amount of the contributor
        vault, // account that receives the contribution
        _token_program,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let current_time = Clock::get()?.unix_timestamp; 
    
    let fundraiser_data: Fundraiser = *bytemuck::try_from_bytes::<Fundraiser>(&fundraiser.try_borrow_mut_data()?).map_err(|_| ProgramError::InvalidAccountData)?;
    let contributor_data: Contributor = *bytemuck::try_from_bytes::<Contributor>(&contributor_account.try_borrow_mut_data()?).map_err(|_| ProgramError::InvalidAccountData)?;

    // make sure the fundraiser duration has been reached
    assert!(fundraiser_data.duration <= (current_time - fundraiser_data.time_started)); // TODO: make sure to save and convert the time correctly

    // make sure the fundraising goal has not been reached
    assert!(fundraiser_data.current_amount < fundraiser_data.amount_to_raise);

    // transfer the contribution amount back to the contributor
    Transfer {
        from: vault,
        to: contributor_ta,
        authority: vault,
        amount: contributor_data.amount
    }.invoke()?;

    // TODO:close vault and fundraiser if vault is empty?

    Ok(())
}