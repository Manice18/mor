use anchor_lang::prelude::*;

#[derive(InitSpace)]
#[account(discriminator = 1)]
pub struct MiningPoolPda {
    pub pool_maker: Pubkey,
    pub mint: Pubkey,
    pub bump: u8,
    pub amount: u64,
}

#[derive(InitSpace)]
#[account(discriminator = 2)]
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

#[derive(InitSpace)]
#[account(discriminator = 3)]
pub struct MiningPoolRewardState {
    pub pool_pda: Pubkey,
    pub amount: u64,
    pub bump: u8,
}
