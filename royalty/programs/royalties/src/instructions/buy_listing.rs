use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, MintTo};
use anchor_spl::associated_token::AssociatedToken;
use crate::state::{RoyaltyListing, ListingStatus, PlatformConfig};
use crate::errors::RoyaltiesError;

#[derive(Accounts)]
pub struct BuyListing<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    /// CHECK: Creator receiving payment
    #[account(mut)]
    pub creator: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"platform_config"],
        bump = platform_config.bump
    )]
    pub platform_config: Account<'info, PlatformConfig>,

    /// CHECK: Platform treasury receiving fees
    #[account(mut, address = platform_config.treasury)]
    pub treasury: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"royalty_listing", creator.key().as_ref(), nft_mint.key().as_ref()],
        bump = royalty_listing.bump,
        constraint = royalty_listing.status == ListingStatus::Active @ RoyaltiesError::ListingNotActive,
        constraint = royalty_listing.creator == creator.key() @ RoyaltiesError::Unauthorized
    )]
    pub royalty_listing: Account<'info, RoyaltyListing>,

    #[account(
        mut,
        constraint = nft_mint.key() == royalty_listing.nft_mint @ RoyaltiesError::Unauthorized
    )]
    pub nft_mint: Account<'info, Mint>,

    /// Buyer's USDC token account
    #[account(
        mut,
        constraint = buyer_usdc.owner == buyer.key(),
        constraint = buyer_usdc.mint == usdc_mint.key()
    )]
    pub buyer_usdc: Account<'info, TokenAccount>,

    /// Creator's USDC token account
    #[account(
        mut,
        constraint = creator_usdc.owner == creator.key(),
        constraint = creator_usdc.mint == usdc_mint.key()
    )]
    pub creator_usdc: Account<'info, TokenAccount>,

    /// Treasury's USDC token account
    #[account(
        mut,
        constraint = treasury_usdc.owner == treasury.key(),
        constraint = treasury_usdc.mint == usdc_mint.key()
    )]
    pub treasury_usdc: Account<'info, TokenAccount>,

    /// USDC mint
    pub usdc_mint: Account<'info, Mint>,

    /// Buyer's NFT token account (will receive the royalty NFT)
    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = nft_mint,
        associated_token::authority = buyer,
    )]
    pub buyer_nft: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<BuyListing>) -> Result<()> {
    let listing = &ctx.accounts.royalty_listing;
    let config = &ctx.accounts.platform_config;

    // Calculate fee split
    let total_price = listing.price;
    let platform_fee = total_price
        .checked_mul(config.platform_fee_bps as u64)
        .ok_or(RoyaltiesError::Overflow)?
        .checked_div(10000)
        .ok_or(RoyaltiesError::Overflow)?;
    let creator_amount = total_price
        .checked_sub(platform_fee)
        .ok_or(RoyaltiesError::Overflow)?;

    // Transfer USDC to creator
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.buyer_usdc.to_account_info(),
                to: ctx.accounts.creator_usdc.to_account_info(),
                authority: ctx.accounts.buyer.to_account_info(),
            },
        ),
        creator_amount,
    )?;

    // Transfer platform fee to treasury
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.buyer_usdc.to_account_info(),
                to: ctx.accounts.treasury_usdc.to_account_info(),
                authority: ctx.accounts.buyer.to_account_info(),
            },
        ),
        platform_fee,
    )?;

    // Mint NFT to buyer
    let creator_key = ctx.accounts.creator.key();
    let nft_mint_key = ctx.accounts.nft_mint.key();
    let seeds = &[
        b"royalty_listing",
        creator_key.as_ref(),
        nft_mint_key.as_ref(),
        &[listing.bump],
    ];
    let signer_seeds = &[&seeds[..]];

    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.nft_mint.to_account_info(),
                to: ctx.accounts.buyer_nft.to_account_info(),
                authority: ctx.accounts.royalty_listing.to_account_info(),
            },
            signer_seeds,
        ),
        1, // Mint 1 NFT
    )?;

    // Update listing status
    let listing = &mut ctx.accounts.royalty_listing;
    listing.status = ListingStatus::Sold;

    // Update platform stats
    let config = &mut ctx.accounts.platform_config;
    config.total_fees_collected = config
        .total_fees_collected
        .checked_add(platform_fee)
        .ok_or(RoyaltiesError::Overflow)?;

    msg!(
        "Purchase complete: {} USDC (fee: {} USDC)",
        total_price as f64 / 1_000_000.0,
        platform_fee as f64 / 1_000_000.0
    );

    Ok(())
}

