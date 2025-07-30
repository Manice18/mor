import * as anchor from "@coral-xyz/anchor";
import { Keypair, PublicKey } from "@solana/web3.js";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  getMint,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { getKeypairFromFile } from "@solana-developers/helpers";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";

import { MorCore } from "../target/types/mor_core";
import { BN } from "bn.js";

describe("Initialize Mining Pool", () => {
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

  let miningPoolPda: PublicKey;
  let poolMakerAta: PublicKey;
  let miningPoolRewardState: PublicKey;

  const TEST_TOKEN = new PublicKey(
    "tesnw8eLyAwSR5oGzGogWcAuJhp4pynBzjKvs6kvw9T"
  );

  const program = anchor.workspace.MorCore as anchor.Program<MorCore>;

  const payer = provider.wallet as NodeWallet;

  it("Initialize Pool & Deposit Tokens", async () => {
    console.log("Test Mint: ", TEST_TOKEN.toBase58());
    console.log("Payer Public Key: ", payer.publicKey.toBase58());

    miningPoolPda = PublicKey.findProgramAddressSync(
      [
        Buffer.from("mining_pool"),
        payer.publicKey.toBuffer(),
        TEST_TOKEN.toBuffer(),
      ],
      program.programId
    )[0];

    miningPoolRewardState = PublicKey.findProgramAddressSync(
      [
        Buffer.from("mining_pool_reward"),
        payer.publicKey.toBuffer(),
        miningPoolPda.toBuffer(),
      ],
      program.programId
    )[0];

    console.log("Calculated PDA: ", miningPoolPda.toBase58());
    console.log(
      "Calculated Reward State PDA: ",
      miningPoolRewardState.toBase58()
    );
    console.log("Program ID: ", program.programId.toBase58());

    poolMakerAta = getAssociatedTokenAddressSync(
      TEST_TOKEN,
      payer.publicKey,
      false,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );

    const vault = getAssociatedTokenAddressSync(
      TEST_TOKEN,
      miningPoolPda,
      true,
      TOKEN_PROGRAM_ID
    );

    const initialMintInfo = await getMint(connection, TEST_TOKEN);

    const tx = await program.methods
      .initializePool(new BN(500 * 10 ** initialMintInfo.decimals)) // 500 tokens
      .accountsStrict({
        poolMaker: payer.publicKey,
        miningPoolPda: miningPoolPda,
        miningPoolRewardState: miningPoolRewardState,
        mint: TEST_TOKEN,
        poolMakerAta: poolMakerAta,
        vault: vault,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SYSTEM_PROGRAM_ID,
      })
      .signers([payer.payer])
      .rpc();
    console.log("Transaction Signature:", tx);
  });
});
