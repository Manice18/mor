use anchor_lang::InstructionData;
use colored::*;
use indicatif::ProgressBar;
use mor_core::instruction;
use solana_client::rpc_client::RpcClient;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::{signature::Signer, transaction::Transaction};
use solana_system_interface::program::ID as SYSTEM_PROGRAM_ID;
use spl_associated_token_account::get_associated_token_address;
use spl_associated_token_account::instruction::create_associated_token_account;

use crate::utils::helpers::{
    CLUSTER_URL, MINER_SEED, MINING_POOL_SEED, PROGRAM_ID, load_payer_keypair,
};

pub fn handle_claim_rewards(token_mint: String) {
    let authority = load_payer_keypair().unwrap();
    let client = RpcClient::new(CLUSTER_URL);

    // Parse the token mint address
    let mint_pubkey = match token_mint.parse::<Pubkey>() {
        Ok(pubkey) => pubkey,
        Err(e) => {
            println!("{} {}", "Invalid token mint address:".red(), e);
            return;
        }
    };

    // Derive mining pool PDA - assuming the authority is also the pool maker
    let (mining_pool_pda, _bump) = Pubkey::find_program_address(
        &[
            MINING_POOL_SEED,
            &authority.pubkey().to_bytes(),
            &mint_pubkey.to_bytes(),
        ],
        &PROGRAM_ID,
    );

    // Derive miner PDA
    let (miner_pda, _bump) = Pubkey::find_program_address(
        &[
            MINER_SEED,
            &authority.pubkey().to_bytes(),
            &mining_pool_pda.to_bytes(),
        ],
        &PROGRAM_ID,
    );

    // Get vault (mining pool's token account)
    let vault = get_associated_token_address(&mining_pool_pda, &mint_pubkey);

    // Get recipient ATA (authority's token account)
    let recipient_ata = get_associated_token_address(&authority.pubkey(), &mint_pubkey);

    let mut instructions = vec![];

    // Check if recipient ATA exists, if not create it
    match client.get_account(&recipient_ata) {
        Ok(_) => {
            // ATA already exists
        }
        Err(_) => {
            // Create ATA instruction
            let create_ata_ix = create_associated_token_account(
                &authority.pubkey(),
                &authority.pubkey(),
                &mint_pubkey,
                &spl_token::ID,
            );
            instructions.push(create_ata_ix);
        }
    }

    // Create the claim rewards instruction data
    let instruction_data = instruction::ClaimRewards {}.data();

    let accounts = vec![
        AccountMeta::new(authority.pubkey(), true), // authority
        AccountMeta::new(miner_pda, false),         // miner
        AccountMeta::new(mining_pool_pda, false),   // mining_pool_pda
        AccountMeta::new(vault, false),             // vault
        AccountMeta::new(mint_pubkey, false),       // mint
        AccountMeta::new(recipient_ata, false),     // recipient_ata
        AccountMeta::new(spl_associated_token_account::ID, false), // associated_token_program
        AccountMeta::new(spl_token::ID, false),     // token_program
        AccountMeta::new(SYSTEM_PROGRAM_ID, false), // system_program
    ];

    // Prepare the claim instruction
    let claim_ix = Instruction {
        program_id: PROGRAM_ID,
        accounts,
        data: instruction_data,
    };

    instructions.push(claim_ix);

    // Get recent blockhash
    let blockhash = match client.get_latest_blockhash() {
        Ok(bh) => bh,
        Err(e) => {
            println!("{} {}", "Failed to get recent blockhash:".red(), e);
            return;
        }
    };

    // Build transaction
    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&authority.pubkey()),
        &[&authority],
        blockhash,
    );

    let pb = ProgressBar::new_spinner();
    pb.set_message("Sending and confirming transaction...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let result = client.send_and_confirm_transaction(&tx);

    pb.finish_and_clear();

    match result {
        Ok(sig) => println!(
            "{} {}{}{}",
            "Transaction sent successfully:".green(),
            "https://explorer.solana.com/tx/".to_string().cyan(),
            sig.to_string().cyan(),
            "?cluster=custom&customUrl=http%3A%2F%2Flocalhost%3A8899"
                .to_string()
                .cyan()
        ),
        Err(e) => println!("{} {}", "Transaction failed:".red(), e),
    }
}
