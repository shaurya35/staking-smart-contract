use anchor_lang::prelude::*;

#[constant]
pub const STAKE_SEED: &[u8] = b"client1";

pub const POINTS_PER_SOL_PER_DAY: u64 = 1_000_000;
pub const LAMPORTS_PER_SOL: u64 = 1_000_000_000;
pub const SECONDS_PER_DAY: u64 = 86_400;
