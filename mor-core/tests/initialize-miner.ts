import * as anchor from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";

import { MorCore } from "../target/types/mor_core";

describe("Initialize Miner", () => {
  const wallet = anchor.Wallet.local();
  let rpcHttpUrl = "http://localhost:8899";
  let rpcWsUrl = "ws://127.0.0.1:8900";

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

  const TEST_TOKEN = new PublicKey(
    "tesnw8eLyAwSR5oGzGogWcAuJhp4pynBzjKvs6kvw9T"
  );

  it("Initialize Miner Account", async () => {
    // Find the PDA for the miner account
    miningPoolPda = PublicKey.findProgramAddressSync(
      [
        Buffer.from("mining_pool"),
        wallet.payer.publicKey.toBuffer(),
        TEST_TOKEN.toBuffer(),
      ],
      program.programId
    )[0];

    minerPda = PublicKey.findProgramAddressSync(
      [
        Buffer.from("miner"),
        wallet.payer.publicKey.toBuffer(),
        miningPoolPda.toBuffer(),
      ],
      program.programId
    )[0];

    // Initialize the miner account
    const tx = await program.methods
      .initializeMiner()
      .accountsStrict({
        miner: minerPda,
        miningPoolPda: miningPoolPda,
        authority: wallet.payer.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([wallet.payer])
      .rpc();

    console.log("Transaction Signature:", tx);
  });
});
