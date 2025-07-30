use anchor_lang::InstructionData;
use colored::*;
use indicatif::ProgressBar;
use mor_core::instruction;
use sha3::{Digest, Sha3_256};
use solana_client::rpc_client::RpcClient;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::{signature::Signer, transaction::Transaction};
use solana_system_interface::program::ID as SYSTEM_PROGRAM_ID;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::Duration;

use crate::utils::helpers::{
    CLUSTER_URL, EPOCH_SLOT_LENGTH, ER_CLUSTER_URL, MINER_SEED, MINING_POOL_REWARD_SEED,
    MINING_POOL_SEED, PROGRAM_ID, load_payer_keypair,
};
use std::str::FromStr;

// Define the Rust struct for the miner account (must match on-chain layout)
use anchor_lang::AnchorDeserialize;

#[derive(Debug, AnchorDeserialize)]
pub struct Miner {
    _authority: Pubkey,
    _last_epoch_mined: u64,
    _rewards: u64,
    _multiplier: u8,
    _staked_amount: u64,
    difficulty: u8,
    _pool: Pubkey,
    _last_staked_timestamp: i64,
    _bump: u8,
}

pub fn handle_mine(token_mint: String) -> Result<(), Box<dyn std::error::Error>> {
    // Parse token mint address
    let mint_pubkey = Pubkey::from_str(&token_mint)?;

    println!("Mining with token mint: {}", mint_pubkey);
    let payer = load_payer_keypair().map_err(|e| format!("Failed to load keypair: {}", e))?;

    let er_client = RpcClient::new(ER_CLUSTER_URL);
    let base_client = RpcClient::new(CLUSTER_URL);

    // Derive PDAs following the current mor-core structure
    let (mining_pool_pda, _) = Pubkey::find_program_address(
        &[
            MINING_POOL_SEED,
            payer.pubkey().as_ref(),
            mint_pubkey.as_ref(),
        ],
        &PROGRAM_ID,
    );

    println!("Mining pool PDA: {}", mining_pool_pda.to_string());

    let (mining_pool_reward_state, _) = Pubkey::find_program_address(
        &[
            MINING_POOL_REWARD_SEED,
            payer.pubkey().as_ref(),
            mining_pool_pda.as_ref(),
        ],
        &PROGRAM_ID,
    );

    let (miner_pubkey, _) = Pubkey::find_program_address(
        &[
            MINER_SEED,
            payer.pubkey().as_ref(),
            mining_pool_pda.as_ref(),
        ],
        &PROGRAM_ID,
    );

    // Ctrl+C handling
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let mut last_epoch_mined: Option<u64> = None;

    while running.load(Ordering::SeqCst) {
        // Fetch miner account initially from er_client, if it fails, try base_client
        let miner_account = match er_client.get_account_data(&miner_pubkey) {
            Ok(account_data) => {
                let data_slice = &account_data[1..];
                match Miner::try_from_slice(data_slice) {
                    Ok(miner_account) => miner_account,
                    Err(e) => {
                        println!(
                            "{} {}",
                            "Failed to deserialize miner account from er_client:".red(),
                            e
                        );
                        break;
                    }
                }
            }
            Err(_) => match base_client.get_account_data(&miner_pubkey) {
                Ok(account_data) => {
                    let data_slice = &account_data[1..];
                    match Miner::try_from_slice(data_slice) {
                        Ok(miner_account) => miner_account,
                        Err(e) => {
                            println!(
                                "{} {}",
                                "Failed to deserialize miner account from base:".red(),
                                e
                            );
                            break;
                        }
                    }
                }
                Err(e) => {
                    println!("{} {}", "Failed to fetch miner account:".red(), e);
                    break;
                }
            },
        };

        let difficulty = miner_account.difficulty as usize;
        println!("{} {}", "Miner difficulty:".cyan(), difficulty);

        // Get current slot and compute epoch
        let slot = match er_client.get_slot() {
            Ok(s) => s,
            Err(e) => {
                println!("{} {}", "Failed to fetch slot:".red(), e);
                break;
            }
        };
        let epoch = slot / EPOCH_SLOT_LENGTH;
        println!("{} {}", "Current epoch:".cyan(), epoch);

        // Only mine if epoch has advanced
        if let Some(last) = last_epoch_mined {
            if epoch == last {
                thread::sleep(Duration::from_secs(5));
                continue;
            }
        }
        last_epoch_mined = Some(epoch);

        // Generate challenge
        let mut epoch_buf = [0u8; 8];
        epoch_buf.copy_from_slice(&epoch.to_le_bytes());
        let mut hasher = Sha3_256::new();
        hasher.update(payer.pubkey().to_bytes());
        hasher.update(epoch_buf);
        let challenge = hasher.finalize_reset();

        // Find valid nonce
        let spinner = ProgressBar::new_spinner();
        spinner.set_message("Searching for valid nonce...");
        spinner.enable_steady_tick(Duration::from_millis(100));
        let mut nonce: u64 = 0;
        let found_nonce = loop {
            if !running.load(Ordering::SeqCst) {
                spinner.finish_and_clear();
                println!("{}", "Mining interrupted by user.".yellow());
                return Ok(());
            }
            let mut nonce_buf = [0u8; 8];
            nonce_buf.copy_from_slice(&nonce.to_le_bytes());
            let mut hasher = Sha3_256::new();
            hasher.update(&challenge);
            hasher.update(nonce_buf);
            let hash = hasher.finalize();
            if hash[..difficulty].iter().all(|&b| b == 0) {
                break nonce;
            }
            nonce = nonce.wrapping_add(1);
            if nonce % 100_000 == 0 {
                spinner.set_message(format!("Tried {} nonces...", nonce));
            }
        };
        spinner.finish_and_clear();
        println!("{} {}", "Found valid nonce:".green(), found_nonce);

        // Prepare instruction - match the current submit_solution structure
        let instruction_data = instruction::SubmitSolution::data(&instruction::SubmitSolution {
            nonce: found_nonce,
            epoch,
        });
        let signers = &[&payer];
        let accounts = vec![
            AccountMeta::new(miner_pubkey, false),               // miner
            AccountMeta::new_readonly(mining_pool_pda, false),   // mining_pool_pda
            AccountMeta::new(mining_pool_reward_state, false),   // mining_pool_reward_state
            AccountMeta::new(payer.pubkey(), true),              // authority
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false), // system_program
        ];
        let ix = Instruction {
            program_id: PROGRAM_ID,
            accounts,
            data: instruction_data,
        };
        let blockhash = match er_client.get_latest_blockhash() {
            Ok(bh) => bh,
            Err(e) => {
                println!("{} {}", "Failed to get recent blockhash:".red(), e);
                break;
            }
        };
        let tx =
            Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), signers, blockhash);

        let pb = ProgressBar::new_spinner();
        pb.set_message("Submitting solution...");
        pb.enable_steady_tick(Duration::from_millis(100));
        let result = er_client.send_and_confirm_transaction(&tx);
        pb.finish_and_clear();
        match result {
            Ok(sig) => println!(
                "{} {}{}{}",
                "Transaction sent successfully:".green(),
                "https://explorer.solana.com/tx/".cyan(),
                sig.to_string().cyan(),
                "?cluster=custom&customUrl=http%3A%2F%2Flocalhost%3A7799".cyan()
            ),
            Err(e) => println!("{} {}", "Transaction failed:".red(), e),
        }
        // Print after every success, then wait for next epoch
        println!("{} {}", "Waiting for next epoch...".yellow(), epoch + 1);
        thread::sleep(Duration::from_secs(5));
    }
    println!("{}", "Mining stopped.");
    Ok(())
}
