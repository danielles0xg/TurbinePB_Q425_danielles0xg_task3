use litesvm::LiteSVM;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

// System program ID constant
const SYSTEM_PROGRAM_ID: Pubkey = Pubkey::new_from_array([0u8; 32]);


use nft_marketplace;

#[test]
fn init_market() {
    // Create the test environment
    let mut svm = LiteSVM::new();

    let program_id = Pubkey::new_from_array(nft_marketplace::ID.to_bytes());
    let program_bytes = include_bytes!("../../../target/deploy/nft_marketplace.so");
    svm.add_program(program_id, program_bytes);


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

    // Create your instruction
    // For now, create a simple instruction with empty data to test the program loading
    let instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(admin.pubkey(), true),
            AccountMeta::new(market_pda, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ],
        data: vec![],  // Empty data for now - would need proper Anchor instruction format
    };
        // Build transaction
        let tx = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&admin.pubkey()),
            &[&admin],
            svm.latest_blockhash(),
        );
        // Send transaction
        let result = svm.send_transaction(tx).unwrap();
        // Check transaction succeeded
        println!("Transaction logs: {:?}", result.logs);

}