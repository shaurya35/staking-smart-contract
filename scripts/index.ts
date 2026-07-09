import { AnchorProvider, BN, Program, Wallet } from "@anchor-lang/core";
import { Connection, Keypair, PublicKey, clusterApiUrl } from "@solana/web3.js";
import { readFileSync } from "fs";
import { homedir } from "os";
import { join } from "path";
import idl from "../target/idl/staking_smart_contract.json";
import type { StakingSmartContract } from "../target/types/staking_smart_contract";

const keypairPath = join(homedir(), ".config/solana/id.json");
const secretKey = Uint8Array.from(
  JSON.parse(readFileSync(keypairPath, "utf-8"))
);
const walletKeypair = Keypair.fromSecretKey(secretKey);

const connection = new Connection(clusterApiUrl("devnet"), "confirmed");
const wallet = new Wallet(walletKeypair);
const provider = new AnchorProvider(connection, wallet, {
  commitment: "confirmed",
});

const program = new Program(idl as StakingSmartContract, provider);

const [pda] = PublicKey.findProgramAddressSync(
  [Buffer.from("client1"), walletKeypair.publicKey.toBuffer()],
  program.programId
);

console.log("Wallet:", walletKeypair.publicKey.toBase58());
console.log("Stake PDA:", pda.toBase58());

const before = await (program.account as any).stakeAccount.fetch(pda);
console.log("Before staked (SOL):", Number(before.stakedAmount) / 1_000_000_000);

const amount = 5_000_000;

const unstake = program.methods.unstake;
if (!unstake) {
  throw new Error("unstake method missing from IDL");
}

const sig = await unstake(new BN(amount))
  .accounts({
    user: walletKeypair.publicKey,
    pdaAccount: pda,
  })
  .rpc();

console.log("Unstaked lamports:", amount);
console.log("Tx:", sig);
console.log(
  "Explorer:",
  `https://explorer.solana.com/tx/${sig}?cluster=devnet`
);

const after = await (program.account as any).stakeAccount.fetch(pda);
console.log("After staked (SOL):", Number(after.stakedAmount) / 1_000_000_000);
