pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;
use anchor_lang::system_program;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("HEdU3KyTDRNCsq5gL1r576WC86rrx1bZFQ9e83iK6xig");

const POINTS_PER_SOL_PER_DAY: u64 = 1_000_000;
const LAMPORTS_PER_SOL: u64 = 1_000_000_000;
const SECONDS_PER_DAY: u64 = 86_400;

#[program]
pub mod staking_smart_contract {
    use anchor_lang::accounts::signer;

use super::*;
  
    pub fn create_pda_account(ctx: Context<CreatePdaAccount>) -> Result<()> {
        let pda_account = &mut ctx.accounts.pda_account;
        let clock = Clock::get()?;

        pda_account.owner = ctx.accounts.payer.key();
        pda_account.staked_amount = 0;
        pda_account.total_points = 0;
        pda_account.last_update_time = clock.unix_timestamp;

        msg!("PDA Account Created Successfully!");
        Ok(())
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        require!(amount > 0, StakeError::InvalidAmount);

        let pda_account = &mut ctx.accounts.pda_account;
        let clock = Clock::get();

        update_points(pda_account, clock.unix_timestamp)?;

        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.system_program.to_account_info(),
                to: ctx.accounts.pda_account.to_account_info(),
            },
        );
        
        system_program::transfer(cpi_context, amount)?;

        pda_account.staked_amount = pda_account.staked_amount.checked_add(amount)
            .ok_or(StakeError::Overflow)?;

        msg!("Staked {} lamports. Total Staked: {}, Total points: {}",
                amount, pda_account.staked_amount, pda_account.total_points / 1_000_000);

        Ok(())
    }

    pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Results<()> {
        require!(amount > 0, StakeError::InvalidAmount);

        let pda_account = &mut ctx.accounts.pda_account;
        let clock = Clock::get();

        require!(pda_account.staked_amount >= amount, StakeError::InsufficientStake);

        update_points(pda_account, clock.unix_timestamp)?;

        let seed = &[
            b"client1",
            ctx.accounts.user.key().as_ref(),
            &[pda_account.bump],
        ];
        
        let signer = &[&seeds[..]];

        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(), 
            system_program::Transfer {
                from: ctx.accounts.pda_account.to_account_info(),
                to: ctx.accounts.user.to_account_info(),
            }, 
            signer,
        );

        system_program::transfer(cpi_contract, amount)?;

        pda_account.staked_amount = pda_account.staked_amount.check_sub(amount)
            .ok_or(StakeError::Underflow)?;

        msg!("Unstaked {} lamports, Remaining staked: {}, Total points: {}", 
            amount, pda_account.staked_amount, pda_account.total_points / 1_000_000);

        Ok(())
    }

    pub fn claim_points(ctx: Context<ClaimPoints>) -> Results<()> {
        let pda_account = &mut ctx.accounts.pda_account;
        let clock = Clock::get();

        update_points(pda_account, clock.unix_timestamp)?;

        let claimable_points = pda_account.total_points / 1_000_000;

        msg!("User has {} claimable points", claimable_points);

        pda_account.total_points = 0;

        Ok(())
    }

    pub fn get_points(ctx: Context<GetPoints>) -> Results<()> {
        let pda_account = ctx.accounts.pda_account;
        let clock = Clock::get();

        let time_elapsed = clock.unix_timestamp.checked_sub(pda_account.last_update_time)
            .ok_or(StakeError::InvalidTimestamp)?;

        let new_points = calculate_points_earned(pda_account.staked_amount, time_elapsed)?;

        let current_total_points = pda_account.total_points.checked_add(new_points)
            .ok_or(StakeError::Overflow)?;

        msg!("Current Points: {}, Staked amount: {} SOL",
                current_total_points / 1_000_000,
                pda_account.staked_amount / LAMPORTS_PER_SOL
            );

        Ok(())
    } 

}

fn update_points(pda_account: &mut StakeAccount, current_time: i64) -> Result<()> {
    let time_elapsed = current_time.checked_sub(pda_account.last_update_time)
        .ok_or(StakeError::InvalidTimestamp);

    if time_elapsed > 0 && pda_account.staked_amount > 0 {
        let new_points = calculate_points_earned(pda_account.staked_amount, time_elapsed)?;
        pda_account.total_points = pda_account.total_points.checked_add(new_points)
            .ok_or(StakeError::Overflow)?;
    }

    pda_account.last_update_time = current_time;
    Ok(())
}

fn calculate_points_earned(staked_amount: u64, time_elapsed_seconds: u64) -> Result<u64> {
    let points = (staked_amount as u128)
        .checked_mul(time_elapsed_seconds as u128)
        .ok_or(StakeError::Overflow)?
        .checked_mul(POINTS_PER_SOL_PER_DAY as u128)
        .ok_or(StakeError::Overflow)?
        .checked_div(LAMPORTS_PER_SOL as u128)
        .ok_or(StakeError::Overflow)?
        .checked_div(SECONDS_PER_DAY as u128)
        .ok_or(StakeError::Overflow)?;

    Ok(points as u64)
}

#[derive(Accounts)]
pub struct CreatePdaAccount<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(
        init,
        payer = payer,
        space = 8 + 32 + 8 + 8 + 8 + 1, 
        seeds = [b"client1", payer.key().as_ref()],
        bump
    )]
    pub pda_account: Account<'info, StakeAccount>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"client1", user.key().as_ref()],
        bump = pda_account.bump,
        constraint = pda_account.owner == user.key() @ StakeError::Unauthorized
    )]
    pub pda_account: Account<'info, StakeAccount>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"client1", user.key().as_ref()],
        bump = pda_account.bump,
        constraint = pda_account.owner == user.key() @ StakeError::Unauthorized
    )]
    pub pda_account: Account<'info, StakeAccount>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimPoints<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"client1", user.key().as_ref()],
        bump = pda_account.bump,
        constraint = pda_account.owner == user.key() @ StakeError::Unauthorized
    )]
    pub pda_account: Account<'info, StakeAccount>,
}

#[derive(Accounts)]
pub struct GetPoints<'info> {
    pub user: Signer<'info>,
    
    #[account(
        seeds = [b"client1", user.key().as_ref()],
        bump = pda_account.bump,
        constraint = pda_account.owner == user.key() @ StakeError::Unauthorized
    )]
    pub pda_account: Account<'info, StakeAccount>,
}

#[account]
pub struct StakeAccount {
    pub owner: Pubkey,           
    pub staked_amount: u64,      
    pub total_points: u64,       
    pub last_update_time: i64,   
    pub bump: u8,                
}

#[error_code]
pub enum StakeError {
    #[msg("Amount must be greater than 0")]
    InvalidAmount,
    #[msg("Insufficient staked amount")]
    InsufficientStake,
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Arithmetic overflow")]
    Overflow,
    #[msg("Arithmetic underflow")]
    Underflow,
    #[msg("Invalid timestamp")]
    InvalidTimestamp,
}