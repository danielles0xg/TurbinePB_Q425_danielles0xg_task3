use anchor_lang::prelude::*;
use mpl_core::instructions::CreateCollectionV2CpiBuilder;

#[derive(Accounts)]
pub struct CreateCollection<'info> {
    /// The collection account to create (mpl-core collection)
    #[account(mut)]
    pub collection: Signer<'info>,

    /// The creator/update authority of the collection
    pub update_authority: Signer<'info>,

    /// The account paying for rent
    #[account(mut)]
    pub payer: Signer<'info>,

    /// System program
    pub system_program: Program<'info, System>,

    /// MPL Core program
    /// CHECK: mpl-core program
    pub mpl_core_program: UncheckedAccount<'info>,
}

#[event]
pub struct CollectionCreated {
    pub collection: Pubkey,
    pub update_authority: Pubkey,
    pub name: String,
    pub uri: String,
    pub timestamp: i64,
}

pub fn process_create_collection(ctx: Context<CreateCollection>, name: String, uri: String) -> Result<()> {
    // Create the collection using mpl-core CPI
    CreateCollectionV2CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
        .collection(&ctx.accounts.collection.to_account_info())
        .update_authority(Some(&ctx.accounts.update_authority.to_account_info()))
        .payer(&ctx.accounts.payer.to_account_info())
        .system_program(&ctx.accounts.system_program.to_account_info())
        .name(name.clone())
        .uri(uri.clone())
        .invoke()?;

    msg!("Collection created: {}",ctx.accounts.collection.key());

    // Emit collection created event
    emit!(CollectionCreated {
        collection: ctx.accounts.collection.key(),
        update_authority: ctx.accounts.update_authority.key(),
        name,
        uri,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
