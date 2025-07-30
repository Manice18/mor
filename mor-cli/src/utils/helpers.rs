use colored::*;
use mor_core::ID;
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, read_keypair_file};
use std::env;

pub const PROGRAM_ID: Pubkey = ID;

pub const CLUSTER_URL: &str = "http://localhost:8899";
pub const ER_CLUSTER_URL: &str = "http://localhost:7799";

pub const EPOCH_SLOT_LENGTH: u64 = 150;

// Seeds for PDAs
pub const MINING_POOL_SEED: &[u8] = b"mining_pool";
pub const MINING_POOL_REWARD_SEED: &[u8] = b"mining_pool_reward";
pub const MINER_SEED: &[u8] = b"miner";

pub const DELEGATION_PROGRAM_ID: Pubkey = pubkey!("DELeGGvXpWV2fqJUhqcF5ZSYMS4JTLjteaAMARRSaeSh");
pub const MAGIC_CONTEXT_PROGRAM_ID: Pubkey = pubkey!("MagicContext1111111111111111111111111111111");
pub const MAGIC_PROGRAM_ID: Pubkey = pubkey!("Magic11111111111111111111111111111111111111");

/// Loads the payer keypair from ~/.config/mor-supply/id.json
/// Returns Ok(Keypair) if found, or Err(String) with a colored error message if not.
pub fn load_payer_keypair() -> Result<Keypair, String> {
    let home_dir = env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
    let keypair_path = format!("{}/.config/mor-supply/id.json", home_dir);
    match read_keypair_file(&keypair_path) {
        Ok(kp) => Ok(kp),
        Err(_) => {
            let msg = format!(
                "{}",
                "Missing keypair. Please run setup-keypair first.".red()
            );
            Err(msg)
        }
    }
}
