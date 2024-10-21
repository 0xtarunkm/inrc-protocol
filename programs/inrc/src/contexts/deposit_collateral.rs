use std::u64;

use anchor_lang::{
    prelude::*,
    solana_program::native_token::LAMPORTS_PER_SOL,
    system_program::{transfer, Transfer},
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{mint_to, Mint, MintTo, TokenAccount, TokenInterface},
};
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};

use crate::error::ErrorCode;
use crate::{
    Collateral, Config, ANCHOR_DISCRIMINATOR, MAX_AGE, PRICE_FEED_DECIMAL_ADJUSTMENT,
    SEED_COLLATERAL_ACCOUNT, SEED_CONFIG_ACCOUNT, SEED_MINT_ACCOUNT, SEED_SOL_ACCOUNT,
    SOL_USD_FEED_ID,
};

#[derive(Accounts)]
pub struct DepositCollateral<'info> {
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
        init,
        payer = depositor,
        seeds = [SEED_COLLATERAL_ACCOUNT, depositor.key().as_ref()],
        bump,
        space = ANCHOR_DISCRIMINATOR + Collateral::INIT_SPACE
    )]
    treasury: Account<'info, Collateral>,
    #[account(
        mut,
        seeds = [SEED_SOL_ACCOUNT, depositor.key().as_ref()],
        bump
    )]
    sol_treasury: SystemAccount<'info>,
    #[account(
        init_if_needed,
        payer = depositor,
        associated_token::mint = mint,
        associated_token::authority = depositor,
        associated_token::token_program = token_program,
    )]
    mint_ata: InterfaceAccount<'info, TokenAccount>,
    price_update: Account<'info, PriceUpdateV2>,

    token_program: Interface<'info, TokenInterface>,
    associated_token_program: Program<'info, AssociatedToken>,
    system_program: Program<'info, System>,
}

impl<'info> DepositCollateral<'info> {
    pub fn deposit_collateral(
        &mut self,
        collateral_amount: u64,
        mint_amount: u64,
        bumps: &DepositCollateralBumps,
    ) -> Result<()> {
        self.treasury.lamport_balance = self
            .sol_treasury
            .lamports()
            .checked_add(collateral_amount)
            .unwrap();
        self.treasury
            .amount_minted
            .checked_add(mint_amount)
            .unwrap();

        if !self.treasury.is_initialized {
            self.treasury.is_initialized = true;
            self.treasury.depositor = self.depositor.key();
            self.treasury.sol_account = self.sol_treasury.key();
            self.treasury.token_ata = self.mint_ata.key();
            self.treasury.bump = bumps.treasury;
            self.treasury.sol_account_bump = bumps.sol_treasury;
        }

        // check health factor
        let health_factor = self.calculate_health_factor(collateral_amount)?;
        require!(
            health_factor >= self.config.min_health_factor,
            ErrorCode::BelowMinHealthFactor
        );

        self.deposit_sol(collateral_amount)?;
        self.mint_token(mint_amount)
    }

    fn mint_token(&mut self, amount: u64) -> Result<()> {
        let cpi_account = MintTo {
            mint: self.mint.to_account_info(),
            to: self.mint_ata.to_account_info(),
            authority: self.mint.to_account_info(),
        };

        let seeds = &[&SEED_MINT_ACCOUNT[..], &[self.config.mint_pda_bump]];
        let signer_seeds = &[&seeds[..]];

        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            cpi_account,
            signer_seeds,
        );

        mint_to(ctx, amount)
    }

    fn deposit_sol(&mut self, amount: u64) -> Result<()> {
        let cpi_accounts = Transfer {
            from: self.depositor.to_account_info(),
            to: self.sol_treasury.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(self.system_program.to_account_info(), cpi_accounts);

        transfer(cpi_ctx, amount)
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
