import * as anchor from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";

import { MorCore } from "../target/types/mor_core";

describe("Fetch Miner", () => {
  const wallet = anchor.Wallet.local();
  let rpcHttpUrl = "http://localhost:8899";
  let rpcWsUrl = "ws://127.0.0.1:8900";

  const TEST_TOKEN = new PublicKey(
    "tesnw8eLyAwSR5oGzGogWcAuJhp4pynBzjKvs6kvw9T"
  );

  const connection = new anchor.web3.Connection(rpcHttpUrl, {
    wsEndpoint: rpcWsUrl,
  });
  const provider = new anchor.AnchorProvider(
    connection,
    wallet,
    anchor.AnchorProvider.defaultOptions()
  );

  anchor.setProvider(provider);

  const program = anchor.workspace.MorCore as anchor.Program<MorCore>;

  let miningPoolPda: PublicKey;
  let minerPda: PublicKey;

  it("Fetch Miner Account", async () => {
    miningPoolPda = PublicKey.findProgramAddressSync(
      [
        Buffer.from("mining_pool"),
        wallet.payer.publicKey.toBuffer(),
        TEST_TOKEN.toBuffer(),
      ],
      program.programId
    )[0];

    // Find the PDA for the miner account
    minerPda = PublicKey.findProgramAddressSync(
      [
        Buffer.from("miner"),
        wallet.payer.publicKey.toBuffer(),
        miningPoolPda.toBuffer(),
      ],
      program.programId
    )[0];

    try {
      // Fetch the miner account data
      const minerAccount = await program.account.minerAccountPoolPda.fetch(
        minerPda
      );

      console.log("Miner Account Details:");
      console.log("Difficulty:", minerAccount.difficulty);
      console.log("Authority:", minerAccount.authority.toString());
      console.log("Last Epoch Mined:", minerAccount.lastEpochMined.toString());
      console.log("Rewards:", minerAccount.rewards.toString());
      console.log("Multiplier:", minerAccount.multiplier);
      console.log("Staked Amount:", minerAccount.stakedAmount.toString());
      console.log(
        "Last Staked Timestamp:",
        new Date(
          minerAccount.lastStakedTimestamp.toNumber() * 1000
        ).toISOString()
      );
      console.log("Pool:", minerAccount.pool.toString());
    } catch (error) {
      console.log("Error fetching miner account:", error);
      console.log(
        "This might mean the miner account hasn't been initialized yet."
      );
    }
  });
});
