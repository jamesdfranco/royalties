use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("8iLsYGHoGtN6gCVmVCSWrbMAKnj1e3hzjv292e3TTBfg");

#[program]
pub mod royalties {
    use super::*;

    /// Initialize the platform configuration
    pub fn initialize(ctx: Context<Initialize>, platform_fee_bps: u16) -> Result<()> {
        instructions::initialize::handler(ctx, platform_fee_bps)
    }

    /// Creator lists royalties for sale - mints NFT representing ownership
    pub fn create_listing(
        ctx: Context<CreateListing>,
        args: CreateListingArgs,
    ) -> Result<()> {
        instructions::create_listing::handler(ctx, args)
    }

    /// Buyer purchases a royalty listing from primary market
    pub fn buy_listing(ctx: Context<BuyListing>) -> Result<()> {
        instructions::buy_listing::handler(ctx)
    }

    /// List a royalty token for resale on secondary market
    pub fn list_for_resale(
        ctx: Context<ListForResale>,
        price: u64,
    ) -> Result<()> {
        instructions::list_for_resale::handler(ctx, price)
    }

    /// Buy a royalty token from secondary market
    pub fn buy_resale(ctx: Context<BuyResale>) -> Result<()> {
        instructions::buy_resale::handler(ctx)
    }

    /// Cancel a resale listing
    pub fn cancel_resale(ctx: Context<CancelResale>) -> Result<()> {
        instructions::cancel_resale::handler(ctx)
    }

    /// Creator deposits payout for royalty holders
    pub fn deposit_payout(ctx: Context<DepositPayout>, amount: u64) -> Result<()> {
        instructions::deposit_payout::handler(ctx, amount)
    }

    /// Holder claims their share of the payout
    pub fn claim_payout(ctx: Context<ClaimPayout>) -> Result<()> {
        instructions::claim_payout::handler(ctx)
    }
}

