use pinocchio::{
    pubkey,
    pubkey::Pubkey
};


#[inline]
pub fn validate_pda(
    seeds: &[&[u8]],
    program_id: &Pubkey,
    address: &Pubkey,
) {

    let pda = pubkey::create_program_address(seeds, program_id).unwrap();
    assert!(address.eq(&pda))
}
