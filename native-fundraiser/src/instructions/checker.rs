use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    ProgramResult,
    pubkey,
    instruction::{Seed, Signer},
};
use pinocchio_token::instructions::Transfer;
use crate::processor::CheckerArgs;
use crate::state::Fundraiser;

pub fn checker(
    accounts: &[AccountInfo],
    args: &[u8]
) -> ProgramResult {

    let [ 
        maker,
        fundraiser,
        vault,
        maker_ata,
        _token_program
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    assert!(maker.is_signer());

    let CheckerArgs {
        fundraiser_bump,
    } = CheckerArgs::try_from(args)?;

    let (_, bump) = pubkey::find_program_address(&[
        b"fundraiser", 
        maker.key().as_ref(), 
    ], &crate::ID);
    assert_eq!(fundraiser_bump, bump as u64);

    let fundraiser_data: Fundraiser = *bytemuck::try_from_bytes::<Fundraiser>(&fundraiser.try_borrow_mut_data()?).map_err(|_| ProgramError::InvalidAccountData)?;

    // check if the fundraising goal has been reached
    assert!(fundraiser_data.current_amount >= fundraiser_data.amount_to_raise);
    
    let bump_binding = bump.to_le_bytes();
    let signer_seeds = [
        Seed::from(b"fundraiser"), 
        Seed::from(maker.key().as_ref()), 
        Seed::from(bump_binding.as_ref())
    ];
    // let signer = [Signer::from(&signer_seeds)];

    // transfer the funds to the maker
    // Transfer {
    //     from: vault,
    //     to: maker_ata,
    //     authority: vault,
    //     amount: fundraiser_data.current_amount,
    // }.invoke_signed(&signer)?;

    // TODO:close fundraiser and contributor accounts?

    Ok(())
}