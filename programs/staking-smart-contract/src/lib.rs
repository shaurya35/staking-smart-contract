pub mod constants;
pub mod error;
pub mod instructions;
pub mod points;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use error::*;
pub use instructions::*;
pub use state::*;

declare_id!("HEdU3KyTDRNCsq5gL1r576WC86rrx1bZFQ9e83iK6xig");

#[program]
pub mod staking_smart_contract {
    use super::*;

    pub fn create_pda_account(ctx: Context<CreatePdaAccount>) -> Result<()> {
        create_pda_account_handler(ctx)
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        stake_handler(ctx, amount)
    }

    pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()> {
        unstake_handler(ctx, amount)
    }

    pub fn claim_points(ctx: Context<ClaimPoints>) -> Result<()> {
        claim_points_handler(ctx)
    }

    pub fn get_points(ctx: Context<GetPoints>) -> Result<()> {
        get_points_handler(ctx)
    }
}
