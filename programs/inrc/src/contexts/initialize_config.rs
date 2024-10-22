use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenInterface};

use crate::{
    Config, ANCHOR_DISCRIMINATOR, LIQUIDATION_BONUS, LIQUIDATION_THRESHOLD, MINT_DECIMAL,
    MIN_HEALTH_FACTOR, SEED_CONFIG_ACCOUNT, SEED_MINT_ACCOUNT,
};

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(mut)]
    signer: Signer<'info>,
    #[account(
        init,
        payer = signer,
        seeds = [SEED_CONFIG_ACCOUNT],
        bump,
        space = ANCHOR_DISCRIMINATOR + Config::INIT_SPACE,
    )]
    config: Account<'info, Config>,
    #[account(
        init,
        payer = signer,
        seeds = [SEED_MINT_ACCOUNT],
        bump,
        mint::decimals = MINT_DECIMAL,
        mint::authority = mint,
        mint::freeze_authority = mint,
        mint::token_program = token_program
    )]
    mint: InterfaceAccount<'info, Mint>,

    token_program: Interface<'info, TokenInterface>,
    system_program: Program<'info, System>,
}

impl<'info> InitializeConfig<'info> {
    pub fn init_config(&mut self, bumps: &InitializeConfigBumps) -> Result<()> {
        self.config.set_inner(Config {
            authority: self.signer.key(),
            mint: self.mint.key(),
            liquidation_threshold: LIQUIDATION_THRESHOLD,
            liquidation_bonus: LIQUIDATION_BONUS,
            min_health_factor: MIN_HEALTH_FACTOR,
            bump: bumps.config,
            mint_pda_bump: bumps.mint,
        });

        Ok(())
    }
}
