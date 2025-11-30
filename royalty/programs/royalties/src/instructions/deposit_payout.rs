use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::{RoyaltyListing, PayoutPool, ListingStatus};
use crate::errors::RoyaltiesError;

#[derive(Accounts)]
pub struct DepositPayout<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        seeds = [b"royalty_listing", creator.key().as_ref(), royalty_listing.nft_mint.as_ref()],
        bump = royalty_listing.bump,
        constraint = royalty_listing.creator == creator.key() @ RoyaltiesError::Unauthorized,
        constraint = royalty_listing.status == ListingStatus::Sold @ RoyaltiesError::ListingNotActive
    )]
    pub royalty_listing: Account<'info, RoyaltyListing>,

    #[account(
        init_if_needed,
        payer = creator,
        space = PayoutPool::LEN,
        seeds = [b"payout_pool", royalty_listing.key().as_ref()],
        bump
    )]
    pub payout_pool: Account<'info, PayoutPool>,

    /// Creator's USDC token account
    #[account(
        mut,
        constraint = creator_usdc.owner == creator.key()
    )]
    pub creator_usdc: Account<'info, TokenAccount>,

    /// Payout pool's USDC vault
    #[account(
        init_if_needed,
        payer = creator,
        token::mint = usdc_mint,
        token::authority = payout_pool,
    )]
    pub pool_vault: Account<'info, TokenAccount>,

    /// USDC mint
    pub usdc_mint: Account<'info, anchor_spl::token::Mint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<DepositPayout>, amount: u64) -> Result<()> {
    require!(amount > 0, RoyaltiesError::InvalidPrice);

    let clock = Clock::get()?;

    // Transfer USDC to pool vault
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.creator_usdc.to_account_info(),
                to: ctx.accounts.pool_vault.to_account_info(),
                authority: ctx.accounts.creator.to_account_info(),
            },
        ),
        amount,
    )?;

    // Update payout pool
    let pool = &mut ctx.accounts.payout_pool;
    
    // If this is a new period, reset tracking
    if pool.total_deposited == pool.total_claimed || pool.total_deposited == 0 {
        pool.period = pool.period.checked_add(1).ok_or(RoyaltiesError::Overflow)?;
        pool.total_deposited = 0;
        pool.total_claimed = 0;
    }

    pool.royalty_listing = ctx.accounts.royalty_listing.key();
    pool.creator = ctx.accounts.creator.key();
    pool.total_deposited = pool
        .total_deposited
        .checked_add(amount)
        .ok_or(RoyaltiesError::Overflow)?;
    pool.deposited_at = clock.unix_timestamp;
    pool.bump = ctx.bumps.payout_pool;

    msg!(
        "Deposited {} USDC for payout period {}",
        amount as f64 / 1_000_000.0,
        pool.period
    );

    Ok(())
}

