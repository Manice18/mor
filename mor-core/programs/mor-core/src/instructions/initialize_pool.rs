use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::{
    helpers::constants::{MINING_POOL_REWARD_SEED, MINING_POOL_SEED},
    helpers::errors::MorErrorCodes,
    states::{MiningPoolPda, MiningPoolRewardState},
};

#[derive(Accounts)]
#[instruction()]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub pool_maker: Signer<'info>,

    #[account(
    init,
    payer = pool_maker,
    space = MiningPoolPda::INIT_SPACE + MiningPoolPda::DISCRIMINATOR.len(),
    seeds = [MINING_POOL_SEED, pool_maker.key().as_ref(), mint.key().as_ref()],
    bump,
    )]
    pub mining_pool_pda: Account<'info, MiningPoolPda>,

    #[account(
        init,
        payer = pool_maker,
        space = MiningPoolRewardState::INIT_SPACE + MiningPoolRewardState::DISCRIMINATOR.len(),
        seeds = [MINING_POOL_REWARD_SEED, pool_maker.key().as_ref(), mining_pool_pda.key().as_ref()],
        bump,
    )]
    pub mining_pool_reward_state: Account<'info, MiningPoolRewardState>,

    /// Token Accounts
    #[account(
        mint::token_program = token_program
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = pool_maker,
        associated_token::token_program = token_program
    )]
    pub pool_maker_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init,
        payer = pool_maker,
        associated_token::mint = mint,
        associated_token::authority = mining_pool_pda,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    /// Programs
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializePool<'info> {
    /// # Create the Mining Pool
    fn create_mining_pool(&mut self, amount: u64, bump: u8) -> Result<()> {
        self.mining_pool_pda.set_inner(MiningPoolPda {
            pool_maker: self.pool_maker.key(),
            mint: self.mint.key(),
            amount,
            bump,
        });

        self.mining_pool_reward_state
            .set_inner(MiningPoolRewardState {
                pool_pda: self.mining_pool_pda.key(),
                amount,
                bump,
            });

        Ok(())
    }

    /// # Deposit the tokens
    fn deposit_tokens(&mut self, amount: u64) -> Result<()> {
        transfer_checked(
            CpiContext::new(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.pool_maker_ata.to_account_info(),
                    mint: self.mint.to_account_info(),
                    to: self.vault.to_account_info(),
                    authority: self.pool_maker.to_account_info(),
                },
            ),
            amount,
            self.mint.decimals,
        )?;

        Ok(())
    }
}

pub fn initialize_pool_handler(ctx: Context<InitializePool>, amount: u64) -> Result<()> {
    // Validate the amount
    require_gte!(amount, 0, MorErrorCodes::InvalidAmount);

    // Save the Mining Pool Data
    ctx.accounts
        .create_mining_pool(amount, ctx.bumps.mining_pool_pda)?;

    // Deposit Tokens
    ctx.accounts.deposit_tokens(amount)?;

    Ok(())
}
