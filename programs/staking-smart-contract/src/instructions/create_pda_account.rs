use anchor_lang::prelude::*;

use crate::{constants::STAKE_SEED, state::StakeAccount};

#[derive(Accounts)]
pub struct CreatePdaAccount<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + StakeAccount::INIT_SPACE,
        seeds = [STAKE_SEED, payer.key().as_ref()],
        bump
    )]
    pub pda_account: Account<'info, StakeAccount>,

    pub system_program: Program<'info, System>,
}

pub fn create_pda_account_handler(ctx: Context<CreatePdaAccount>) -> Result<()> {
    let pda_account = &mut ctx.accounts.pda_account;
    let clock = Clock::get()?;

    pda_account.owner = ctx.accounts.payer.key();
    pda_account.staked_amount = 0;
    pda_account.total_points = 0;
    pda_account.last_update_time = clock.unix_timestamp;
    pda_account.bump = ctx.bumps.pda_account;

    msg!("PDA Account Created Successfully!");
    Ok(())
}
