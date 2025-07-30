use anchor_lang::prelude::*;
use sha3::{Digest, Sha3_256};
use crate::{
    helpers::{constants::{EPOCH_SLOT_LENGTH, MINER_SEED, MINING_POOL_REWARD_SEED, MINING_POOL_SEED}, errors::MorErrorCodes, utils::generate_challenge},
    states::{MinerAccountPoolPda, MiningPoolPda, MiningPoolRewardState},
};

#[derive(Accounts)]
#[instruction()]
pub struct SubmitSolution<'info> {
    #[account(
        init_if_needed,
        space = MinerAccountPoolPda::INIT_SPACE + MinerAccountPoolPda::DISCRIMINATOR.len(),
        payer = authority, 
        seeds = [MINER_SEED, authority.key().as_ref(), mining_pool_pda.key().as_ref()], 
        bump
    )]
    pub miner: Account<'info, MinerAccountPoolPda>,

    #[account(
        seeds = [MINING_POOL_SEED, mining_pool_pda.pool_maker.key().as_ref(), mining_pool_pda.mint.key().as_ref()],
        bump = mining_pool_pda.bump,
      )]
    pub mining_pool_pda: Account<'info, MiningPoolPda>,

    #[account(
        init_if_needed,
        payer = authority,
        space = MiningPoolRewardState::INIT_SPACE + MiningPoolRewardState::DISCRIMINATOR.len(),
        seeds = [MINING_POOL_REWARD_SEED, mining_pool_pda.pool_maker.key().as_ref(), mining_pool_pda.key().as_ref()],
        bump,
    )]
    pub mining_pool_reward_state: Account<'info, MiningPoolRewardState>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> SubmitSolution<'info> {
    fn submit_solution(&mut self, nonce: u64, epoch: u64) -> Result<()> {
        let miner = &mut self.miner;
        miner.authority = self.authority.key();

        let slot = Clock::get()?.slot;
        let current_epoch = slot / EPOCH_SLOT_LENGTH;

        msg!(
            "Epoch Range Elapsed: {:?}",
            (epoch as i64 - current_epoch as i64).abs()
        );

        // Only allow epoch that is within ±4 of the current
        require!(
            (epoch as i64 - current_epoch as i64).abs() <= 4,
            MorErrorCodes::InvalidEpochRange
        );

        // Ensure user hasn't mined this epoch already // TODO: uncomment this when we update difficulty logic
        // require!(epoch != miner.last_epoch_mined, MorErrorCodes::AlreadyMined);

        msg!("nonce: {}", nonce);
        msg!("slot: {}", slot);
        msg!("epoch: {}", epoch);

        let challenge = generate_challenge(&miner.authority, epoch);
        let mut hasher = Sha3_256::new();
        hasher.update(&challenge);
        hasher.update(&nonce.to_le_bytes());
        let result = hasher.finalize();
        msg!("RESULT: {:?}", result);

        for i in 0..miner.difficulty {
            if result[i as usize] != 0 {
                return err!(MorErrorCodes::InvalidSolution);
            }
        }

        // Check if mining pool has rewards available
        require!(
            self.mining_pool_reward_state.amount > 0,
            MorErrorCodes::NoRewardsAvailable
        );

        // Calculate reward amount based on available pool amount and miner multiplier
        // Use 5% of available pool amount as base reward, multiplied by miner's multiplier
        let base_reward_percentage = 5; // 5% of available pool amount
        let base_reward = (self.mining_pool_reward_state.amount * base_reward_percentage) / 100;
        let reward_amount = base_reward * miner.multiplier as u64;
        
        // Ensure we don't exceed the available amount in the pool
        let actual_reward = if reward_amount > self.mining_pool_reward_state.amount {
            self.mining_pool_reward_state.amount
        } else {
            reward_amount
        };

        // Update miner rewards and reduce pool amount
        miner.rewards += actual_reward;
        self.mining_pool_reward_state.amount = self.mining_pool_reward_state.amount.saturating_sub(actual_reward);
        
        miner.last_epoch_mined = epoch;
        msg!("miner rewards: {}", miner.rewards);
        msg!("pool remaining amount: {}", self.mining_pool_reward_state.amount);

        // TODO: need to fix difficulty logic since above difficulty 3 it takes too long to mine
        // Simple adaptive difficulty: adjust based on how quickly the miner is solving epochs
        let target_epoch_time: u64 = EPOCH_SLOT_LENGTH; // Target: 1 epoch per EPOCH_SLOT_LENGTH slots
        if miner.last_epoch_mined != u64::MAX {
            let last_mined_slot = miner.last_epoch_mined * EPOCH_SLOT_LENGTH;
            let time_since_last = slot.saturating_sub(last_mined_slot);
            if time_since_last < target_epoch_time && miner.difficulty < 4 {
                miner.difficulty += 1;
                msg!("Increasing difficulty to {}", miner.difficulty);
            } else if time_since_last > target_epoch_time && miner.difficulty > 1 {
                miner.difficulty -= 1;
                msg!("Decreasing difficulty to {}", miner.difficulty);
            }
        }
        Ok(())
    }
}

pub fn submit_solution_handler(ctx: Context<SubmitSolution>, nonce: u64, epoch: u64) -> Result<()> {
    ctx.accounts.submit_solution(nonce, epoch)?;

    Ok(())
}
