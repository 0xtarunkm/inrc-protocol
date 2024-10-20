use anchor_lang::prelude::*;

#[constant]
pub const SEED_CONFIG_ACCOUNT: &[u8] = b"config";
pub const SEED_MINT_ACCOUNT: &[u8] = b"mint";

pub const ANCHOR_DISCRIMINATOR: usize = 8;
pub const MINT_DECIMAL: u8 = 6;
