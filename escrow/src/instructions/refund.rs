use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program::invoke_signed,
    pubkey::Pubkey,
    program_error::ProgramError,
    program_pack::Pack,
    system_program,
};
use spl_token::{instruction::{close_account, transfer_checked}, state::Mint};

use crate::state::Escrow;

pub fn refund(
    program_id: &Pubkey, 
    accounts: &[AccountInfo]
) -> ProgramResult {

    let [
        maker,
        mint_a,
        escrow,
        maker_ta_a,
        vault,
        token_program,
        system_program
     ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // check that system program is correct
    // possibly could leave that out -- why?
    assert!(system_program::check_id(system_program.key));

    // check that token program is correct
    assert!(spl_token::check_id(token_program.key));

    // check that own program id is correct
    assert!(crate::check_id(program_id));

    // check that maker is signer
    assert!(maker.is_signer);

    assert!(maker.is_writable);

    // assert_eq!(mint_a.owner, token_program.key);
    // assert_eq!(mint_ta_a.owner, token_program.key);
    // assert_eq!(vault.owner, system_program.key); // because vault was not created yet?

    // assert!(vault.is_writable); // berg didn't check this ?

    let mint_a_decimals = Mint::unpack(&mint_a.try_borrow_data()?)?.decimals;

    // let escrow_seeds = &[b"escrow", maker.key.as_ref(), &[args.escrow_bump]];
    // let expected_escrow = Pubkey::create_program_address(escrow_seeds, program_id).unwrap();
    // assert_eq!(&expected_escrow, escrow.key);

    // assert!(validate_escrow_seeds(escrow_seeds, escrow.key));
    // let expected_escrow = Pubkey::create_program_address(escrow_seeds, program_id).unwrap(); // save cu by using this instead of find_program_address
    // assert_eq!(&expected_escrow, escrow.key);
    // assert_eq!(escrow.owner, program_id);

    let escrow_data = *bytemuck::try_from_bytes_mut::<Escrow>(*escrow.data.borrow_mut()).map_err(|_| ProgramError::AccountBorrowFailed)?;
    let escrow_seeds = &[b"escrow", maker.key.as_ref(), &[escrow_data.bump as u8]];

    // Transfer A from vault to taker_ta_a
    invoke_signed(
        &transfer_checked(
           token_program.key,
           vault.key,
           mint_a.key,
           maker_ta_a.key,
           escrow.key,
           &[],
           escrow_data.receive,
           mint_a_decimals,
        )?,
        accounts,
        &[escrow_seeds]
    )?;

    // close escrow
    let mut escrow_data = escrow.data.borrow_mut();
    escrow_data.fill(0);
    let maker_orig_lamports = maker.lamports();
    **maker.lamports.borrow_mut() = maker_orig_lamports.checked_add(escrow.lamports()).ok_or(ProgramError::ArithmeticOverflow)?;
    **escrow.lamports.borrow_mut() = 0;

    // close vault
    invoke_signed(
        &close_account(
            token_program.key,
            vault.key,
            maker.key,
            escrow.key,
            &[],
        )?,
        accounts,
        &[escrow_seeds]
    )?;

    Ok(())
}