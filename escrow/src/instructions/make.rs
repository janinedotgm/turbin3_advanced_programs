use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_pack::Pack,
    rent::Rent,
    system_instruction,
    system_program,
    sysvar::Sysvar,
    pubkey::Pubkey,
    program_error::ProgramError
};
use spl_token::{instruction::transfer_checked, state::Mint};

use crate::{processor::EscrowArgs, state::Escrow};

pub fn make(
    program_id: &Pubkey, 
    accounts: &[AccountInfo], 
    args: EscrowArgs,
) -> ProgramResult {

    let [
        maker, 
        mint_a,
        mint_b, // pubkey is enought, we don't need token info
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

    // check that token program is correct - not needed is already performed?
    // assert!(spl_token::check_id(token_program.key)); 

    // check that own program id is correct
    assert!(crate::check_id(program_id));

    // check that maker is signer
    assert!(maker.is_signer);

    assert!(maker.is_writable);

    // assert_eq!(mint_a.owner, token_program.key); no neeed
    // assert_eq!(mint_b.owner, token_program.key); no neeed
    // assert_eq!(mint_ta_a.owner, token_program.key);
    assert_eq!(vault.owner, system_program.key); // because vault was not created yet?

    // assert!(vault.is_writable); // berg didn't check this ?

    let mint_unpacked = Mint::unpack(&mint_a.try_borrow_data()?)?;

    assert!(escrow.is_writable && escrow.data_is_empty());
    let escrow_seeds = &[b"escrow", maker.key.as_ref(), &[args.escrow_bump]];

    invoke_signed(
        &system_instruction::create_account(
            maker.key, //from_pubkey: 
            escrow.key, //to_pubkey,
            Rent::get()?.minimum_balance(Escrow::LEN), //lamports,
            Escrow::LEN as u64, //space,
            &crate::id()//program_id
        ),
        accounts, // account_infos????
       &[escrow_seeds], // signer seeds
    )?;

    let new_escrow = Escrow{
        maker: *maker.key,
        mint_a: *mint_a.key,
        mint_b: *mint_b.key,
        receive: args.receive,
        bump: args.escrow_bump as u64,
    };

    let mut escrow_data = *bytemuck::try_from_bytes_mut::<Escrow>(*escrow.data.borrow_mut())
        .map_err(|_| ProgramError::AccountBorrowFailed)?;

    escrow_data.clone_from(&new_escrow);

    // Transfer to vault
    invoke(
        &transfer_checked(
           token_program.key,
           maker_ta_a.key,
           mint_a.key,
           vault.key,
           maker.key,
           &[],
           args.amount,
           mint_unpacked.decimals,
        )?,
        &[
            maker.clone(),
            maker_ta_a.clone(),
            vault.clone(),
        ]
    )?;

    Ok(())
}