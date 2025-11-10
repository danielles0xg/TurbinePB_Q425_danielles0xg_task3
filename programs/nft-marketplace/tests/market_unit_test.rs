use litesvm::LiteSVM;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

// System program ID constant
const SYSTEM_PROGRAM_ID: Pubkey = Pubkey::new_from_array([0u8; 32]);

// Import solana_program for mpl-core compatibility
use anchor_lang::solana_program;

use nft_marketplace;

// Helper to create Anchor instruction discriminator
fn anchor_discriminator(namespace: &str, name: &str) -> [u8; 8] {
    let preimage = format!("{}:{}", namespace, name);
    let mut hasher = solana_sdk::hash::Hasher::default();
    hasher.hash(preimage.as_bytes());
    let hash = hasher.result();
    let mut discriminator = [0u8; 8];
    discriminator.copy_from_slice(&hash.to_bytes()[..8]);
    discriminator
}

#[test]
fn init_market() {
    // Create the test environment
    let mut svm = LiteSVM::new();

    let program_id = Pubkey::new_from_array(nft_marketplace::ID.to_bytes());
    let program_bytes = include_bytes!("../../../target/deploy/nft_marketplace.so");
    svm.add_program(program_id, program_bytes).unwrap();


    // Create a test account
    let admin = Keypair::new();
    let fee_recipient = Keypair::new();

    // Fund the account with SOL
    svm.airdrop(&admin.pubkey(), 1_000_000_000).unwrap();

    // Check the balance
    let balance = svm.get_balance(&admin.pubkey()).unwrap();
    assert_eq!(balance, 1_000_000_000);

    // Derive the market PDA
    let (market_pda, _bump) = Pubkey::find_program_address(&[b"market"], &program_id);

    // Prepare instruction parameters
    let minting_fee_bps: u64 = 200;  // 2%
    let maker_fee_bps: u64 = 100;    // 1%
    let taker_fee_bps: u64 = 150;    // 1.5%

    // Create instruction data with Anchor format
    let mut data = Vec::new();

    // Add discriminator for "global:init_market"
    data.extend_from_slice(&anchor_discriminator("global", "init_market"));

    // Add parameters (serialized in order)
    data.extend_from_slice(&fee_recipient.pubkey().to_bytes());  // fee_recipient: Pubkey (32 bytes)
    data.extend_from_slice(&minting_fee_bps.to_le_bytes());      // minting_fee_bps: u64 (8 bytes)
    data.extend_from_slice(&maker_fee_bps.to_le_bytes());        // maker_fee_bps: u64 (8 bytes)
    data.extend_from_slice(&taker_fee_bps.to_le_bytes());        // taker_fee_bps: u64 (8 bytes)

    // Create instruction
    let instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(admin.pubkey(), true),           // admin (signer)
            AccountMeta::new(market_pda, false),              // market (PDA to initialize)
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),  // system_program
        ],
        data,
    };
    // Build transaction
    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&admin.pubkey()),
        &[&admin],
        svm.latest_blockhash(),
    );

    // Send transaction
    let result = svm.send_transaction(tx);

    // Check result
    match result {
        Ok(tx_result) => {
            println!("Market initialized successfully!");
            println!("Transaction logs:");
            for log in &tx_result.logs {
                println!("  {}", log);
            }

            // Verify market account was created
            let market_account = svm.get_account(&market_pda);
            assert!(market_account.is_some(), "Market account should exist");
            println!("\nMarket PDA: {}", market_pda);
            println!("Admin: {}", admin.pubkey());
            println!("Fee recipient: {}", fee_recipient.pubkey());
        }
        Err(e) => {
            panic!("Transaction failed: {:?}", e);
        }
    }
}

fn get_mpl_core_id() -> [u8; 32] {
    let mpl_core_str = "CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d";
    let pubkey: Pubkey = mpl_core_str.parse().unwrap();
    println!("Bytes: {:?}", pubkey.to_bytes());
    return pubkey.to_bytes();
}

#[test]
fn test_create_collection() {
    // Create the test environment
    let mut svm = LiteSVM::new();

    let program_id = Pubkey::new_from_array(nft_marketplace::ID.to_bytes());
    let program_bytes = include_bytes!("../../../target/deploy/nft_marketplace.so");
    svm.add_program(program_id, program_bytes).unwrap();

    // MPL Core program ID: CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d
    let mpl_core_id : Pubkey = Pubkey::new_from_array(get_mpl_core_id());

    // Load mpl-core program
    let mpl_core_bytes = include_bytes!("mpl_core.so");
    svm.add_program(mpl_core_id, mpl_core_bytes).unwrap();

    // Create keypairs
    let collection = Keypair::new();
    let update_authority = Keypair::new();
    let payer = Keypair::new();

    // Fund the payer
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    // Collection parameters
    let name = "My NFT Collection";
    let uri = "https://example.com/collection.json";

    // Create instruction data with Anchor format
    let mut data = Vec::new();

    // Add discriminator for "global:create_collection"
    data.extend_from_slice(&anchor_discriminator("global", "create_collection"));

    // Serialize parameters using borsh
    // name: String
    let name_bytes = name.as_bytes();
    data.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
    data.extend_from_slice(name_bytes);

    // uri: String
    let uri_bytes = uri.as_bytes();
    data.extend_from_slice(&(uri_bytes.len() as u32).to_le_bytes());
    data.extend_from_slice(uri_bytes);

    // Create instruction (no market/fee_recipient needed)
    let instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(collection.pubkey(), true),              // collection (signer)
            AccountMeta::new_readonly(update_authority.pubkey(), true), // update_authority (signer)
            AccountMeta::new(payer.pubkey(), true),                   // payer (signer)
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),      // system_program
            AccountMeta::new_readonly(mpl_core_id, false),            // mpl_core_program
        ],
        data,
    };

    // Build transaction
    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
        &[&payer, &collection, &update_authority],
        svm.latest_blockhash(),
    );

    // Send transaction
    let result = svm.send_transaction(tx);

    // Check result
    let tx_result = result.expect("Transaction should succeed");

    println!("Collection created successfully!");
    println!("Transaction logs:");
    for log in &tx_result.logs {
        println!("  {}", log);
    }

    // Verify collection account was created
    let collection_account = svm.get_account(&collection.pubkey());
    assert!(collection_account.is_some(), "Collection account should exist");

    println!("\nCollection: {}", collection.pubkey());
    println!("Update Authority: {}", update_authority.pubkey());
    println!("Name: {}", name);
    println!("URI: {}", uri);
}

#[test]
fn test_add_listing() {
    // Create the test environment
    let mut svm = LiteSVM::new();

    let program_id = Pubkey::new_from_array(nft_marketplace::ID.to_bytes());
    let program_bytes = include_bytes!("../../../target/deploy/nft_marketplace.so");
    svm.add_program(program_id, program_bytes).unwrap();

    // MPL Core program ID
    let mpl_core_id = Pubkey::new_from_array(get_mpl_core_id());
    let mpl_core_bytes = include_bytes!("mpl_core.so");
    svm.add_program(mpl_core_id, mpl_core_bytes).unwrap();

    // Create keypairs
    let admin = Keypair::new();
    let fee_recipient = Keypair::new();
    let collection = Keypair::new();
    let asset = Keypair::new();
    let update_authority = Keypair::new();
    let seller = Keypair::new();

    // Fund accounts
    svm.airdrop(&admin.pubkey(), 10_000_000_000).unwrap();
    svm.airdrop(&seller.pubkey(), 10_000_000_000).unwrap();

    // Step 1: Initialize market
    let (market_pda, _) = Pubkey::find_program_address(&[b"market"], &program_id);

    let mut init_market_data = Vec::new();
    init_market_data.extend_from_slice(&anchor_discriminator("global", "init_market"));
    init_market_data.extend_from_slice(&fee_recipient.pubkey().to_bytes());
    init_market_data.extend_from_slice(&200u64.to_le_bytes()); // minting_fee_bps
    init_market_data.extend_from_slice(&100u64.to_le_bytes()); // maker_fee_bps
    init_market_data.extend_from_slice(&150u64.to_le_bytes()); // taker_fee_bps

    let init_market_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(admin.pubkey(), true),
            AccountMeta::new(market_pda, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data: init_market_data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[init_market_ix],
        Some(&admin.pubkey()),
        &[&admin],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).expect("Market initialization should succeed");
    println!("✓ Market initialized");

    // Step 2: Create collection
    let collection_name = "Test Collection";
    let collection_uri = "https://example.com/collection.json";

    let mut create_collection_data = Vec::new();
    create_collection_data.extend_from_slice(&anchor_discriminator("global", "create_collection"));
    
    let name_bytes = collection_name.as_bytes();
    create_collection_data.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
    create_collection_data.extend_from_slice(name_bytes);
    
    let uri_bytes = collection_uri.as_bytes();
    create_collection_data.extend_from_slice(&(uri_bytes.len() as u32).to_le_bytes());
    create_collection_data.extend_from_slice(uri_bytes);

    let create_collection_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(collection.pubkey(), true),
            AccountMeta::new_readonly(update_authority.pubkey(), true),
            AccountMeta::new(seller.pubkey(), true),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new_readonly(mpl_core_id, false),
        ],
        data: create_collection_data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[create_collection_ix],
        Some(&seller.pubkey()),
        &[&seller, &collection, &update_authority],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).expect("Collection creation should succeed");
    println!("✓ Collection created: {}", collection.pubkey());

    // Step 3: Mint an asset from the collection using mpl-core
    let asset_name = "Test NFT #1";
    let asset_uri = "https://example.com/nft1.json";

    // Convert Pubkeys to solana_program::pubkey::Pubkey for mpl-core compatibility
    let asset_pubkey_sp = solana_program::pubkey::Pubkey::new_from_array(asset.pubkey().to_bytes());
    let collection_pubkey_sp = solana_program::pubkey::Pubkey::new_from_array(collection.pubkey().to_bytes());
    let update_authority_pubkey_sp = solana_program::pubkey::Pubkey::new_from_array(update_authority.pubkey().to_bytes());
    let seller_pubkey_sp = solana_program::pubkey::Pubkey::new_from_array(seller.pubkey().to_bytes());
    let system_program_sp = solana_program::pubkey::Pubkey::new_from_array(SYSTEM_PROGRAM_ID.to_bytes());

    // Use mpl-core's instruction builder
    let create_asset_ix_mpl = mpl_core::instructions::CreateV2Builder::new()
        .asset(asset_pubkey_sp)
        .collection(Some(collection_pubkey_sp))
        .authority(Some(update_authority_pubkey_sp))
        .payer(seller_pubkey_sp)
        .owner(Some(seller_pubkey_sp))
        .system_program(system_program_sp)
        .name(asset_name.to_string())
        .uri(asset_uri.to_string())
        .instruction();

    // Convert back to solana_sdk::instruction::Instruction
    let create_asset_ix = Instruction {
        program_id: Pubkey::new_from_array(create_asset_ix_mpl.program_id.to_bytes()),
        accounts: create_asset_ix_mpl.accounts.iter().map(|meta| {
            AccountMeta {
                pubkey: Pubkey::new_from_array(meta.pubkey.to_bytes()),
                is_signer: meta.is_signer,
                is_writable: meta.is_writable,
            }
        }).collect(),
        data: create_asset_ix_mpl.data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[create_asset_ix],
        Some(&seller.pubkey()),
        &[&seller, &asset, &update_authority],
        svm.latest_blockhash(),
    );

    let asset_result = svm.send_transaction(tx);
    match &asset_result {
        Ok(tx_result) => {
            println!("✓ Asset created: {}", asset.pubkey());
            for log in &tx_result.logs {
                println!("  {}", log);
            }
        }
        Err(e) => {
            println!("Asset creation failed: {:?}", e);
        }
    }
    asset_result.expect("Asset creation should succeed");

    // Step 4: Create listing
    let listing_price: u64 = 1_000_000_000; // 1 SOL in lamports

    let (listing_pda, _) = Pubkey::find_program_address(
        &[
            b"listing",
            seller.pubkey().as_ref(),
            collection.pubkey().as_ref(),
            asset.pubkey().as_ref(),
        ],
        &program_id,
    );

    let mut add_listing_data = Vec::new();
    add_listing_data.extend_from_slice(&anchor_discriminator("global", "add_listing"));
    add_listing_data.extend_from_slice(&listing_price.to_le_bytes());

    let add_listing_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(seller.pubkey(), true),           // seller
            AccountMeta::new(listing_pda, false),              // listing PDA
            AccountMeta::new_readonly(market_pda, false),      // market
            AccountMeta::new_readonly(collection.pubkey(), false), // collection
            AccountMeta::new_readonly(asset.pubkey(), false),  // asset
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false), // system_program
        ],
        data: add_listing_data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[add_listing_ix],
        Some(&seller.pubkey()),
        &[&seller],
        svm.latest_blockhash(),
    );

    let result = svm.send_transaction(tx);

    // Check result
    let tx_result = result.expect("Listing creation should succeed");

    println!("✓ Listing created successfully!");
    println!("\nTransaction logs:");
    for log in &tx_result.logs {
        println!("  {}", log);
    }

    // Verify listing account was created
    let listing_account = svm.get_account(&listing_pda);
    assert!(listing_account.is_some(), "Listing account should exist");

    println!("\n=== Listing Details ===");
    println!("Listing PDA: {}", listing_pda);
    println!("Seller: {}", seller.pubkey());
    println!("Collection: {}", collection.pubkey());
    println!("Asset: {}", asset.pubkey());
    println!("Price: {} lamports ({} SOL)", listing_price, listing_price as f64 / 1_000_000_000.0);
}