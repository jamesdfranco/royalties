use anchor_lang::prelude::*;

/// Platform configuration account
#[account]
#[derive(Default)]
pub struct PlatformConfig {
    /// Platform authority (admin)
    pub authority: Pubkey,
    /// Platform treasury for collecting fees
    pub treasury: Pubkey,
    /// Platform fee in basis points (e.g., 500 = 5%)
    pub platform_fee_bps: u16,
    /// Secondary market fee in basis points (e.g., 250 = 2.5%)
    pub secondary_fee_bps: u16,
    /// Total fees collected (for tracking)
    pub total_fees_collected: u64,
    /// Bump seed for PDA
    pub bump: u8,
}

impl PlatformConfig {
    pub const LEN: usize = 8 + // discriminator
        32 + // authority
        32 + // treasury
        2 +  // platform_fee_bps
        2 +  // secondary_fee_bps
        8 +  // total_fees_collected
        1;   // bump
}

/// Royalty listing account - represents a creator's royalty offering
#[account]
pub struct RoyaltyListing {
    /// Creator's wallet address
    pub creator: Pubkey,
    /// NFT mint address
    pub nft_mint: Pubkey,
    /// Revenue source description (stored as hash, full data on IPFS)
    pub metadata_uri: String,
    /// Percentage of royalties being sold (basis points, e.g., 500 = 5%)
    pub percentage_bps: u16,
    /// Duration in seconds (0 = perpetual)
    pub duration_seconds: u64,
    /// Start timestamp
    pub start_timestamp: i64,
    /// Price in USDC (6 decimals)
    pub price: u64,
    /// Whether resale is allowed
    pub resale_allowed: bool,
    /// Creator's royalty on resales (basis points)
    pub creator_royalty_bps: u16,
    /// Current status
    pub status: ListingStatus,
    /// Bump seed
    pub bump: u8,
}

impl RoyaltyListing {
    pub const LEN: usize = 8 + // discriminator
        32 + // creator
        32 + // nft_mint
        4 + 200 + // metadata_uri (max 200 chars)
        2 +  // percentage_bps
        8 +  // duration_seconds
        8 +  // start_timestamp
        8 +  // price
        1 +  // resale_allowed
        2 +  // creator_royalty_bps
        1 +  // status
        1;   // bump
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum ListingStatus {
    Active,    // Available for purchase
    Sold,      // Purchased, NFT minted
    Cancelled, // Creator cancelled
    Expired,   // Duration ended
}

impl Default for ListingStatus {
    fn default() -> Self {
        ListingStatus::Active
    }
}

/// Secondary market listing
#[account]
pub struct ResaleListing {
    /// Current owner (seller)
    pub seller: Pubkey,
    /// Original royalty listing
    pub royalty_listing: Pubkey,
    /// NFT mint
    pub nft_mint: Pubkey,
    /// Asking price in USDC
    pub price: u64,
    /// Listing timestamp
    pub listed_at: i64,
    /// Bump seed
    pub bump: u8,
}

impl ResaleListing {
    pub const LEN: usize = 8 + // discriminator
        32 + // seller
        32 + // royalty_listing
        32 + // nft_mint
        8 +  // price
        8 +  // listed_at
        1;   // bump
}

/// Payout pool for a royalty listing
#[account]
pub struct PayoutPool {
    /// Associated royalty listing
    pub royalty_listing: Pubkey,
    /// Creator who deposits
    pub creator: Pubkey,
    /// Total amount deposited for current period
    pub total_deposited: u64,
    /// Amount already claimed
    pub total_claimed: u64,
    /// Deposit timestamp
    pub deposited_at: i64,
    /// Payout period identifier
    pub period: u64,
    /// Bump seed
    pub bump: u8,
}

impl PayoutPool {
    pub const LEN: usize = 8 + // discriminator
        32 + // royalty_listing
        32 + // creator
        8 +  // total_deposited
        8 +  // total_claimed
        8 +  // deposited_at
        8 +  // period
        1;   // bump
}

/// Tracks claims for a specific holder in a payout period
#[account]
pub struct PayoutClaim {
    /// The payout pool
    pub payout_pool: Pubkey,
    /// The holder who claimed
    pub holder: Pubkey,
    /// Amount claimed
    pub amount_claimed: u64,
    /// Claim timestamp
    pub claimed_at: i64,
    /// Bump seed
    pub bump: u8,
}

impl PayoutClaim {
    pub const LEN: usize = 8 + // discriminator
        32 + // payout_pool
        32 + // holder
        8 +  // amount_claimed
        8 +  // claimed_at
        1;   // bump
}

