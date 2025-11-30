use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};
use crate::state::{RoyaltyListing, ListingStatus, PlatformConfig};
use crate::errors::RoyaltiesError;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateListingArgs {
    /// IPFS URI to metadata/contract PDF
    pub metadata_uri: String,
    /// Percentage of royalties in basis points (e.g., 500 = 5%)
    pub percentage_bps: u16,
    /// Duration in seconds (0 = perpetual)
    pub duration_seconds: u64,
    /// Price in USDC (6 decimals)
    pub price: u64,
    /// Allow resale on secondary market
    pub resale_allowed: bool,
    /// Creator royalty on resales (basis points)
    pub creator_royalty_bps: u16,
}

#[derive(Accounts)]
#[instruction(args: CreateListingArgs)]
pub struct CreateListing<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        seeds = [b"platform_config"],
        bump = platform_config.bump
    )]
    pub platform_config: Account<'info, PlatformConfig>,

    #[account(
        init,
        payer = creator,
        space = RoyaltyListing::LEN,
        seeds = [b"royalty_listing", creator.key().as_ref(), nft_mint.key().as_ref()],
        bump
    )]
    pub royalty_listing: Account<'info, RoyaltyListing>,

    #[account(
        init,
        payer = creator,
        mint::decimals = 0,
        mint::authority = royalty_listing,
        mint::freeze_authority = royalty_listing,
    )]
    pub nft_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<CreateListing>, args: CreateListingArgs) -> Result<()> {
    // Validate inputs
    require!(
        args.percentage_bps > 0 && args.percentage_bps <= 10000,
        RoyaltiesError::InvalidPercentage
    );
    require!(args.price > 0, RoyaltiesError::InvalidPrice);
    require!(
        args.metadata_uri.len() > 0 && args.metadata_uri.len() <= 200,
        RoyaltiesError::InvalidMetadataUri
    );
    require!(
        args.creator_royalty_bps <= 1000, // Max 10% creator royalty on resales
        RoyaltiesError::FeeTooHigh
    );

    let listing = &mut ctx.accounts.royalty_listing;
    let clock = Clock::get()?;

    listing.creator = ctx.accounts.creator.key();
    listing.nft_mint = ctx.accounts.nft_mint.key();
    listing.metadata_uri = args.metadata_uri;
    listing.percentage_bps = args.percentage_bps;
    listing.duration_seconds = args.duration_seconds;
    listing.start_timestamp = clock.unix_timestamp;
    listing.price = args.price;
    listing.resale_allowed = args.resale_allowed;
    listing.creator_royalty_bps = args.creator_royalty_bps;
    listing.status = ListingStatus::Active;
    listing.bump = ctx.bumps.royalty_listing;

    msg!(
        "Listing created: {}% for {} USDC",
        args.percentage_bps as f64 / 100.0,
        args.price as f64 / 1_000_000.0
    );

    Ok(())
}

