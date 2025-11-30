use anchor_lang::prelude::*;
use crate::state::PlatformConfig;
use crate::errors::RoyaltiesError;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = PlatformConfig::LEN,
        seeds = [b"platform_config"],
        bump
    )]
    pub platform_config: Account<'info, PlatformConfig>,

    /// CHECK: Treasury account to receive fees
    pub treasury: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Initialize>, platform_fee_bps: u16) -> Result<()> {
    // Max 10% fee
    require!(platform_fee_bps <= 1000, RoyaltiesError::FeeTooHigh);

    let config = &mut ctx.accounts.platform_config;
    config.authority = ctx.accounts.authority.key();
    config.treasury = ctx.accounts.treasury.key();
    config.platform_fee_bps = platform_fee_bps;
    config.secondary_fee_bps = 250; // 2.5% default for secondary
    config.total_fees_collected = 0;
    config.bump = ctx.bumps.platform_config;

    msg!("Platform initialized with {}bps fee", platform_fee_bps);
    Ok(())
}

