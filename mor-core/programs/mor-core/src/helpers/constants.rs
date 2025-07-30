use anchor_lang::prelude::*;

#[constant]
pub const MINER_SEED: &[u8] = b"miner";

#[constant]
pub const MINING_POOL_SEED: &[u8] = b"mining_pool";

#[constant]
pub const MINING_POOL_REWARD_SEED: &[u8] = b"mining_pool_reward";

#[constant]
pub const MINT_AUTHORITY_SEED: &[u8] = b"mint";

#[constant]
pub const EPOCH_SLOT_LENGTH: u64 = 150;
