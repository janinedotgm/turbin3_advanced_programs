
// pub fn contribute(
//     program_id: &Pubkey,
//     accounts: &[AccountInfo],
//     args: ContributeArgs
// ) -> ProgramResult {
//     let [
//         contributor, // who wants to contribute e.g. sending funds to the vault
//         mint_to_raise, // the mint that the fundraiser is raising
//         fundraiser, // the fundraiser account
//         contributor_account, // the contributor account to save the total amount contributed
//         contributor_ta, // the contributor token account
//         vault, // the vault account
//         token_program,
//         system_program,
//     ] = accounts
//     else {
//         return Err(ProgramError::NotEnoughAccountKeys)
//     };

//     assert!(system_program::check_id(system_program.key));
//     assert!(crate::check_id(program_id));
//     assert!(contributor.is_signer);

//     assert_eq!(mint_to_raise.key, mint_to_raise.key);

//     let mint_data: Mint = Mint::unpack(&mint_to_raise.try_borrow_data()?)?;

//     let mut fundraiser_data = *bytemuck::try_from_bytes_mut::<Fundraiser>(*fundraiser.data.borrow_mut())
//             .map_err(|_| ProgramError::AccountBorrowFailed)?;

//     // Contribution checks
//     // [x] min amount
//     // [x] max amount
//     // [x] round is still active
//     // [x] check that the contributor has not reached the max per contribution
//     // [] check the amount to raise has not been reached

//     // check min amount
//     if args.amount < 1_u64.pow(mint_data.decimals as u32) {
//         return Err(ProgramError::InvalidArgument);
//     }

//     // check amount is less than amount to raise
//     if args.amount > fundraiser_data.amount_to_raise {
//         return Err(ProgramError::InvalidArgument);
//     }

//     // check round is still active
//     let current_time = Clock::get()?.unix_timestamp;
//     if fundraiser_data.duration > ((current_time - fundraiser_data.time_started) / SECONDS_TO_DAYS) as u64 {
//         return Err(ProgramError::InvalidArgument); // Fundraiser has ended
//     }



//     // check if contributor_account already exists
//     let contributor_seeds = &[b"contributor", contributor.key.as_ref(), &[args.contributor_bump]];
//     if contributor_account.owner == system_program.key {
//         // create new contributor account
//         invoke_signed(
//             &system_instruction::create_account(
//                 contributor.key,
//                 contributor_account.key,
//                 Rent::get()?.minimum_balance(Fundraiser::LEN),
//                 Fundraiser::LEN as u64,
//                 &crate::id(),
//             ),
//             accounts,
//             &[contributor_seeds]
//         )?;
//     }

//     // check that the contributor has not reached the max per contribution
//     let contributor_data = *bytemuck::try_from_bytes::<Contributor>(*contributor_account.data.borrow())
//         .map_err(|_| ProgramError::AccountBorrowFailed)?;

//     if contributor_data.amount + args.amount > (fundraiser_data.amount_to_raise * MAX_CONTRIBUTION_PERCENTAGE) / PERCENTAGE_SCALER {
//         return Err(ProgramError::InvalidArgument); // Contributor has reached the max per contribution
//     }
    

//     invoke(
//         &transfer_checked(
//             token_program.key, 
//             contributor_ta.key, 
//             mint_to_raise.key, 
//             vault.key,
//             contributor.key, 
//             &[contributor.key], 
//             args.amount, 
//             mint_data.decimals, 
//         )?,
//         accounts
//     )?;

//     fundraiser_data.current_amount += args.amount;
//     let fundraiser_data_serialized = bytemuck::bytes_of(&fundraiser_data);
//     fundraiser.data.borrow_mut().copy_from_slice(fundraiser_data_serialized);
    

//     Ok(())
// }