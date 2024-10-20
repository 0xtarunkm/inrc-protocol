pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("Az9osrJiK6y3gRhnBNuBL6TVyM4QNWJjmHytWLMFvaEc");

#[program]
pub mod inrc {
    use super::*;
}
