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
use crate::utils::validate_pda;

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
        vault_bump,
    } = CheckerArgs::try_from(args)?;

    let u8_fundraiser_bump = fundraiser_bump as u8;
    let fundraiser_bump_bytes = u8_fundraiser_bump.to_le_bytes();
    validate_pda(&[
        b"fundraiser",
        maker.key().as_ref(),
        &fundraiser_bump_bytes
    ], &crate::ID, fundraiser.key());

    let u8_vault_bump = vault_bump as u8;
    let vault_bump_bytes = u8_vault_bump.to_le_bytes();
    validate_pda(&[
        b"vault",
        fundraiser.key().as_ref(),
        &vault_bump_bytes
    ], &crate::ID, vault.key());

    let fundraiser_data: Fundraiser = *bytemuck::try_from_bytes::<Fundraiser>(&fundraiser.try_borrow_mut_data()?).map_err(|_| ProgramError::InvalidAccountData)?;

    // check if the fundraising goal has been reached
    assert!(fundraiser_data.current_amount >= fundraiser_data.amount_to_raise);
    
    
    let signer_seeds = [
        Seed::from(b"fundraiser"), 
        Seed::from(maker.key().as_ref()), 
        Seed::from(fundraiser_bump_bytes.as_ref())
    ];
    let signer = [Signer::from(&signer_seeds)];

    // transfer the funds to the maker
    Transfer {
        from: vault,
        to: maker_ata,
        authority: vault,
        amount: fundraiser_data.current_amount,
    }.invoke_signed(&signer)?;

    // TODO:close fundraiser and contributor accounts?

    Ok(())
}