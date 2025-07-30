import * as anchor from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  getOrCreateAssociatedTokenAccount,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { getKeypairFromFile } from "@solana-developers/helpers";

import { MorCore } from "../target/types/mor_core";

describe("Claim Rewards", () => {
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

  const TEST_TOKEN = new PublicKey(
    "tesnw8eLyAwSR5oGzGogWcAuJhp4pynBzjKvs6kvw9T"
  );

  let miningPoolPda: PublicKey;
  let minerPda: PublicKey;

  it("Claim-Rewards", async () => {
    miningPoolPda = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("mining_pool"),
        provider.wallet.publicKey.toBuffer(),
        TEST_TOKEN.toBuffer(),
      ],
      program.programId
    )[0];

    minerPda = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("miner"),
        provider.wallet.publicKey.toBuffer(),
        miningPoolPda.toBuffer(),
      ],
      program.programId
    )[0];

    const vault = getAssociatedTokenAddressSync(
      TEST_TOKEN,
      miningPoolPda,
      true,
      TOKEN_PROGRAM_ID
    );

    const recipientTokenAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      wallet.payer,
      TEST_TOKEN,
      wallet.payer.publicKey
    );

    const tx = await program.methods
      .claimRewards()
      .accountsStrict({
        miner: minerPda,
        mint: TEST_TOKEN,
        miningPoolPda: miningPoolPda,
        vault: vault,
        recipientAta: recipientTokenAccount.address,
        authority: wallet.payer.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([wallet.payer])
      .rpc();

    console.log("Transaction Signature:", tx);
  });
});
