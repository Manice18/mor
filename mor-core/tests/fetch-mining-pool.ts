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

  const [miningPoolPda] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("mining_pool"),
      wallet.payer.publicKey.toBuffer(),
      TEST_TOKEN.toBuffer(),
    ],
    program.programId
  );

  it("Fetch Mining Pool Account", async () => {
    // Find the PDA for the mining pool account

    try {
      // Fetch the miner account data
      const miningPoolAccount = await program.account.miningPoolPda.fetch(
        miningPoolPda
      );

      console.log("Mining Pool Account Details:");
      console.log("Mining Pool PDA:", miningPoolPda.toString());
      console.log("Pool Maker:", miningPoolAccount.poolMaker.toString());
      console.log("Mint:", miningPoolAccount.mint.toString());
      console.log("Bump:", miningPoolAccount.bump.toString());
      console.log("Amount in pool:", miningPoolAccount.amount.toString());
    } catch (error) {
      console.log("Error fetching miner account:", error);
      console.log(
        "This might mean the miner account hasn't been initialized yet."
      );
    }
  });

  it("Fetch Mining Pool Reward State", async () => {
    // Find the PDA for the mining pool account
    const [miningPoolRewardStatePda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("mining_pool_reward"),
        wallet.payer.publicKey.toBuffer(),
        miningPoolPda.toBuffer(),
      ],
      program.programId
    );

    try {
      // Fetch the miner account data
      const miningPoolRewardStateAccount =
        await program.account.miningPoolRewardState.fetch(
          miningPoolRewardStatePda
        );

      console.log("Mining Pool Reward State Account Details:");
      console.log(
        "Mining Pool Reward State PDA:",
        miningPoolRewardStatePda.toString()
      );
      console.log("Pool PDA:", miningPoolRewardStateAccount.poolPda.toString());
      console.log("Amount:", miningPoolRewardStateAccount.amount.toString());
      console.log("Bump:", miningPoolRewardStateAccount.bump.toString());
    } catch (error) {
      console.log("Error fetching miner account:", error);
      console.log(
        "This might mean the miner account hasn't been initialized yet."
      );
    }
  });
});
