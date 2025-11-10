use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
use mpl_core::instructions::TransferV1CpiBuilder;
use crate::{Listing, Market, ErrorCode};

#[derive(Accounts)]
pub struct MatchListing<'info> {
    /// The buyer purchasing the NFT
    #[account(mut)]
    pub buyer: Signer<'info>,

    /// The seller receiving payment (no signature needed - NFT is in escrow)
    /// CHECK: Validated against listing.seller
    #[account(mut)]
    pub seller: UncheckedAccount<'info>,

    /// The listing being purchased
    #[account(
        mut,
        seeds = [
            b"listing",
            listing.seller.as_ref(),
            listing.collection.as_ref(),
            listing.asset.as_ref()
        ],
        bump = listing.bump,
    )]
    pub listing: Account<'info, Listing>,

    /// Market account for fee configuration
    #[account(
        seeds = [b"market"],
        bump = market.bump,
    )]
    pub market: Account<'info, Market>,

    /// Fee recipient receiving marketplace fee
    /// CHECK: Validated against market.fee_recipient
    #[account(mut)]
    pub fee_recipient: UncheckedAccount<'info>,

    /// The NFT asset being purchased
    /// CHECK: Validated against listing.asset
    #[account(mut)]
    pub asset: UncheckedAccount<'info>,

    /// The collection the NFT belongs to
    /// CHECK: Validated against listing.collection
    pub collection: UncheckedAccount<'info>,

    /// MPL Core program for NFT transfer
    /// CHECK: mpl-core program
    pub mpl_core_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[event]
pub struct ListingSold {
    pub listing: Pubkey,
    pub seller: Pubkey,
    pub buyer: Pubkey,
    pub asset: Pubkey,
    pub price: u64,
    pub fee_amount: u64,
    pub timestamp: i64,
}

pub fn process_match_listing(ctx: Context<MatchListing>) -> Result<()> {
    // Save values and accounts we need before mutating listing
    let listing_key = ctx.accounts.listing.key();
    let listing_price = ctx.accounts.listing.price;
    let listing_bump = ctx.accounts.listing.bump;
    let listing_seller = ctx.accounts.listing.seller;
    let listing_collection = ctx.accounts.listing.collection;
    let listing_asset = ctx.accounts.listing.asset;
    let listing_account_info = ctx.accounts.listing.to_account_info();

    let listing = &mut ctx.accounts.listing;
    let market = &ctx.accounts.market;
    let clock = Clock::get()?;

    // Validate listing is active
    require!(listing.is_active, ErrorCode::ListingNotActive);

    // Validate seller matches listing
    require!(
        listing.seller == ctx.accounts.seller.key(),
        ErrorCode::InvalidAsset
    );

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

    // Validate fee_recipient matches market
    require!(
        market.fee_recipient == ctx.accounts.fee_recipient.key(),
        ErrorCode::InvalidFeeRecipient
    );

    // Calculate fee amount (taker fee)
    let fee_amount = (listing_price as u128)
        .checked_mul(market.taker_fee_bps as u128)
        .unwrap()
        .checked_div(10000)
        .unwrap() as u64;

    // Total amount buyer needs to pay
    let total_amount = listing_price.checked_add(fee_amount).unwrap();

    msg!(
        "Purchase: price={} lamports, fee={} lamports, total={} lamports",
        listing_price,
        fee_amount,
        total_amount
    );

    // Transfer payment from buyer to seller
    transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.buyer.to_account_info(),
                to: ctx.accounts.seller.to_account_info(),
            },
        ),
        listing_price,
    )?;

    // Transfer fee from buyer to fee_recipient
    if fee_amount > 0 {
        transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.buyer.to_account_info(),
                    to: ctx.accounts.fee_recipient.to_account_info(),
                },
            ),
            fee_amount,
        )?;
    }

    // Prepare listing PDA signer seeds (using saved values to avoid borrow conflicts)
    let seeds = &[
        b"listing",
        listing_seller.as_ref(),
        listing_collection.as_ref(),
        listing_asset.as_ref(),
        &[listing_bump],
    ];
    let signer = &[&seeds[..]];

    // Transfer NFT from listing PDA to buyer using PDA as authority
    TransferV1CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
        .asset(&ctx.accounts.asset.to_account_info())
        .collection(Some(&ctx.accounts.collection.to_account_info()))
        .payer(&ctx.accounts.buyer.to_account_info())
        .authority(Some(&listing_account_info))  // PDA is authority
        .new_owner(&ctx.accounts.buyer.to_account_info())
        .invoke_signed(signer)?;  // Sign with PDA seeds

    // Mark listing as inactive
    listing.is_active = false;

    msg!(
        "NFT sold: asset {} transferred from escrow to buyer {} for {} lamports (seller: {})",
        ctx.accounts.asset.key(),
        ctx.accounts.buyer.key(),
        listing_price,
        ctx.accounts.seller.key()
    );

    // Emit sale event
    emit!(ListingSold {
        listing: listing_key,
        seller: ctx.accounts.seller.key(),
        buyer: ctx.accounts.buyer.key(),
        asset: ctx.accounts.asset.key(),
        price: listing_price,
        fee_amount,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}
