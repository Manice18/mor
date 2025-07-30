// Helps to get the account details of a miner account

use anchor_lang::AnchorDeserialize;
use colored::*;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;

use crate::utils::helpers::{
    CLUSTER_URL, MINER_SEED, MINING_POOL_SEED, PROGRAM_ID, load_payer_keypair,
};

#[derive(Debug, AnchorDeserialize)]
pub struct MinerAccountPoolPda {
    pub authority: Pubkey,
    pub last_epoch_mined: u64,
    pub rewards: u64,
    pub multiplier: u8,
    pub staked_amount: u64,
    pub difficulty: u8,
    pub pool: Pubkey,
    pub last_staked_timestamp: i64,
    pub bump: u8,
}

pub fn handle_get_account(token_mint: String) {
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

    match client.get_account_data(&miner_pda) {
        Ok(account_data) => {
            // Skip the first 8 bytes (discriminator) for Anchor accounts
            let data_slice = &account_data[1..];

            match MinerAccountPoolPda::try_from_slice(data_slice) {
                Ok(miner_account) => {
                    println!("{}", "Miner Account Details:".green().bold());
                    println!("{} {}", "Miner PDA:".cyan(), miner_pda);
                    println!("{} {}", "Authority:".cyan(), miner_account.authority);
                    println!("{} {}", "Mining Pool:".cyan(), miner_account.pool);
                    if miner_account.last_epoch_mined == u64::MAX {
                        println!("{} {}", "Last Epoch Mined:".cyan(), "Never mined".yellow());
                    } else {
                        println!(
                            "{} {}",
                            "Last Epoch Mined:".cyan(),
                            miner_account.last_epoch_mined
                        );
                    }
                    println!("{} {}", "Rewards:".cyan(), miner_account.rewards);
                    println!("{} {}", "Multiplier:".cyan(), miner_account.multiplier);
                    println!(
                        "{} {}",
                        "Staked Amount:".cyan(),
                        miner_account.staked_amount
                    );
                    println!("{} {}", "Difficulty:".cyan(), miner_account.difficulty);
                    if miner_account.last_staked_timestamp == 0 {
                        println!(
                            "{} {}",
                            "Last Staked Timestamp:".cyan(),
                            "Never staked".yellow()
                        );
                    } else {
                        println!(
                            "{} {}",
                            "Last Staked Timestamp:".cyan(),
                            miner_account.last_staked_timestamp
                        );
                    }
                    println!("{} {}", "Bump:".cyan(), miner_account.bump);
                }
                Err(e) => {
                    println!("{} {}", "Failed to deserialize miner account:".red(), e);
                }
            }
        }
        Err(e) => {
            println!("{} {}", "Failed to fetch miner account:".red(), e);
            println!(
                "{}",
                "This might mean the miner account hasn't been initialized yet.".yellow()
            );
            println!("{}", "Try running the create-account command first.".cyan());
        }
    }
}
