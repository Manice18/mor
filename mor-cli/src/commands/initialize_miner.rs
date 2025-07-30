use anchor_lang::InstructionData;
use colored::*;
use indicatif::ProgressBar;
use mor_core::instruction;
use solana_client::rpc_client::RpcClient;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::{signature::Signer, transaction::Transaction};
use solana_system_interface::program::ID as SYSTEM_PROGRAM_ID;

use crate::utils::helpers::{
    CLUSTER_URL, MINER_SEED, MINING_POOL_SEED, PROGRAM_ID, load_payer_keypair,
};

pub fn handle_initialize_miner(token_mint: String) {
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
            &authority.pubkey().as_ref(),
            &mint_pubkey.to_bytes(),
        ],
        &PROGRAM_ID,
    );

    // Derive miner PDA
    let (miner_pda, _bump) = Pubkey::find_program_address(
        &[
            MINER_SEED,
            &authority.pubkey().as_ref(),
            &mining_pool_pda.to_bytes(),
        ],
        &PROGRAM_ID,
    );

    // Create the instruction data
    let instruction_data = instruction::InitializeMiner {}.data();

    let accounts = vec![
        AccountMeta::new(authority.pubkey(), true), // authority
        AccountMeta::new(miner_pda, false),         // miner
        AccountMeta::new(mining_pool_pda, false),   // mining_pool_pda
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false), // system_program
    ];

    // Prepare the instruction
    let ix = Instruction {
        program_id: PROGRAM_ID,
        accounts,
        data: instruction_data,
    };

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
        &[ix],
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
