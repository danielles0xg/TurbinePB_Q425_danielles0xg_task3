use anchor_lang::prelude::*;

mod error;
use error::ErrorCode;

declare_id!("JAqqAnG68DTJrWMmmz6FnNKfvBaSKsLTMqS3u7iTF5ck");

#[program]
pub mod nft_marketplace {
    use super::*;

    pub fn init_market(
        ctx: Context<InitMarket>,
        fee_recipient: Pubkey,
        minting_fee_bps: u64,
        maker_fee_bps: u64,
        taker_fee_bps: u64 
        ) -> Result<()> {
            let market = &mut ctx.accounts.market;
            market.admin = ctx.accounts.admin.key();
            market.fee_recipient = fee_recipient.key();
            market.minting_fee_bps = minting_fee_bps;
            market.maker_fee_bps = maker_fee_bps;
            market.taker_fee_bps = taker_fee_bps;
            market.bump = ctx.bumps.market;
            Ok(())
        }

    // only admin
    pub fn update_market(
        ctx: Context<UpdateMarket>,
        fee_recipient: Pubkey,
        minting_fee_bps: u64,
        maker_fee_bps: u64,
        taker_fee_bps: u64
    ) -> Result<()> {
        require!(minting_fee_bps <= 10000, ErrorCode::FeeTooHigh);
        require!(maker_fee_bps <= 10000, ErrorCode::FeeTooHigh);
        require!(taker_fee_bps <= 10000, ErrorCode::FeeTooHigh);

        let market = &mut ctx.accounts.market;
        market.fee_recipient = fee_recipient;
        market.minting_fee_bps = minting_fee_bps;
        market.maker_fee_bps = maker_fee_bps;
        market.taker_fee_bps = taker_fee_bps;

        Ok(())
    }

    // mint nft
    // pub fn create_collection(ctx: Context<CreateCollection>) -> Result<()> {
    //     Ok(())
    // }

    // // make offer by seller
    // pub fn add_listing(ctx: Context<ListItem>) -> Result<()> {
    //     Ok(())
    // }

    // pub fn remove_listing(ctx: Context<RemoveListing>) -> Result<()> {
    //     Ok(())
    // }

    // // match offer by buyer
    // pub fn match_listing(ctx: Context<MatchListing>) -> Result<()> {
    //     Ok(())
    // }
}

// ***************************************************
// ******************* GLOBAL STATE ******************
// ***************************************************


#[account]
#[derive(InitSpace)]
pub struct Market {
    pub admin: Pubkey,
    pub fee_recipient: Pubkey,
    pub minting_fee_bps: u64, // basis points
    pub maker_fee_bps: u64,
    pub taker_fee_bps: u64,
    pub bump: u8
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


