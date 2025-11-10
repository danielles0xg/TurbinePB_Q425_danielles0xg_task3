use anchor_lang::prelude::*;
use mpl_core::instructions::TransferV1CpiBuilder;
use crate::Listing;
use crate::ErrorCode;

#[derive(Accounts)]
pub struct RemoveListing<'info> {
    /// The seller canceling their listing
    #[account(mut)]
    pub seller: Signer<'info>,

    /// The listing being canceled
    #[account(
        mut,
        seeds = [
            b"listing",
            listing.seller.as_ref(),
            listing.collection.as_ref(),
            listing.asset.as_ref()
        ],
        bump = listing.bump,
        has_one = seller,
        close = seller  // Return rent to seller
    )]
    pub listing: Account<'info, Listing>,

    /// The collection the NFT belongs to
    /// CHECK: Validated against listing.collection
    pub collection: UncheckedAccount<'info>,

    /// The NFT asset being returned to seller
    /// CHECK: Validated against listing.asset
    #[account(mut)]
    pub asset: UncheckedAccount<'info>,

    /// MPL Core program for NFT transfer
    /// CHECK: mpl-core program
    pub mpl_core_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[event]
pub struct ListingCanceled {
    pub listing: Pubkey,
    pub seller: Pubkey,
    pub asset: Pubkey,
    pub timestamp: i64,
}

pub fn process_remove_listing(ctx: Context<RemoveListing>) -> Result<()> {
    let listing = &ctx.accounts.listing;
    let clock = Clock::get()?;

    // Validate listing is active
    require!(listing.is_active, ErrorCode::ListingNotActive);

    // Validate asset matches listing
    require!(
        listing.asset == ctx.accounts.asset.key(),
        ErrorCode::InvalidAsset
    );

    // Validate collection matches listing
    require!(
        listing.collection == ctx.accounts.collection.key(),
        ErrorCode::InvalidAsset
    );

    // Prepare listing PDA signer seeds
    let seeds = &[
        b"listing",
        listing.seller.as_ref(),
        listing.collection.as_ref(),
        listing.asset.as_ref(),
        &[listing.bump],
    ];
    let signer = &[&seeds[..]];

    // Transfer NFT from listing PDA back to seller
    TransferV1CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
        .asset(&ctx.accounts.asset.to_account_info())
        .collection(Some(&ctx.accounts.collection.to_account_info()))
        .payer(&ctx.accounts.seller.to_account_info())
        .authority(Some(&ctx.accounts.listing.to_account_info()))  // PDA is authority
        .new_owner(&ctx.accounts.seller.to_account_info())
        .invoke_signed(signer)?;  // Sign with PDA seeds

    msg!(
        "Listing canceled: {} - NFT {} returned to seller {}",
        ctx.accounts.listing.key(),
        ctx.accounts.asset.key(),
        ctx.accounts.seller.key()
    );

    // Emit cancellation event
    emit!(ListingCanceled {
        listing: ctx.accounts.listing.key(),
        seller: ctx.accounts.seller.key(),
        asset: ctx.accounts.asset.key(),
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}
