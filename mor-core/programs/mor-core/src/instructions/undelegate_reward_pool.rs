use crate::{
    helpers::constants::{MINING_POOL_REWARD_SEED, MINING_POOL_SEED},
    helpers::errors::MorErrorCodes,
    states::{MiningPoolPda, MiningPoolRewardState},
};
use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::commit;
use ephemeral_rollups_sdk::ephem::commit_and_undelegate_accounts;

#[commit]
#[derive(Accounts)]
#[instruction()]
pub struct UndelegateRewardPool<'info> {
    pub payer: Signer<'info>,
    /// CHECK The pda to undelegate
    #[account(
        mut,
        seeds = [MINING_POOL_REWARD_SEED, payer.key().as_ref(), mining_pool_pda.key().as_ref()],
        bump = mining_pool_reward_state.bump,
    )]
    pub mining_pool_reward_state: Account<'info, MiningPoolRewardState>,

    #[account(
        seeds = [MINING_POOL_SEED, mining_pool_pda.pool_maker.key().as_ref(), mining_pool_pda.mint.key().as_ref()],
        bump = mining_pool_pda.bump,
      )]
    pub mining_pool_pda: Account<'info, MiningPoolPda>,
}

impl<'info> UndelegateRewardPool<'info> {
    fn undelegate_reward_pool(&mut self) -> Result<()> {
        // Only the mining pool's pool_maker can undelegate the reward pool
        require!(
            self.payer.key() == self.mining_pool_pda.pool_maker,
            MorErrorCodes::InvalidAuthority
        );

        // undelegate the reward pool account to the ER
        commit_and_undelegate_accounts(
            self.payer.to_account_info().as_ref(),
            vec![&self.mining_pool_reward_state.to_account_info()],
            self.magic_context.as_ref(),
            self.magic_program.as_ref(),
        )?;
        Ok(())
    }
}

pub fn undelegate_reward_pool_handler(ctx: Context<UndelegateRewardPool>) -> Result<()> {
    ctx.accounts.undelegate_reward_pool()?;

    Ok(())
}
