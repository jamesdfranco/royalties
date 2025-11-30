use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::{RoyaltyListing, ResaleListing, ListingStatus};
use crate::errors::RoyaltiesError;

#[derive(Accounts)]
pub struct ListForResale<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    #[account(
        seeds = [b"royalty_listing", royalty_listing.creator.as_ref(), royalty_listing.nft_mint.as_ref()],
        bump = royalty_listing.bump,
        constraint = royalty_listing.status == ListingStatus::Sold @ RoyaltiesError::ListingNotActive,
        constraint = royalty_listing.resale_allowed @ RoyaltiesError::ResaleNotAllowed
    )]
    pub royalty_listing: Account<'info, RoyaltyListing>,

    #[account(
        init,
        payer = seller,
        space = ResaleListing::LEN,
        seeds = [b"resale_listing", royalty_listing.key().as_ref(), seller.key().as_ref()],
        bump
    )]
    pub resale_listing: Account<'info, ResaleListing>,

    /// Seller's NFT token account
    #[account(
        mut,
        constraint = seller_nft.owner == seller.key() @ RoyaltiesError::NotOwner,
        constraint = seller_nft.mint == royalty_listing.nft_mint @ RoyaltiesError::Unauthorized,
        constraint = seller_nft.amount == 1 @ RoyaltiesError::NotOwner
    )]
    pub seller_nft: Account<'info, TokenAccount>,

    /// Escrow account to hold NFT during listing
    #[account(
        init_if_needed,
        payer = seller,
        token::mint = nft_mint,
        token::authority = resale_listing,
    )]
    pub escrow_nft: Account<'info, TokenAccount>,

    /// CHECK: NFT mint
    pub nft_mint: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<ListForResale>, price: u64) -> Result<()> {
    require!(price > 0, RoyaltiesError::InvalidPrice);

    let clock = Clock::get()?;

    // Transfer NFT to escrow
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.seller_nft.to_account_info(),
                to: ctx.accounts.escrow_nft.to_account_info(),
                authority: ctx.accounts.seller.to_account_info(),
            },
        ),
        1,
    )?;

    // Create resale listing
    let resale = &mut ctx.accounts.resale_listing;
    resale.seller = ctx.accounts.seller.key();
    resale.royalty_listing = ctx.accounts.royalty_listing.key();
    resale.nft_mint = ctx.accounts.royalty_listing.nft_mint;
    resale.price = price;
    resale.listed_at = clock.unix_timestamp;
    resale.bump = ctx.bumps.resale_listing;

    msg!("Listed for resale at {} USDC", price as f64 / 1_000_000.0);

    Ok(())
}

