use crate::state::Fundraiser;
use pinocchio::{
    program_error::ProgramError,
    ProgramResult,
    account_info::AccountInfo,
    sysvars::rent::Rent,
    sysvars::Sysvar,
    instruction::{Seed, Signer},
    pubkey
};
use pinocchio_system::instructions::CreateAccount;

use crate::processor::Initialize;

pub fn initialize(
    accounts: &[AccountInfo],
    args: &[u8]
) -> ProgramResult {
    let [
        maker, // person who is starting the fundraiser (signer)
        mint_to_raise, // the mint that the maker wants to receive
        fundraiser, // the PDA that will hold the fundraiser data (seeds: [b"fundraiser", maker.key().as_ref()] + bump, payer: maker, space)
        _system_program, 
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys)
    };

    assert!(maker.is_signer());

    let Initialize {
        amount, // the target amount that the maker is trying to raise
        duration, // the timeframe to collect all the contributions (in days)
        fundraiser_bump, // since our Fundraiser account will be a PDA (Program Derived Address), we will pass the bump of the account
    } = Initialize::try_from(args)?;

    // check derived address and get bump
    let (_, bump) = pubkey::find_program_address(&[
        b"fundraiser", 
        maker.key().as_ref(), 
    ], &crate::ID);
    assert_eq!(fundraiser_bump, bump as u64);

    let bump_binding = bump.to_le_bytes();
    let signer_seeds = [
        Seed::from(b"fundraiser"), 
        Seed::from(maker.key().as_ref()), 
        Seed::from(bump_binding.as_ref())
    ];
    let signer = [Signer::from(&signer_seeds)];

    // calculate space
    let space = core::mem::size_of::<Fundraiser>();

    // get minimum balance (rent)
    let rent = Rent::get()?;
    let minimum_balance = rent.minimum_balance(space);

    // create the fundraiser account
    CreateAccount{
        from: maker,
        to: fundraiser,
        lamports: minimum_balance,
        space: space as u64,
        owner: &crate::ID,
    }
    .invoke_signed(&signer)?;
    

    // assign fundraiser data
    let mut binding = fundraiser.try_borrow_mut_data()?;
        
    let binding = binding.as_mut();
    
    let fundraiser_account = bytemuck::try_from_bytes_mut::<Fundraiser>(binding)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    *fundraiser_account = Fundraiser {
        maker: *maker.key(),
        mint_to_raise: *mint_to_raise.key(),
        amount_to_raise: amount,
        current_amount: 0,
        time_started: 0,
        duration,
        bump: bump as u64,
    };

    Ok(())
}