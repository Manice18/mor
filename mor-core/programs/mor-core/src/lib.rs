use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::ephemeral;

pub mod helpers;
pub mod instructions;
pub mod states;

use instructions::*;

declare_id!("8BwLz8SvdFeT7qqd1nJFQMypTtuuWEpEEpVz6x6DA4Hm");

#[ephemeral]
#[program]
pub mod mor_core {
    use super::*;

    pub fn initialize_pool(ctx: Context<InitializePool>, amount: u64) -> Result<()> {
        instructions::initialize_pool::initialize_pool_handler(ctx, amount)
    }

    pub fn initialize_miner(ctx: Context<InitializeMiner>) -> Result<()> {
        instructions::initialize_miner::initialize_miner_handler(ctx)
    }

    pub fn delegate(ctx: Context<DelegateMinerAccount>) -> Result<()> {
        instructions::delegate::delegate_handler(ctx)
    }

    pub fn submit_solution(ctx: Context<SubmitSolution>, nonce: u64, epoch: u64) -> Result<()> {
        instructions::submit_solution::submit_solution_handler(ctx, nonce, epoch)
    }

    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        instructions::claim_rewards::claim_rewards_handler(ctx)
    }

    pub fn undelegate_miner(ctx: Context<UndelegateMinerAccount>) -> Result<()> {
        instructions::undelegate::undelegate_handler(ctx)
    }

    pub fn delegate_reward_pool(ctx: Context<DelegateRewardPool>) -> Result<()> {
        instructions::delegate_reward_pool::delegate_reward_pool_handler(ctx)
    }

    pub fn undelegate_reward_pool(ctx: Context<UndelegateRewardPool>) -> Result<()> {
        instructions::undelegate_reward_pool::undelegate_reward_pool_handler(ctx)
    }
}
