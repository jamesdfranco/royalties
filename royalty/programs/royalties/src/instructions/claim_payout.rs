use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::{RoyaltyListing, PayoutPool, PayoutClaim};
use crate::errors::RoyaltiesError;

#[derive(Accounts)]
pub struct ClaimPayout<'info> {
    #[account(mut)]
    pub holder: Signer<'info>,

    #[account(
        seeds = [b"royalty_listing", royalty_listing.creator.as_ref(), royalty_listing.nft_mint.as_ref()],
        bump = royalty_listing.bump
    )]
    pub royalty_listing: Account<'info, RoyaltyListing>,

    #[account(
        mut,
        seeds = [b"payout_pool", royalty_listing.key().as_ref()],
        bump = payout_pool.bump
    )]
    pub payout_pool: Account<'info, PayoutPool>,

    #[account(
        init,
        payer = holder,
        space = PayoutClaim::LEN,
        seeds = [
            b"payout_claim",
            payout_pool.key().as_ref(),
            holder.key().as_ref(),
            &payout_pool.period.to_le_bytes()
        ],
        bump
    )]
    pub payout_claim: Account<'info, PayoutClaim>,

    /// Holder's NFT account - proves ownership
    #[account(
        constraint = holder_nft.owner == holder.key() @ RoyaltiesError::NotOwner,
        constraint = holder_nft.mint == royalty_listing.nft_mint @ RoyaltiesError::Unauthorized,
        constraint = holder_nft.amount == 1 @ RoyaltiesError::NotOwner
    )]
    pub holder_nft: Account<'info, TokenAccount>,

    /// Pool vault holding USDC
    #[account(
        mut,
        constraint = pool_vault.owner == payout_pool.key()
    )]
    pub pool_vault: Account<'info, TokenAccount>,

    /// Holder's USDC account to receive payout
    #[account(
        mut,
        constraint = holder_usdc.owner == holder.key()
    )]
    pub holder_usdc: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<ClaimPayout>) -> Result<()> {
    let pool = &ctx.accounts.payout_pool;

    // Check there's something to claim
    let available = pool
        .total_deposited
        .checked_sub(pool.total_claimed)
        .ok_or(RoyaltiesError::Overflow)?;
    require!(available > 0, RoyaltiesError::PayoutPoolEmpty);

    // Calculate holder's share
    // Since each NFT represents the full percentage, the holder gets the full available amount
    // In a fractional system, this would be proportional to tokens held
    let claim_amount = available;

    let clock = Clock::get()?;

    // Transfer from pool vault to holder
    let royalty_listing_key = ctx.accounts.royalty_listing.key();
    let seeds = &[
        b"payout_pool",
        royalty_listing_key.as_ref(),
        &[pool.bump],
    ];
    let signer_seeds = &[&seeds[..]];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.pool_vault.to_account_info(),
                to: ctx.accounts.holder_usdc.to_account_info(),
                authority: ctx.accounts.payout_pool.to_account_info(),
            },
            signer_seeds,
        ),
        claim_amount,
    )?;

    // Get pool key before mutable borrow
    let payout_pool_key = ctx.accounts.payout_pool.key();
    let pool_period = ctx.accounts.payout_pool.period;

    // Update pool
    let pool = &mut ctx.accounts.payout_pool;
    pool.total_claimed = pool
        .total_claimed
        .checked_add(claim_amount)
        .ok_or(RoyaltiesError::Overflow)?;

    // Record claim
    let claim = &mut ctx.accounts.payout_claim;
    claim.payout_pool = payout_pool_key;
    claim.holder = ctx.accounts.holder.key();
    claim.amount_claimed = claim_amount;
    claim.claimed_at = clock.unix_timestamp;
    claim.bump = ctx.bumps.payout_claim;

    msg!(
        "Claimed {} USDC for period {}",
        claim_amount as f64 / 1_000_000.0,
        pool_period
    );

    Ok(())
}

