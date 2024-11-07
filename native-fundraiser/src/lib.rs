use processor::process_instruction;
use pinocchio::entrypoint;
use five8_const;

mod processor;
mod instructions;
mod state;
mod tests;
mod constants;

pub const ID: [u8; 32] =
    five8_const::decode_32_const("22222222222222222222222222222222222222222222");

entrypoint!(process_instruction);



