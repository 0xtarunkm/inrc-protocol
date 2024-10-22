use anchor_lang::prelude::*;
use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;
use anchor_lang::system_program::{transfer, Transfer};
use anchor_spl::token_interface::{burn, Burn, Mint, TokenAccount, TokenInterface};
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};

use crate::error::ErrorCode;
use crate::{
    Collateral, Config, MAX_AGE, PRICE_FEED_DECIMAL_ADJUSTMENT, SEED_COLLATERAL_ACCOUNT, SEED_CONFIG_ACCOUNT, SEED_SOL_ACCOUNT, SOL_USD_FEED_ID
};

#[derive(Accounts)]
pub struct WithdrawCollateral<'info> {
    #[account(mut)]
    depositor: Signer<'info>,
    #[account(
        mut,
        seeds = [SEED_CONFIG_ACCOUNT],
        bump = config.bump,
        has_one = mint,
    )]
    config: Account<'info, Config>,
    #[account(mut)]
    mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        seeds = [SEED_COLLATERAL_ACCOUNT, depositor.key().as_ref()],
        bump = treasury.bump,
        has_one = mint_ata,
        has_one = sol_treasury
    )]
    treasury: Account<'info, Collateral>,
    #[account(mut)]
    sol_treasury: SystemAccount<'info>,
    #[account(mut)]
    mint_ata: InterfaceAccount<'info, TokenAccount>,
    price_update: Account<'info, PriceUpdateV2>,

    token_program: Interface<'info, TokenInterface>,
    system_program: Program<'info, System>,
}

impl<'info> WithdrawCollateral<'info> {
    pub fn redeem_collateral(&mut self, collateral_amount: u64, burn_amount: u64) -> Result<()> {
        self.treasury.lamport_balance = self.sol_treasury.lamports().checked_sub(collateral_amount).unwrap();
        self.treasury.amount_minted.checked_sub(burn_amount).unwrap();

        // check health factor
        let health_factor = self.calculate_health_factor(collateral_amount)?;
        require!(
            health_factor >= self.config.min_health_factor,
            ErrorCode::BelowMinHealthFactor
        );

        self.burn_token(burn_amount)?;
        self.withdraw_sol(collateral_amount)
    }

    fn withdraw_sol(&mut self, amount: u64) -> Result<()> {
        let cpi_accounts = Transfer {
            from: self.sol_treasury.to_account_info(),
            to: self.depositor.to_account_info(),
        };

        let binding = self.depositor.key();
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
            authority: self.depositor.to_account_info(),
            mint: self.mint.to_account_info(),
            from: self.mint_ata.to_account_info(),
        };

        let ctx = CpiContext::new(self.token_program.to_account_info(), cpi_account);

        burn(ctx, amount)
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
