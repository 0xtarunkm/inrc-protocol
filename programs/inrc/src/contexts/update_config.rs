use anchor_lang::prelude::*;

use crate::{Config, SEED_CONFIG_ACCOUNT};

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    #[account(mut)]
    signer: Signer<'info>,
    #[account(
        mut,
        seeds = [SEED_CONFIG_ACCOUNT, signer.key().as_ref()],
        bump = config.bump
    )]
    config: Account<'info, Config>,
}

impl<'info> UpdateConfig<'info> {
    pub fn update_config(&mut self, min_health_factor: u64) -> Result<()> {
        self.config.min_health_factor = min_health_factor;

        Ok(())
    }
}
