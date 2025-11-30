use anchor_lang::prelude::*;

#[error_code]
pub enum RoyaltiesError {
    #[msg("Platform fee exceeds maximum allowed (10%)")]
    FeeTooHigh,
    
    #[msg("Listing is not active")]
    ListingNotActive,
    
    #[msg("Listing has expired")]
    ListingExpired,
    
    #[msg("Insufficient funds for purchase")]
    InsufficientFunds,
    
    #[msg("Resale is not allowed for this listing")]
    ResaleNotAllowed,
    
    #[msg("You don't own this NFT")]
    NotOwner,
    
    #[msg("Invalid percentage (must be 1-10000 bps)")]
    InvalidPercentage,
    
    #[msg("Invalid price")]
    InvalidPrice,
    
    #[msg("Payout pool is empty")]
    PayoutPoolEmpty,
    
    #[msg("Already claimed for this period")]
    AlreadyClaimed,
    
    #[msg("Unauthorized")]
    Unauthorized,
    
    #[msg("Invalid metadata URI")]
    InvalidMetadataUri,
    
    #[msg("Calculation overflow")]
    Overflow,
}

