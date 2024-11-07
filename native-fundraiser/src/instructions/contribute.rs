use pinocchio::{
    ProgramResult,
    account_info::AccountInfo,
    msg,
    program_error::ProgramError,
    pubkey,
    instruction::{
        Seed,
        Signer,
    }
};
use crate::processor::Contribute;
use crate::constants::MIN_AMOUNT_TO_RAISE;
use crate::state::Fundraiser;

pub fn contribute(
    accounts: &[AccountInfo],
    args: &[u8]
) -> ProgramResult {
    msg!("contribute: started");
    let [ 
        contributor,
        mint_to_raise,
        fundraiser,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    assert!(contributor.is_signer());

    let Contribute {
        amount,
        contributor_bump,
    } = Contribute::try_from(args)?;

    msg!("contribute: amount: {}", amount);
    msg!("contribute: contributor_bump: {}", contributor_bump);

    // Check if the amount to contribute meets the minimum amount required
    assert!(amount >= MIN_AMOUNT_TO_RAISE);

    // Borrow the data and immediately convert it
    let fundraiser_clone = fundraiser.clone();
    let fundraiser_data: Fundraiser = *bytemuck::try_from_bytes::<Fundraiser>(&fundraiser_clone.try_borrow_data()?).map_err(|_| ProgramError::InvalidAccountData)?;

    // Check if the total amount raised is less than the maximum amount to raise
    assert!(amount + fundraiser_data.current_amount <= fundraiser_data.amount_to_raise);

    // Check if contributor pda already exists
    let contributor_pda = pubkey::find_program_address(&[b"contributor", &contributor.key().as_ref(), &contributor_bump.to_le_bytes()], &crate::ID);

    // Check if the amount to contribute is less than the maximum allowed contribution per contributor
    // if contributor_pda

    Ok(())
}