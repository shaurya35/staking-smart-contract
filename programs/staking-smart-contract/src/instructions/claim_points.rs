use anchor_lang::prelude::*;

use crate::{
    constants::STAKE_SEED,
    error::StakeError,
    points::update_points,
    state::StakeAccount,
};

#[derive(Accounts)]
pub struct ClaimPoints<'info> {
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [STAKE_SEED, user.key().as_ref()],
        bump = pda_account.bump,
        constraint = pda_account.owner == user.key() @ StakeError::Unauthorized
    )]
    pub pda_account: Account<'info, StakeAccount>,
}

pub fn claim_points_handler(ctx: Context<ClaimPoints>) -> Result<()> {
    let pda_account = &mut ctx.accounts.pda_account;
    let clock = Clock::get()?;

    update_points(pda_account, clock.unix_timestamp)?;

    let claimable_points = pda_account.total_points / 1_000_000;
    msg!("User has {} claimable points", claimable_points);

    // Points are tracked on-chain only; claiming resets the accrued balance.
    pda_account.total_points = 0;

    Ok(())
}
