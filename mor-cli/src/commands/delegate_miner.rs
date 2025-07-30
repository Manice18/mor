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
    CLUSTER_URL, DELEGATION_PROGRAM_ID, MINER_SEED, MINING_POOL_SEED, PROGRAM_ID,
    load_payer_keypair,
};

pub fn handle_delegate_miner(token_mint: String) {
    let payer = load_payer_keypair().unwrap();
    let client = RpcClient::new(CLUSTER_URL);

    // Parse the token mint address
    let mint_pubkey = match token_mint.parse::<Pubkey>() {
        Ok(pubkey) => pubkey,
        Err(e) => {
            println!("{} {}", "Invalid token mint address:".red(), e);
            return;
        }
    };

    // Derive mining pool PDA - assuming the payer is also the pool maker
    let (mining_pool_pda, _bump) = Pubkey::find_program_address(
        &[
            MINING_POOL_SEED,
            &payer.pubkey().to_bytes(),
            &mint_pubkey.to_bytes(),
        ],
        &PROGRAM_ID,
    );

    // Derive miner PDA
    let (miner_pda, _bump) = Pubkey::find_program_address(
        &[
            MINER_SEED,
            &payer.pubkey().to_bytes(),
            &mining_pool_pda.to_bytes(),
        ],
        &PROGRAM_ID,
    );

    // Create the instruction data
    let instruction_data = instruction::Delegate {}.data();

    // Derive PDA for buffer mining pool account
    let (buffer_miner_pda_pubkey, _bump) =
        Pubkey::find_program_address(&[b"buffer", miner_pda.as_ref()], &PROGRAM_ID);

    // Derive PDA for delegation record mining pool account
    let (delegation_record_miner_pda_pubkey, _bump) =
        Pubkey::find_program_address(&[b"delegation", miner_pda.as_ref()], &DELEGATION_PROGRAM_ID);

    // Derive PDA for delegation metadata mining pool account
    let (delegation_miner_pda_pubkey, _bump) = Pubkey::find_program_address(
        &[b"delegation-metadata", miner_pda.as_ref()],
        &DELEGATION_PROGRAM_ID,
    );

    let accounts = vec![
        AccountMeta::new(payer.pubkey(), true), // payer
        AccountMeta::new(buffer_miner_pda_pubkey, false),
        AccountMeta::new(delegation_record_miner_pda_pubkey, false),
        AccountMeta::new(delegation_miner_pda_pubkey, false),
        AccountMeta::new(miner_pda, false),       // miner_pda
        AccountMeta::new(mining_pool_pda, false), // mining_pool_pda
        AccountMeta::new(PROGRAM_ID, false),
        AccountMeta::new(DELEGATION_PROGRAM_ID, false), // delegation program
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
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], blockhash);

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
