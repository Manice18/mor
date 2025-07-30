use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::{
    helpers::{
        constants::{MINER_SEED, MINING_POOL_SEED},
        errors::MorErrorCodes,
    },
    states::{MinerAccountPoolPda, MiningPoolPda},
};

#[derive(Accounts)]
#[instruction()]
pub struct ClaimRewards<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
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
        mut,
        associated_token::mint = mint,
        associated_token::authority = mining_pool_pda,
        associated_token::token_program = token_program
      )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = mint,
        associated_token::authority = authority,
        associated_token::token_program = token_program
    )]
    pub recipient_ata: InterfaceAccount<'info, TokenAccount>,

    /// Programs
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> ClaimRewards<'info> {
    fn claim_rewards(&mut self) -> Result<()> {
        let miner = &mut self.miner;

        // Check if miner has rewards to claim
        require!(miner.rewards > 0, MorErrorCodes::NoRewardsToClaim);

        let pool_maker_key = self.mining_pool_pda.pool_maker.key();
        let mint_key = self.mining_pool_pda.mint.key();
        // Create the signer seeds for the Vault
        let signer_seeds: [&[&[u8]]; 1] = [&[
            MINING_POOL_SEED,
            pool_maker_key.as_ref(),
            mint_key.as_ref(),
            &[self.mining_pool_pda.bump],
        ]];

        // Transfer Token A (Vault -> Taker)
        transfer_checked(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.vault.to_account_info(),
                    to: self.recipient_ata.to_account_info(),
                    mint: self.mint.to_account_info(),
                    authority: self.mining_pool_pda.to_account_info(),
                },
                &signer_seeds,
            ),
            miner.rewards,
            self.mint.decimals,
        )?;
        miner.rewards = 0;

        Ok(())
    }
}

pub fn claim_rewards_handler(ctx: Context<ClaimRewards>) -> Result<()> {
    ctx.accounts.claim_rewards()?;

    Ok(())
}
