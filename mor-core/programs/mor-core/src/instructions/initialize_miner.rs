use anchor_lang::prelude::*;

use crate::{
    helpers::constants::{MINER_SEED, MINING_POOL_SEED},
    states::{MinerAccountPoolPda, MiningPoolPda},
};

#[derive(Accounts)]
#[instruction()]
pub struct InitializeMiner<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space =  MinerAccountPoolPda::INIT_SPACE + MinerAccountPoolPda::DISCRIMINATOR.len(),
        seeds = [MINER_SEED, authority.key().as_ref(), mining_pool_pda.key().as_ref()],
        bump
    )]
    pub miner: Account<'info, MinerAccountPoolPda>,

    #[account(
        mut,
        seeds = [MINING_POOL_SEED, mining_pool_pda.pool_maker.key().as_ref(), mining_pool_pda.mint.key().as_ref()],
        bump = mining_pool_pda.bump,
      )]
    pub mining_pool_pda: Account<'info, MiningPoolPda>,

    pub system_program: Program<'info, System>,
}

impl<'info> InitializeMiner<'info> {
    fn initialize_miner(&mut self, bumps: &InitializeMinerBumps) -> Result<()> {
        let miner = &mut self.miner;

        // Initialize miner account with default values
        miner.authority = self.authority.key();
        miner.last_epoch_mined = u64::MAX; // means "never mined"
        miner.rewards = 0;
        miner.multiplier = 1; // Start with base multiplier
        miner.staked_amount = 0;
        miner.last_staked_timestamp = 0;
        miner.difficulty = 0;
        miner.pool = self.mining_pool_pda.key();
        miner.bump = bumps.miner;
        Ok(())
    }
}

pub fn initialize_miner_handler(ctx: Context<InitializeMiner>) -> Result<()> {
    ctx.accounts.initialize_miner(&ctx.bumps)?;

    Ok(())
}
