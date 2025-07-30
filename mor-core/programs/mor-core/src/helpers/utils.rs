use anchor_lang::prelude::Pubkey;
use sha3::{Digest, Sha3_256};

pub fn generate_challenge(pubkey: &Pubkey, epoch: u64) -> [u8; 32] {
    let mut hasher = Sha3_256::new();
    hasher.update(pubkey.to_bytes());
    hasher.update(&epoch.to_le_bytes());
    hasher.finalize().into()
}

// Linear tier multiplier
pub fn calculate_multiplier(staked_tokens: u64, base_rate: u64) -> u64 {
    let staked_token_in_whole = staked_tokens.saturating_div(1_000_000_000);
    if staked_token_in_whole == 0 {
        return 0;
    }

    let tier_bonus = match staked_token_in_whole {
        1..=9 => staked_token_in_whole.saturating_div(2), // 0.5x per token (rounded down)
        10..=49 => {
            4_u64.saturating_add((staked_token_in_whole.saturating_sub(10)).saturating_div(2))
        } // 4 + 0.5x per token
        50..=99 => {
            24_u64.saturating_add((staked_token_in_whole.saturating_sub(50)).saturating_div(3))
        } // 24 + 0.33x per token
        _ => 40_u64.saturating_add((staked_tokens.saturating_sub(100)).saturating_div(5)), // 40 + 0.2x per token
    };

    base_rate.saturating_add(tier_bonus)
}
