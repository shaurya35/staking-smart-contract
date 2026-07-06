# staking-contract

A small Anchor program that lets a user stake SOL on chain and earn points
over time.

This is a simple example to learn how a Solana program holds SOL in a PDA and
tracks rewards based on how long you stake.

## What it does

Each user gets their own staking account (a PDA) and can:

- `create_pda_account` - set up the user's staking account
- `stake(amount)` - move SOL into the PDA and start earning points
- `unstake(amount)` - move SOL back from the PDA to the user
- `claim_points` - read the points earned so far and reset them
- `get_points` - read the current points without changing anything

Points build up at a rate of 1 point per SOL per day.

## How it works

1. The staking account stores the owner, the amount staked, the points earned,
   and the last time points were updated.
2. Before any stake or unstake, the program settles points up to the current
   time so nothing is lost.
3. `stake` and `unstake` move SOL with a CPI to the System Program. For
   unstake, the PDA signs the transfer itself using its seeds.

## State

```rust
#[account]
pub struct StakeAccount {
    pub owner: Pubkey,
    pub staked_amount: u64,
    pub total_points: u64,
    pub last_update_time: i64,
    pub bump: u8,
}
```

## Project layout

```
staking-contract/
  src/lib.rs     the program code
  Cargo.toml     project settings and dependencies
```

## Build

```bash
anchor build
```

## Deploy

```bash
anchor deploy
```

Run this against your chosen cluster, for example a local validator or devnet.
