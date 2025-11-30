use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use anchor_spl::associated_token::AssociatedToken;
use crate::state::{RoyaltyListing, ResaleListing, PlatformConfig};
use crate::errors::RoyaltiesError;

#[derive(Accounts)]
pub struct BuyResale<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    /// CHECK: Seller receiving payment
    #[account(mut)]
    pub seller: UncheckedAccount<'info>,

    /// CHECK: Original creator receiving royalty
    #[account(mut)]
    pub creator: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"platform_config"],
        bump = platform_config.bump
    )]
    pub platform_config: Account<'info, PlatformConfig>,

    /// CHECK: Platform treasury
    #[account(mut, address = platform_config.treasury)]
    pub treasury: UncheckedAccount<'info>,

    #[account(
        seeds = [b"royalty_listing", creator.key().as_ref(), royalty_listing.nft_mint.as_ref()],
        bump = royalty_listing.bump
    )]
    pub royalty_listing: Account<'info, RoyaltyListing>,

    #[account(
        mut,
        seeds = [b"resale_listing", royalty_listing.key().as_ref(), seller.key().as_ref()],
        bump = resale_listing.bump,
        constraint = resale_listing.seller == seller.key() @ RoyaltiesError::Unauthorized,
        close = seller
    )]
    pub resale_listing: Account<'info, ResaleListing>,

    /// Escrow holding the NFT
    #[account(
        mut,
        constraint = escrow_nft.owner == resale_listing.key() @ RoyaltiesError::Unauthorized,
        constraint = escrow_nft.amount == 1 @ RoyaltiesError::NotOwner
    )]
    pub escrow_nft: Account<'info, TokenAccount>,

    /// Buyer's NFT account
    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = nft_mint,
        associated_token::authority = buyer,
    )]
    pub buyer_nft: Account<'info, TokenAccount>,

    /// CHECK: NFT mint
    pub nft_mint: UncheckedAccount<'info>,

    /// Buyer's USDC
    #[account(
        mut,
        constraint = buyer_usdc.owner == buyer.key()
    )]
    pub buyer_usdc: Account<'info, TokenAccount>,

    /// Seller's USDC
    #[account(
        mut,
        constraint = seller_usdc.owner == seller.key()
    )]
    pub seller_usdc: Account<'info, TokenAccount>,

    /// Creator's USDC (for royalty)
    #[account(
        mut,
        constraint = creator_usdc.owner == creator.key()
    )]
    pub creator_usdc: Account<'info, TokenAccount>,

    /// Treasury's USDC
    #[account(
        mut,
        constraint = treasury_usdc.owner == treasury.key()
    )]
    pub treasury_usdc: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<BuyResale>) -> Result<()> {
    // Get values early to reduce stack usage
    let total_price = ctx.accounts.resale_listing.price;
    let secondary_fee_bps = ctx.accounts.platform_config.secondary_fee_bps;
    let creator_royalty_bps = ctx.accounts.royalty_listing.creator_royalty_bps;

    // Calculate fee splits
    let platform_fee = total_price
        .checked_mul(secondary_fee_bps as u64)
        .ok_or(RoyaltiesError::Overflow)?
        .checked_div(10000)
        .ok_or(RoyaltiesError::Overflow)?;

    let creator_royalty = total_price
        .checked_mul(creator_royalty_bps as u64)
        .ok_or(RoyaltiesError::Overflow)?
        .checked_div(10000)
        .ok_or(RoyaltiesError::Overflow)?;

    let seller_amount = total_price
        .checked_sub(platform_fee)
        .ok_or(RoyaltiesError::Overflow)?
        .checked_sub(creator_royalty)
        .ok_or(RoyaltiesError::Overflow)?;

    // Transfer to seller
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.buyer_usdc.to_account_info(),
                to: ctx.accounts.seller_usdc.to_account_info(),
                authority: ctx.accounts.buyer.to_account_info(),
            },
        ),
        seller_amount,
    )?;

    // Transfer platform fee
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.buyer_usdc.to_account_info(),
                to: ctx.accounts.treasury_usdc.to_account_info(),
                authority: ctx.accounts.buyer.to_account_info(),
            },
        ),
        platform_fee,
    )?;

    // Transfer creator royalty
    if creator_royalty > 0 {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.buyer_usdc.to_account_info(),
                    to: ctx.accounts.creator_usdc.to_account_info(),
                    authority: ctx.accounts.buyer.to_account_info(),
                },
            ),
            creator_royalty,
        )?;
    }

    // Transfer NFT from escrow to buyer
    let resale_bump = ctx.accounts.resale_listing.bump;
    let royalty_listing_key = ctx.accounts.royalty_listing.key();
    let seller_key = ctx.accounts.seller.key();
    let seeds = &[
        b"resale_listing",
        royalty_listing_key.as_ref(),
        seller_key.as_ref(),
        &[resale_bump],
    ];
    let signer_seeds = &[&seeds[..]];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.escrow_nft.to_account_info(),
                to: ctx.accounts.buyer_nft.to_account_info(),
                authority: ctx.accounts.resale_listing.to_account_info(),
            },
            signer_seeds,
        ),
        1,
    )?;

    // Update platform stats
    let config = &mut ctx.accounts.platform_config;
    config.total_fees_collected = config
        .total_fees_collected
        .checked_add(platform_fee)
        .ok_or(RoyaltiesError::Overflow)?;

    msg!(
        "Resale complete: {} USDC (platform: {}, creator: {})",
        total_price as f64 / 1_000_000.0,
        platform_fee as f64 / 1_000_000.0,
        creator_royalty as f64 / 1_000_000.0
    );

    Ok(())
}

