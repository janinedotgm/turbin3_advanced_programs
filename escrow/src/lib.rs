use solana_program::{
    entrypoint,
    declare_id,
};
use processor::process_instruction;

mod tests;
mod state;
mod instructions;
mod processor;


// gives you the option to do check_id on the program id
declare_id!("22222222222222222222222222222222222222222222");

entrypoint!(process_instruction);
