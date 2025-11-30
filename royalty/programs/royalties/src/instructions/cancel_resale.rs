use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::{RoyaltyListing, ResaleListing};
use crate::errors::RoyaltiesError;

#[derive(Accounts)]
pub struct CancelResale<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    #[account(
        seeds = [b"royalty_listing", royalty_listing.creator.as_ref(), royalty_listing.nft_mint.as_ref()],
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
        constraint = escrow_nft.owner == resale_listing.key() @ RoyaltiesError::Unauthorized
    )]
    pub escrow_nft: Account<'info, TokenAccount>,

    /// Seller's NFT account to receive back
    #[account(
        mut,
        constraint = seller_nft.owner == seller.key()
    )]
    pub seller_nft: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<CancelResale>) -> Result<()> {
    let resale = &ctx.accounts.resale_listing;
    let royalty_listing_key = ctx.accounts.royalty_listing.key();
    let seller_key = ctx.accounts.seller.key();

    // Transfer NFT back from escrow
    let seeds = &[
        b"resale_listing",
        royalty_listing_key.as_ref(),
        seller_key.as_ref(),
        &[resale.bump],
    ];
    let signer_seeds = &[&seeds[..]];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.escrow_nft.to_account_info(),
                to: ctx.accounts.seller_nft.to_account_info(),
                authority: ctx.accounts.resale_listing.to_account_info(),
            },
            signer_seeds,
        ),
        1,
    )?;

    msg!("Resale listing cancelled");

    Ok(())
}

