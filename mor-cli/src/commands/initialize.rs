// to be executed by admin of the platform
use anchor_lang::InstructionData;
use colored::*;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Signer,
    transaction::Transaction,
};
use solana_system_interface::program::ID as SYSTEM_PROGRAM_ID;
use spl_associated_token_account::get_associated_token_address;
use std::str::FromStr;

use mor_core::instruction::{DelegateRewardPool, InitializePool};

use crate::utils::helpers::{
    DELEGATION_PROGRAM_ID, MINING_POOL_REWARD_SEED, MINING_POOL_SEED, PROGRAM_ID,
    load_payer_keypair,
};

// Define the constants locally since they're not exported from helpers
// const MINING_POOL_SEED: &str = "mining_pool";
// const MINING_POOL_REWARD_SEED: &str = "mining_pool_reward";

pub fn handle_initialize_pool(
    amount: u64,
    token_mint: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let rpc_client = RpcClient::new("http://localhost:8899");

    // Parse token mint address
    let mint_pubkey = Pubkey::from_str(&token_mint)?;

    // Load keypair from file using the local path
    let payer = load_payer_keypair().unwrap();

    // Derive PDAs
    let (mining_pool_pda, _) = Pubkey::find_program_address(
        &[
            MINING_POOL_SEED,
            payer.pubkey().as_ref(),
            mint_pubkey.as_ref(),
        ],
        &mor_core::ID,
    );

    let (mining_pool_reward_state, _) = Pubkey::find_program_address(
        &[
            MINING_POOL_REWARD_SEED,
            payer.pubkey().as_ref(),
            mining_pool_pda.as_ref(),
        ],
        &mor_core::ID,
    );

    // Get associated token accounts
    let pool_maker_ata = get_associated_token_address(&payer.pubkey(), &mint_pubkey);
    let vault = get_associated_token_address(&mining_pool_pda, &mint_pubkey);

    // Build instruction
    let instruction_data = InitializePool { amount };

    let accounts = vec![
        AccountMeta::new(payer.pubkey(), true),   // pool_maker
        AccountMeta::new(mining_pool_pda, false), // mining_pool_pda
        AccountMeta::new(mining_pool_reward_state, false), // mining_pool_reward_state
        AccountMeta::new_readonly(mint_pubkey, false), // mint
        AccountMeta::new(pool_maker_ata, false),  // pool_maker_ata
        AccountMeta::new(vault, false),           // vault
        AccountMeta::new_readonly(spl_associated_token_account::ID, false), // associated_token_program
        AccountMeta::new_readonly(spl_token::ID, false),                    // token_program
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),                // system_program
    ];

    let instruction = Instruction {
        program_id: mor_core::ID,
        accounts,
        data: instruction_data.data(),
    };

    // Create and send transaction
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    match rpc_client.send_and_confirm_transaction_with_spinner(&transaction) {
        Ok(signature) => {
            println!(
                "Successfully initialized mining pool for token mint: {}",
                token_mint
            );
            println!(
                "{} {}{}{}",
                "Transaction sent successfully:".green(),
                "https://explorer.solana.com/tx/".to_string().cyan(),
                signature.to_string().cyan(),
                "?cluster=custom&customUrl=http%3A%2F%2Flocalhost%3A8899"
                    .to_string()
                    .cyan()
            );
            println!("Mining pool PDA: {}", mining_pool_pda);
            println!("Mining pool reward state PDA: {}", mining_pool_reward_state);

            // Now delegate the reward pool to Ephemeral Rollup
            println!("\nDelegating reward pool to Ephemeral Rollup...");
            match delegate_reward_pool_to_er(
                &rpc_client,
                &payer,
                mining_pool_pda,
                mining_pool_reward_state,
            ) {
                Ok(delegate_signature) => {
                    println!("Successfully delegated reward pool to ER!");
                    println!(
                        "{} {}{}{}",
                        "Delegate Transaction signature:".green(),
                        "https://explorer.solana.com/tx/".to_string().cyan(),
                        delegate_signature.to_string().cyan(),
                        "?cluster=custom&customUrl=http%3A%2F%2Flocalhost%3A8899"
                            .to_string()
                            .cyan()
                    );
                }
                Err(e) => {
                    eprintln!("Warning: Failed to delegate reward pool to ER: {}", e);
                    eprintln!("Pool initialization was successful, but delegation failed.");
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to send transaction: {}", e);
            return Err(Box::new(e));
        }
    }

    Ok(())
}

// Function to delegate reward pool to Ephemeral Rollup
fn delegate_reward_pool_to_er(
    rpc_client: &RpcClient,
    payer: &solana_sdk::signature::Keypair,
    mining_pool_pda: Pubkey,
    mining_pool_reward_state: Pubkey,
) -> Result<solana_sdk::signature::Signature, Box<dyn std::error::Error>> {
    // Build delegate reward pool instruction
    let instruction_data = DelegateRewardPool {};

    // Derive PDA for buffer mining pool account
    let (buffer_mining_pool_reward_state_pubkey, _bump) =
        Pubkey::find_program_address(&[b"buffer", mining_pool_reward_state.as_ref()], &PROGRAM_ID);

    // Derive PDA for delegation record mining pool account
    let (delegation_record_mining_pool_reward_state_pubkey, _bump) = Pubkey::find_program_address(
        &[b"delegation", mining_pool_reward_state.as_ref()],
        &DELEGATION_PROGRAM_ID,
    );

    // Derive PDA for delegation metadata mining pool account
    let (delegation_mining_pool_reward_state_pubkey, _bump) = Pubkey::find_program_address(
        &[b"delegation-metadata", mining_pool_reward_state.as_ref()],
        &DELEGATION_PROGRAM_ID,
    );

    let accounts = vec![
        AccountMeta::new(payer.pubkey(), true), // payer
        AccountMeta::new(buffer_mining_pool_reward_state_pubkey, false),
        AccountMeta::new(delegation_record_mining_pool_reward_state_pubkey, false),
        AccountMeta::new(delegation_mining_pool_reward_state_pubkey, false),
        AccountMeta::new(mining_pool_reward_state, false), // mining_pool_reward_state
        AccountMeta::new_readonly(mining_pool_pda, false), // mining_pool_pda
        AccountMeta::new(PROGRAM_ID, false),
        AccountMeta::new(DELEGATION_PROGRAM_ID, false), // delegation program
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false), // system_program
    ];

    let instruction = Instruction {
        program_id: mor_core::ID,
        accounts,
        data: instruction_data.data(),
    };

    // Create and send transaction
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
        &[payer],
        recent_blockhash,
    );

    let signature = rpc_client.send_and_confirm_transaction_with_spinner(&transaction)?;
    Ok(signature)
}
