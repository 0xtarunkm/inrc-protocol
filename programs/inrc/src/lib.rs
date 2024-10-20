pub mod constants;
pub mod contexts;
pub mod error;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use contexts::*;
pub use state::*;

declare_id!("Az9osrJiK6y3gRhnBNuBL6TVyM4QNWJjmHytWLMFvaEc");

#[program]
pub mod inrc {
    use super::*;

    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        liquidation_threshold: u64,
        liquidation_bonus: u64,
        min_health_factor: u64,
    ) -> Result<()> {
        ctx.accounts.init_config(
            liquidation_threshold,
            liquidation_bonus,
            min_health_factor,
            &ctx.bumps,
        )
    }
}
