use anchor_lang::prelude::*;

#[error_code]
pub enum MorErrorCodes {
    #[msg("You already mined this epoch.")]
    AlreadyMined,
    #[msg("Invalid solution.")]
    InvalidSolution,
    #[msg("Token cap reached.")]
    CapReached,
    #[msg("No rewards available to claim")]
    NoRewardsToClaim,
    #[msg("No rewards available in mining pool")]
    NoRewardsAvailable,
    #[msg("Invalid authority.")]
    InvalidAuthority,
    #[msg("Invalid epoch.")]
    InvalidEpoch,
    #[msg("Invalid epoch range.")]
    InvalidEpochRange,
    #[msg("Insufficient staked amount")]
    InsufficientStakedAmount,
    #[msg("Invalid amount.")]
    InvalidAmount,
}
