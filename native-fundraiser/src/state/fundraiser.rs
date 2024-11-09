use pinocchio::pubkey::Pubkey;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Fundraiser {
    pub maker: Pubkey,
    pub mint_to_raise: Pubkey,
    pub amount_to_raise: u64,
    pub current_amount: u64,
    pub time_started: i64,
    pub duration: i64,
    pub bump: u64,
}

impl Fundraiser {
    pub const LEN: usize = std::mem::size_of::<Fundraiser>();
}



