use anchor_lang::prelude::*;
use instructions::*;
mod instructions;


mod error;
use error::ErrorCode;

declare_id!("JAqqAnG68DTJrWMmmz6FnNKfvBaSKsLTMqS3u7iTF5ck");

#[program]
pub mod nft_marketplace {
    use super::*;

    pub fn init_market(
        ctx: Context<InitMarket>,
        fee_recipient: Pubkey,
        taker_fee_bps: u64
        ) -> Result<()> {
            require!(taker_fee_bps <= 10000, ErrorCode::FeeTooHigh);

            let market = &mut ctx.accounts.market;
            market.admin = ctx.accounts.admin.key();
            market.fee_recipient = fee_recipient.key();
            market.taker_fee_bps = taker_fee_bps;
            market.bump = ctx.bumps.market;
            Ok(())
        }

    // only admin
    pub fn update_market(
        ctx: Context<UpdateMarket>,
        fee_recipient: Pubkey,
        taker_fee_bps: u64
    ) -> Result<()> {
        require!(taker_fee_bps <= 10000, ErrorCode::FeeTooHigh); // basis points n/10_000 = percentage

        let market = &mut ctx.accounts.market;
        market.fee_recipient = fee_recipient;
        market.taker_fee_bps = taker_fee_bps;

        Ok(())
    }

    // Create an NFT collection
    pub fn create_collection(ctx: Context<CreateCollection>, name: String, uri: String) -> Result<()> {
        instructions::create_collection::process_create_collection(ctx, name, uri)
    }

    // List an NFT for sale
    pub fn add_listing(ctx: Context<AddListing>, price: u64) -> Result<()> {
        instructions::add_listing::process_add_listing(ctx, price)
    }

    // Remove a listing and return NFT to seller
    pub fn remove_listing(ctx: Context<RemoveListing>) -> Result<()> {
        instructions::remove_listing::process_remove_listing(ctx)
    }

    // Manual order matching
    pub fn match_listing(ctx: Context<MatchListing>) -> Result<()> {
        instructions::match_listing::process_match_listing(ctx)
    }
}

// ***************************************************
// ******************* GLOBAL STATE ******************
// ***************************************************


#[account]
#[derive(InitSpace)]
pub struct Market {
    pub admin: Pubkey,
    pub fee_recipient: Pubkey,
    pub taker_fee_bps: u64, // basis points (buyer pays this fee to market)
    pub bump: u8
}

#[account]
#[derive(InitSpace)]
pub struct Listing {
    pub seller: Pubkey,              // The person who listed the NFT
    pub collection: Pubkey,          // The collection the NFT belongs to
    pub asset: Pubkey,               // The NFT asset ID (mpl-core asset)
    pub price: u64,                  // Price in lamports 1e9
    pub is_active: bool,             // Whether the listing is still active
    pub created_at: i64,             // Unix timestamp when listed
    pub bump: u8,                    // PDA bump
}


// ***************************************************
// ********************* INX STATE *******************
// ***************************************************

// INIT MARKET

#[derive(Accounts)]
pub struct InitMarket<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = Market::DISCRIMINATOR.len() + Market::INIT_SPACE,
        seeds = [b"market"],
        bump,
    )]
    pub market: Account<'info, Market>,
    pub system_program: Program<'info, System>,
}

// UPDATE MARKET

#[derive(Accounts)]
pub struct UpdateMarket<'info> {
    pub admin: Signer <'info>,
    #[account(
        mut,
        seeds = [b"market"],
        bump = market.bump,
        has_one = admin
    )]
    pub market : Account<'info, Market>,
}


