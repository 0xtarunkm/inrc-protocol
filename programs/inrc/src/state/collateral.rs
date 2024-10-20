use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace, Debug)]
pub struct Collateral {
    pub depositor: Pubkey,
    pub sol_account: Pubkey,
    pub token_ata: Pubkey,
    pub lamport_balance: u64,
    pub amount_minted: u64,
    pub bump: u8,
    pub sol_account_bump: u8,
    pub is_initialized: bool,
}
