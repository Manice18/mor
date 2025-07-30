use crate::{
    helpers::constants::{MINER_SEED, MINING_POOL_SEED},
    states::{MinerAccountPoolPda, MiningPoolPda},
};
use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::delegate;
use ephemeral_rollups_sdk::cpi::DelegateConfig;

#[delegate]
#[derive(Accounts)]
#[instruction()]
pub struct DelegateMinerAccount<'info> {
    pub payer: Signer<'info>,
    /// CHECK The pda to delegate
    #[account(mut,
        del,
        seeds = [MINER_SEED, payer.key().as_ref(), mining_pool_pda.key().as_ref()],
        bump
    )]
    pub miner_pda: AccountInfo<'info>,

    #[account(
        seeds = [MINING_POOL_SEED, mining_pool_pda.pool_maker.key().as_ref(), mining_pool_pda.mint.key().as_ref()],
        bump = mining_pool_pda.bump,
      )]
    pub mining_pool_pda: Account<'info, MiningPoolPda>,
}

impl<'info> DelegateMinerAccount<'info> {
    fn delegate_miner_account(&mut self) -> Result<()> {
        let mining_pool_pda_key = self.mining_pool_pda.key();
        // delegate the session account to the ER
        let payer_key = self.payer.key();
        let miner_seeds = [MINER_SEED, payer_key.as_ref(), mining_pool_pda_key.as_ref()];
        self.delegate_miner_pda(&self.payer, &miner_seeds, DelegateConfig::default())?;
        Ok(())
    }
}

pub fn delegate_handler(ctx: Context<DelegateMinerAccount>) -> Result<()> {
    ctx.accounts.delegate_miner_account()?;

    Ok(())
}
