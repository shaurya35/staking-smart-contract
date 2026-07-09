use anchor_lang::prelude::*;

use crate::{
    constants::{LAMPORTS_PER_SOL, STAKE_SEED},
    error::StakeError,
    points::calculate_points_earned,
    state::StakeAccount,
};

#[derive(Accounts)]
pub struct GetPoints<'info> {
    pub user: Signer<'info>,

    #[account(
        seeds = [STAKE_SEED, user.key().as_ref()],
        bump = pda_account.bump,
        constraint = pda_account.owner == user.key() @ StakeError::Unauthorized
    )]
    pub pda_account: Account<'info, StakeAccount>,
}

pub fn get_points_handler(ctx: Context<GetPoints>) -> Result<()> {
    let pda_account = &ctx.accounts.pda_account;
    let clock = Clock::get()?;

    let time_elapsed = clock
        .unix_timestamp
        .checked_sub(pda_account.last_update_time)
        .ok_or(StakeError::InvalidTimestamp)?;

    let new_points = if time_elapsed > 0 && pda_account.staked_amount > 0 {
        calculate_points_earned(pda_account.staked_amount, time_elapsed as u64)?
    } else {
        0
    };

    let current_total_points = pda_account
        .total_points
        .checked_add(new_points)
        .ok_or(StakeError::Overflow)?;

    msg!(
        "Current Points: {}, Staked amount: {} SOL",
        current_total_points / 1_000_000,
        pda_account.staked_amount / LAMPORTS_PER_SOL
    );

    Ok(())
}
