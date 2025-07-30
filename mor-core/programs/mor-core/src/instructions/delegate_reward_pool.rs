use crate::{
    helpers::constants::{MINER_SEED, MINING_POOL_REWARD_SEED, MINING_POOL_SEED},
    helpers::errors::MorErrorCodes,
    states::{MiningPoolPda, MiningPoolRewardState},
};
use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::delegate;
use ephemeral_rollups_sdk::cpi::DelegateConfig;

#[delegate]
#[derive(Accounts)]
#[instruction()]
pub struct DelegateRewardPool<'info> {
    pub payer: Signer<'info>, // pool maker
    /// CHECK The pda to delegate
    #[account(
        mut,
        del,
        seeds = [MINING_POOL_REWARD_SEED, mining_pool_pda.pool_maker.key().as_ref(), mining_pool_pda.key().as_ref()],
        bump,
    )]
    pub mining_pool_reward_state: AccountInfo<'info>,

    #[account(
        seeds = [MINING_POOL_SEED, mining_pool_pda.pool_maker.key().as_ref(), mining_pool_pda.mint.key().as_ref()],
        bump = mining_pool_pda.bump,
      )]
    pub mining_pool_pda: Account<'info, MiningPoolPda>,
}

impl<'info> DelegateRewardPool<'info> {
    fn delegate_reward_pool(&mut self) -> Result<()> {
        // Only the mining pool's pool_maker can delegate the reward pool
        require!(
            self.payer.key() == self.mining_pool_pda.pool_maker,
            MorErrorCodes::InvalidAuthority
        );

        let mining_pool_pda_key = self.mining_pool_pda.key();
        // delegate the session account to the ER
        let payer_key = self.payer.key();
        let mining_pool_reward_state_seeds = [
            MINING_POOL_REWARD_SEED,
            payer_key.as_ref(),
            mining_pool_pda_key.as_ref(),
        ];
        self.delegate_mining_pool_reward_state(
            &self.payer,
            &mining_pool_reward_state_seeds,
            DelegateConfig::default(),
        )?;
        Ok(())
    }
}

pub fn delegate_reward_pool_handler(ctx: Context<DelegateRewardPool>) -> Result<()> {
    ctx.accounts.delegate_reward_pool()?;

    Ok(())
}
