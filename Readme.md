# Solana Turbine3 Advanced Programs with Rust & Anchor

This repository contains my work for the Solana Turbine3 Advanced Programs with Rust & Anchor course. I will be adding my projects to this repository and documenting my progress here.

## Learnings / Notes / Cheatsheet

## Crates

### Native Rust
```rust
use solana_program::entrypoint;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey, hash::hashv
};
```

### Optimization Ideas
```rust
use five8_const::decode_32_const;
use pinocchio::{entrypoint, msg};
use pinocchio::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey
};
use solana_nostd_sha256::hashv;
```

### Notes


## Entrypoint

### Native Rust

### Optimization Ideas

### Notes


## IX Discriminator

### Native Rust

### Optimization Ideas

### Notes


## IX Serde

### Native 
```rust
let lamports: u64 = u64::from_le_bytes([
    data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
]);
```
### Optimized
```rust
let lamports: u64 = unsafe { *(data.as_ptr() as *const u64) };
```

### Notes


## Account Serde

### Native 

### Optimized

### Notes


## Account Checks

### Native

### Optimized

### Notes


## Balance Checks

### Native
```rust
**vault.try_borrow_mut_lamports()? -= lamports;
**signer.try_borrow_mut_lamports()? += lamports;
```

### Optimized
```rust
unsafe {
    *vault.borrow_mut_lamports_unchecked() -= lamports;
    *signer.borrow_mut_lamports_unchecked() += lamports;
}
```

### Notes


## CPIs 

### Native

### Optimized

### Notes


## Signer Checks

### Native

### Optimized

### Notes


## PDAs
Program Derived Addresses

### Native
```rust
use solana_program::hash::hashv

let pda = hashv(&[
    signer.key.as_ref(),
    &[bump],
    ID.as_ref(),
    PDA_MARKER,
]);
```

### Optimized
```rust
use solana_nostd_sha256::hashv;

let pda = hashv(&[
    signer.key().as_ref(),
    &[bump],
    ID.as_ref(),
    PDA_MARKER,
]);
```

### Notes


## Error Handling

### Native

### Optimized

### Notes
Programs return the result in the `r0` register. Similar to C the exit code 0 is a success and any other number is an error. Therefore we can add the error code to the `r0` register directly using assembly to return an error.
```rust
// Ensure signer had 0 bytes data length
if *(input.add(0x0058) as *const u64) != 0 {
   sol_log_("Invalid account length Signer".as_ptr(), 29);
   core::arch::asm!("lddw r0, 2"); // load immediate value 2 into r0 register
   return;  
};
```

## Testing

### Native

### Optimized

### Notes


## Projects

1. **PaulX Tutorial**
   - Completed the [PaulX tutorial](https://paulx.dev/blog/2021/01/14/programming-on-solana-an-introduction/), which covers the basics of programming on Solana using an escrow program as an example.

2. **Vault Program**
   - Currently working on a vault program. More details and progress updates will be added soon.

## To-Do List

- [x] Complete PaulX tutorial
- [ ] Make PaulX code better
- [ ] Make the Vault Program work
- [ ] Make the Vault Program better

## File Outline

- `paulx_escrow/`: Contains the code and notes from the PaulX tutorial.
- `vault_program/`: Work in progress for the vault program.
- `README.md`: This file, documenting the course progress and project details.

---

Feel free to reach out if you have any questions or suggestions regarding my projects or this repository.

