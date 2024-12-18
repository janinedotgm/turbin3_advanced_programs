use bytemuck::{Pod, Zeroable};
use pinocchio::{
    program_error::ProgramError, 
    ProgramResult,
    pubkey::Pubkey,
    account_info::AccountInfo,
};

use crate::instructions::*;

pub enum FundraiserInstructions {
    Initialize,
    Contribute,
    Checker,
    Refund,
}

impl TryFrom<&u8> for FundraiserInstructions {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Initialize),
            1 => Ok(Self::Contribute),
            2 => Ok(Self::Checker),
            3 => Ok(Self::Refund),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Pod, Zeroable)]
pub struct InitializeArgs {
    pub amount: u64,
    pub duration: i64,
    pub fundraiser_bump: u64,
}

impl TryFrom<&[u8]> for InitializeArgs {
    
    type Error = ProgramError;
    
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        bytemuck::try_pod_read_unaligned::<Self>(data)
            .map_err(|_| ProgramError::InvalidInstructionData)
    }
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Pod, Zeroable)]
pub struct ContributeArgs {
    pub amount: u64,
    pub vault_bump: u64,
}

impl TryFrom<&[u8]> for ContributeArgs {
    
    type Error = ProgramError;
    
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        bytemuck::try_pod_read_unaligned::<Self>(data)
            .map_err(|_| ProgramError::InvalidInstructionData)
    }
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Pod, Zeroable)]
pub struct CheckerArgs {
    pub fundraiser_bump: u64,
    pub vault_bump: u64,
}

impl TryFrom<&[u8]> for CheckerArgs {
    
    type Error = ProgramError;
    
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        bytemuck::try_pod_read_unaligned::<Self>(data)
            .map_err(|_| ProgramError::InvalidInstructionData)
    }
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Pod, Zeroable)]
pub struct RefundArgs {
    pub fundraiser_bump: u64,
    pub vault_bump: u64,
}

impl TryFrom<&[u8]> for RefundArgs {
    
    type Error = ProgramError;
    
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        bytemuck::try_pod_read_unaligned::<Self>(data)
            .map_err(|_| ProgramError::InvalidInstructionData)
    }
}

pub fn process_instruction(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {

    if program_id.ne(&crate::ID) {
        return Err(ProgramError::IncorrectProgramId);
    }

    let (discriminator, data) = instruction_data.split_first().ok_or(ProgramError::InvalidInstructionData)?;

    match FundraiserInstructions::try_from(discriminator)? {
        FundraiserInstructions::Initialize => initialize(accounts, data),
        FundraiserInstructions::Contribute => contribute(accounts, data),
        FundraiserInstructions::Checker => checker(accounts, data),
        FundraiserInstructions::Refund => refund(accounts, data)
    }
}
