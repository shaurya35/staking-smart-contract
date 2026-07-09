use anchor_lang::{prelude::*, system_program};

use crate::{
    constants::STAKE_SEED,
    error::StakeError,
    points::update_points,
    state::StakeAccount,
};

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [STAKE_SEED, user.key().as_ref()],
        bump = pda_account.bump,
        constraint = pda_account.owner == user.key() @ StakeError::Unauthorized
    )]
    pub pda_account: Account<'info, StakeAccount>,

    pub system_program: Program<'info, System>,
}

pub fn stake_handler(ctx: Context<Stake>, amount: u64) -> Result<()> {
    require!(amount > 0, StakeError::InvalidAmount);

    let clock = Clock::get()?;
    update_points(&mut ctx.accounts.pda_account, clock.unix_timestamp)?;

    let cpi_context = CpiContext::new(
        system_program::ID,
        system_program::Transfer {
            from: ctx.accounts.user.to_account_info(),
            to: ctx.accounts.pda_account.to_account_info(),
        },
    );
    system_program::transfer(cpi_context, amount)?;

    let pda_account = &mut ctx.accounts.pda_account;
    pda_account.staked_amount = pda_account
        .staked_amount
        .checked_add(amount)
        .ok_or(StakeError::Overflow)?;

    msg!(
        "Staked {} lamports. Total Staked: {}, Total points: {}",
        amount,
        pda_account.staked_amount,
        pda_account.total_points / 1_000_000
    );

    Ok(())
}
