// Example Solana program for Truent analysis
// This demonstrates how Truent analyzes Rust-based Solana programs

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

// Account data structure
#[derive(Default)]
pub struct TokenAccount {
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
    pub delegate: Option<Pubkey>,
    pub delegated_amount: u64,
}

// Entry point for transfer instruction
#[entrypoint]
pub fn process_transfer(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    
    let source = next_account_info(account_iter)?;
    let destination = next_account_info(account_iter)?;
    let authority = next_account_info(account_iter)?;
    
    // Example: validate authority signature
    if !authority.is_signer {
        return Err(ProgramError::MissingSignerPermission);
    }
    
    // Parse instruction data (simplified)
    if instruction_data.len() < 8 {
        return Err(ProgramError::InvalidInstructionData);
    }
    
    let amount = u64::from_le_bytes([
        instruction_data[0], instruction_data[1],
        instruction_data[2], instruction_data[3],
        instruction_data[4], instruction_data[5],
        instruction_data[6], instruction_data[7],
    ]);
    
    msg!("Transferring {} tokens", amount);
    
    // Mutate state (simplified - normally use account deserialization)
    // source.amount -= amount;
    // destination.amount += amount;
    
    Ok(())
}

// Additional helper function
pub fn validate_owner(account: &AccountInfo, owner: &Pubkey) -> ProgramResult {
    if account.owner != owner {
        return Err(ProgramError::IllegalOwner);
    }
    Ok(())
}
