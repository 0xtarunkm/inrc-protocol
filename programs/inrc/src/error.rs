use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid price from the feed")]
    InvalidPrice,
    #[msg("Below minimum health factor")]
    BelowMinHealthFactor,
}
