use anchor_lang::prelude::*;
use crate::{Listing, Market};

#[derive(Accounts)]
pub struct AddListing<'info> {
    /// The seller listing their NFT
    #[account(mut)]
    pub seller: Signer<'info>,

    /// The listing account - PDA with seeds [b"listing", seller, collection, asset]
    #[account(
        init,
        payer = seller,
        space = 8 + Listing::INIT_SPACE,
        seeds = [
            b"listing",
            seller.key().as_ref(),
            collection.key().as_ref(),
            asset.key().as_ref()
        ],
        bump
    )]
    pub listing: Account<'info, Listing>,

    /// Market account (for validation)
    #[account(
        seeds = [b"market"],
        bump = market.bump,
    )]
    pub market: Account<'info, Market>,

    /// The collection the NFT belongs to
    /// CHECK: Validated as mpl-core collection
    pub collection: UncheckedAccount<'info>,

    /// The NFT asset being listed
    /// CHECK: Validated as mpl-core asset from the collection
    pub asset: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[event]
pub struct ListingCreated {
    pub listing: Pubkey,
    pub seller: Pubkey,
    pub collection: Pubkey,
    pub asset: Pubkey,
    pub price: u64,
    pub timestamp: i64,
}

pub fn process_add_listing(
    ctx: Context<AddListing>,
    price: u64,
) -> Result<()> {
    let listing = &mut ctx.accounts.listing;
    let clock = Clock::get()?;

    // Initialize listing data
    listing.seller = ctx.accounts.seller.key();
    listing.collection = ctx.accounts.collection.key();
    listing.asset = ctx.accounts.asset.key();
    listing.price = price;
    listing.is_active = true;
    listing.created_at = clock.unix_timestamp;
    listing.bump = ctx.bumps.listing;

    msg!(
        "Listing created: {} for asset: {} at price: {} lamports",
        ctx.accounts.listing.key(),
        ctx.accounts.asset.key(),
        price
    );

    // Emit listing created event
    emit!(ListingCreated {
        listing: ctx.accounts.listing.key(),
        seller: ctx.accounts.seller.key(),
        collection: ctx.accounts.collection.key(),
        asset: ctx.accounts.asset.key(),
        price,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}
