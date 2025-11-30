# Royalties Smart Contracts

Solana program for a royalty marketplace built with Anchor Framework. Enables creators to tokenize revenue streams as NFTs and distribute royalty payouts to holders.

## Program Architecture

### State Accounts

**PlatformConfig** - Global platform configuration
- Platform authority and treasury addresses
- Primary and secondary market fee structures (basis points)
- Total fees collected tracking

**RoyaltyListing** - Individual royalty listing 
- Creator and NFT mint addresses
- Metadata URI, percentage, duration, pricing
- Resale permissions and creator royalty percentage
- Listing status tracking (Active/Sold/Cancelled/Expired)

**ResaleListing** - Secondary market listing
- Seller and original royalty listing references
- Resale price and timestamp
- Escrow account for NFT during listing

**PayoutPool** - Royalty distribution pool
- Links to royalty listing and creator
- Total deposited/claimed amounts
- Period-based distribution tracking

**PayoutClaim** - Individual holder claim record
- Holder address and claimed amount
- Timestamp for claim verification

### Instructions

#### initialize
Initialize the platform with configurable fees
- **Accounts**: authority, platform_config, treasury, system_program
- **Args**: platform_fee_bps (u16)
- **Access**: Platform authority only

#### create_listing
Create a new royalty listing and mint NFT
- **Accounts**: creator, platform_config, royalty_listing, nft_mint, token_program, system_program, rent
- **Args**: CreateListingArgs (metadata_uri, percentage_bps, duration_seconds, price, resale_allowed, creator_royalty_bps)
- **Validations**: Fee limits, percentage bounds, metadata URI format

#### buy_listing
Purchase primary market listing with USDC
- **Accounts**: buyer, creator, platform_config, treasury, royalty_listing, nft_mint, USDC accounts (buyer/creator/treasury), token programs
- **Logic**: 
  - Transfer USDC from buyer to creator (minus platform fee)
  - Transfer platform fee to treasury
  - Mint NFT to buyer
  - Update listing status to Sold

#### list_for_resale
List owned NFT on secondary market
- **Accounts**: seller, royalty_listing, resale_listing, seller_nft, escrow_nft, nft_mint, token_program, system_program, rent
- **Args**: price (u64)
- **Validations**: NFT ownership, resale permission, active listing
- **Logic**: Transfer NFT from seller to escrow account

#### buy_resale
Purchase from secondary market
- **Accounts**: buyer, seller, creator, platform_config, treasury, royalty_listing, resale_listing, escrow_nft, buyer_nft, USDC accounts
- **Logic**:
  - Calculate and distribute fees (platform + creator royalty)
  - Transfer USDC to seller (minus fees)
  - Transfer NFT from escrow to buyer
  - Close resale listing

#### cancel_resale
Cancel secondary market listing
- **Accounts**: seller, royalty_listing, resale_listing, escrow_nft, seller_nft, token_program
- **Access**: Original seller only
- **Logic**: Return NFT from escrow to seller, close listing

#### deposit_payout
Deposit royalty payouts to pool
- **Accounts**: creator, royalty_listing, payout_pool, creator_usdc, pool_vault, usdc_mint, token_program, system_program, rent
- **Args**: amount (u64)
- **Access**: Original creator only
- **Logic**: Transfer USDC from creator to pool vault

#### claim_payout
Claim proportional share of payouts
- **Accounts**: holder, royalty_listing, payout_pool, payout_claim, holder_nft, pool_vault, holder_usdc, token_program, system_program
- **Logic**:
  - Verify NFT ownership
  - Calculate proportional share based on percentage_bps
  - Transfer USDC from pool to holder
  - Record claim to prevent double-claiming

## Error Codes

| Code | Error | Description |
|------|-------|-------------|
| 6000 | FeeTooHigh | Platform fee exceeds 10% maximum |
| 6001 | ListingNotActive | Listing status is not Active |
| 6002 | ListingExpired | Listing duration has expired |
| 6003 | InsufficientFunds | Buyer has insufficient USDC |
| 6004 | ResaleNotAllowed | Creator disabled resale for this listing |
| 6005 | NotOwner | Caller doesn't own the NFT |
| 6006 | InvalidPercentage | Percentage not in valid range (1-10000 bps) |
| 6007 | InvalidPrice | Price is zero or invalid |
| 6008 | PayoutPoolEmpty | No funds available in payout pool |
| 6009 | AlreadyClaimed | Holder already claimed for this period |
| 6010 | Unauthorized | Caller lacks required permissions |
| 6011 | InvalidMetadataUri | Metadata URI format invalid |
| 6012 | Overflow | Arithmetic operation overflow |

## Security Features

- **Fee Validation**: Maximum 10% (1000 bps) platform fee enforcement
- **Access Control**: Instruction-level authorization checks (creator, owner, authority)
- **Overflow Protection**: Safe math operations with explicit overflow handling
- **Double-Claim Prevention**: Period-based claim tracking with PayoutClaim accounts
- **Escrow Safety**: PDA-based escrow accounts for secure NFT custody during resale
- **Status Validation**: Listing status checks before state-modifying operations

## Building & Testing

```bash
# Build the program
anchor build

# Run tests
anchor test

# Deploy to devnet
anchor deploy --provider.cluster devnet
```

## Dependencies

```toml
[dependencies]
anchor-lang = "0.29.0"
anchor-spl = "0.29.0"
```

## Program ID

Update in `Anchor.toml` and `lib.rs` after deployment:

```rust
declare_id!("YOUR_PROGRAM_ID_HERE");
```

## License

MIT
