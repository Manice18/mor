import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import { MorCore } from "../target/types/mor_core";

describe("Delegate Reward Pool", () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const TEST_TOKEN = new PublicKey(
    "tesnw8eLyAwSR5oGzGogWcAuJhp4pynBzjKvs6kvw9T"
  );

  const providerEphemeralRollup = new anchor.AnchorProvider(
    new anchor.web3.Connection(
      process.env.PROVIDER_ENDPOINT || "http://0.0.0.0:7799/",
      {
        wsEndpoint: process.env.WS_ENDPOINT || "ws://0.0.0.0:7800/",
      }
    ),
    anchor.Wallet.local()
  );

  //@ts-ignore
  console.log("Base Layer Connection: ", provider.connection._rpcEndpoint);
  console.log(
    "Ephemeral Rollup Connection: ",
    //@ts-ignore
    providerEphemeralRollup.connection._rpcEndpoint
  );
  console.log(`Current SOL Public Key: ${anchor.Wallet.local().publicKey}`);

  const program = anchor.workspace.MorCore as anchor.Program<MorCore>;

  let miningPoolPda: PublicKey;
  let miningPoolRewardStatePda: PublicKey;

  it("Delegate reward pool to ER", async () => {
    miningPoolPda = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("mining_pool"),
        provider.wallet.publicKey.toBuffer(),
        TEST_TOKEN.toBuffer(),
      ],
      program.programId
    )[0];

    miningPoolRewardStatePda = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("mining_pool_reward"),
        provider.wallet.publicKey.toBuffer(),
        miningPoolPda.toBuffer(),
      ],
      program.programId
    )[0];

    console.log("Program ID: ", program.programId.toString());
    console.log("Mining Pool PDA: ", miningPoolPda.toString());
    console.log(
      "Mining Pool Reward State PDA: ",
      miningPoolRewardStatePda.toString()
    );

    const start = Date.now();
    let tx = await program.methods
      .delegateRewardPool()
      .accountsPartial({
        payer: provider.wallet.publicKey,
        miningPoolRewardState: miningPoolRewardStatePda,
        miningPoolPda: miningPoolPda,
      })
      .transaction();

    tx.feePayer = provider.wallet.publicKey;
    tx.recentBlockhash = (
      await provider.connection.getLatestBlockhash()
    ).blockhash;

    tx = await providerEphemeralRollup.wallet.signTransaction(tx);
    const txHash = await provider.sendAndConfirm(tx, [], {
      skipPreflight: true,
      commitment: "confirmed",
    });

    const duration = Date.now() - start;
    console.log(
      `${duration}ms (Base Layer) Delegate Reward Pool txHash: ${txHash}`
    );
  });
});
