use anchor_lang::{
    prelude::*,
    solana_program::native_token::LAMPORTS_PER_SOL,
    system_program::{transfer, Transfer},
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{burn, Burn, Mint, TokenAccount, TokenInterface},
};
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};

use crate::{
    error::ErrorCode, MAX_AGE, PRICE_FEED_DECIMAL_ADJUSTMENT, SEED_SOL_ACCOUNT, SOL_USD_FEED_ID,
};
use crate::{Collateral, Config, SEED_CONFIG_ACCOUNT};

#[derive(Accounts)]
pub struct Liquidate<'info> {
    #[account(mut)]
    liquidator: Signer<'info>,
    #[account(mut)]
    mint: InterfaceAccount<'info, Mint>,
    #[account(
        seeds = [SEED_CONFIG_ACCOUNT],
        bump = config.bump,
        has_one = mint
    )]
    config: Account<'info, Config>,
    #[account(
        mut,
        has_one = sol_treasury
    )]
    treasury: Account<'info, Collateral>,
    #[account(mut)]
    sol_treasury: SystemAccount<'info>,
    #[account(
        init_if_needed,
        payer = liquidator,
        associated_token::mint = mint,
        associated_token::authority = liquidator,
        associated_token::token_program = token_program,
    )]
    liquidator_ata: InterfaceAccount<'info, TokenAccount>,
    price_update: Account<'info, PriceUpdateV2>,

    token_program: Interface<'info, TokenInterface>,
    associated_token_program: Program<'info, AssociatedToken>,
    system_program: Program<'info, System>,
}

impl<'info> Liquidate<'info> {
    pub fn liquidate(&mut self, amount_to_burn: u64) -> Result<()> {
        // check health factor
        let health_factor = self.calculate_health_factor(self.treasury.amount_minted)?;
        require!(
            health_factor < self.config.min_health_factor,
            ErrorCode::AboveMinHealthFactor
        );

        let lamports = self.get_lamports_from_usd(&amount_to_burn)?;
        let liquidation_bonus = (lamports * self.config.liquidation_bonus) / 100;
        let amount_to_liquidate = lamports + liquidation_bonus;

        self.withdraw_sol(amount_to_liquidate)?;
        self.burn_token(amount_to_burn)?;

        self.treasury.lamport_balance = self.sol_treasury.lamports();
        self.treasury.amount_minted -= amount_to_burn;

        Ok(())
    }

    fn withdraw_sol(&mut self, amount: u64) -> Result<()> {
        let cpi_accounts = Transfer {
            from: self.sol_treasury.to_account_info(),
            to: self.liquidator.to_account_info(),
        };

        let binding = self.treasury.depositor.key();
        let seeds = &[
            &SEED_SOL_ACCOUNT[..],
            &binding.as_ref(),
            &[self.treasury.sol_treasury_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            self.system_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );

        transfer(cpi_ctx, amount)
    }

    fn burn_token(&mut self, amount: u64) -> Result<()> {
        let cpi_account = Burn {
            authority: self.liquidator.to_account_info(),
            mint: self.mint.to_account_info(),
            from: self.liquidator_ata.to_account_info(),
        };

        let ctx = CpiContext::new(self.token_program.to_account_info(), cpi_account);

        burn(ctx, amount)
    }

    fn get_lamports_from_usd(&mut self, usd_amount: &u64) -> Result<u64> {
        let feed_id = get_feed_id_from_hex(SOL_USD_FEED_ID)?;
        let price = self
            .price_update
            .get_price_no_older_than(&Clock::get()?, MAX_AGE, &feed_id)?;

        require!(price.price > 0, ErrorCode::InvalidPrice);

        let price_in_usd = price.price as u128 * PRICE_FEED_DECIMAL_ADJUSTMENT;

        let amount_in_lamports = (*usd_amount as u128 * (LAMPORTS_PER_SOL as u128)) / price_in_usd;

        Ok(amount_in_lamports as u64)
    }

    fn calculate_health_factor(&mut self, collateral_amount: u64) -> Result<u64> {
        let feed_id = get_feed_id_from_hex(SOL_USD_FEED_ID)?;
        let price = self
            .price_update
            .get_price_no_older_than(&Clock::get()?, MAX_AGE, &feed_id)?;

        require!(price.price > 0, ErrorCode::InvalidPrice);

        let price_in_usd = price.price as u128 * PRICE_FEED_DECIMAL_ADJUSTMENT;

        let amount_in_usd = (collateral_amount as u128 * price_in_usd) / (LAMPORTS_PER_SOL as u128);

        let collateral_adjusted_for_liquidation_threshold =
            (amount_in_usd * self.config.liquidation_threshold as u128) / 100;

        if self.treasury.amount_minted == 0 {
            msg!("Health factor max");
            return Ok(u64::MAX);
        }

        let health_factor =
            collateral_adjusted_for_liquidation_threshold as u64 / self.treasury.amount_minted;

        Ok(health_factor)
    }
}
