use anchor_lang::prelude::*;

use crate::{
    constants::STAKE_SEED,
    error::StakeError,
    points::update_points,
    state::StakeAccount,
};

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [STAKE_SEED, user.key().as_ref()],
        bump = pda_account.bump,
        constraint = pda_account.owner == user.key() @ StakeError::Unauthorized
    )]
    pub pda_account: Account<'info, StakeAccount>,
}

pub fn unstake_handler(ctx: Context<Unstake>, amount: u64) -> Result<()> {
    require!(amount > 0, StakeError::InvalidAmount);

    let clock = Clock::get()?;

    require!(
        ctx.accounts.pda_account.staked_amount >= amount,
        StakeError::InsufficientStake
    );

    update_points(&mut ctx.accounts.pda_account, clock.unix_timestamp)?;

    // Program-owned accounts cannot use system_program::transfer as the source.
    // Move lamports directly while keeping the account rent-exempt.
    let pda_info = ctx.accounts.pda_account.to_account_info();
    let user_info = ctx.accounts.user.to_account_info();
    let rent_minimum = Rent::get()?.minimum_balance(pda_info.data_len());
    let remaining_lamports = pda_info
        .lamports()
        .checked_sub(amount)
        .ok_or(StakeError::Underflow)?;
    require!(
        remaining_lamports >= rent_minimum,
        StakeError::RentExemptViolation
    );

    let user_lamports = user_info
        .lamports()
        .checked_add(amount)
        .ok_or(StakeError::Overflow)?;

    **pda_info.try_borrow_mut_lamports()? = remaining_lamports;
    **user_info.try_borrow_mut_lamports()? = user_lamports;

    ctx.accounts.pda_account.staked_amount = ctx
        .accounts
        .pda_account
        .staked_amount
        .checked_sub(amount)
        .ok_or(StakeError::Underflow)?;

    msg!(
        "Unstaked {} lamports, Remaining staked: {}, Total points: {}",
        amount,
        ctx.accounts.pda_account.staked_amount,
        ctx.accounts.pda_account.total_points / 1_000_000
    );

    Ok(())
}
