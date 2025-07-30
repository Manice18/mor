use crate::{
    helpers::constants::{MINER_SEED, MINING_POOL_SEED},
    states::{MinerAccountPoolPda, MiningPoolPda},
};
use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::commit;
use ephemeral_rollups_sdk::ephem::commit_and_undelegate_accounts;

#[commit]
#[derive(Accounts)]
#[instruction()]
pub struct UndelegateMinerAccount<'info> {
    pub payer: Signer<'info>,
    /// CHECK The pda to delegate
    #[account(
        mut,
        seeds = [MINER_SEED, payer.key().as_ref(), mining_pool_pda.key().as_ref()],
        bump = miner_pda.bump
    )]
    pub miner_pda: Account<'info, MinerAccountPoolPda>,

    #[account(
        seeds = [MINING_POOL_SEED, mining_pool_pda.pool_maker.key().as_ref(), mining_pool_pda.mint.key().as_ref()],
        bump = mining_pool_pda.bump,
    )]
    pub mining_pool_pda: Account<'info, MiningPoolPda>,
}

impl<'info> UndelegateMinerAccount<'info> {
    fn undelegate_miner_account(&mut self) -> Result<()> {
        // undelegate the session account to the ER
        commit_and_undelegate_accounts(
            self.payer.to_account_info().as_ref(),
            vec![&self.miner_pda.to_account_info()],
            self.magic_context.as_ref(),
            self.magic_program.as_ref(),
        )?;
        Ok(())
    }
}

pub fn undelegate_handler(ctx: Context<UndelegateMinerAccount>) -> Result<()> {
    ctx.accounts.undelegate_miner_account()?;

    Ok(())
}
