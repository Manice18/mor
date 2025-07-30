import * as anchor from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { sha3_256 } from "@noble/hashes/sha3";

import { MorCore } from "../target/types/mor_core";

describe("SubmitSolution", () => {
  const wallet = anchor.Wallet.local();
  const connection = new anchor.web3.Connection(
    "http://localhost:7799",
    "confirmed"
  );
  const provider = new anchor.AnchorProvider(connection, wallet, {});
  anchor.setProvider(provider);

  const program = anchor.workspace.MorCore as anchor.Program<MorCore>;

  const EPOCH_SLOT_LENGTH = 150;

  const TEST_TOKEN = new PublicKey(
    "tesnw8eLyAwSR5oGzGogWcAuJhp4pynBzjKvs6kvw9T"
  );

  const MINER_SEED = "miner";
  const MINING_POOL_SEED = "mining_pool";
  const MINING_POOL_REWARD_SEED = "mining_pool_reward";

  let minerPda: PublicKey;
  let miningPoolPda: PublicKey;
  let miningPoolRewardState: PublicKey;

  const payer = wallet.payer;

  // Epoch and Challenge helpers
  const getEpoch = async (): Promise<number> => {
    const slot = await connection.getSlot("confirmed");
    return Math.floor(slot / EPOCH_SLOT_LENGTH);
  };

  const generateChallenge = (authority: PublicKey, epoch: number): Buffer => {
    const epochBuf = Buffer.alloc(8);
    epochBuf.writeBigUInt64LE(BigInt(epoch));
    const hasher = sha3_256.create();
    hasher.update(authority.toBuffer());
    hasher.update(epochBuf);
    return Buffer.from(hasher.digest()); // 32 bytes
  };

  const findValidNonce = (challenge: Buffer, difficulty: number): number => {
    for (let nonce = 0; nonce < Number.MAX_SAFE_INTEGER; nonce++) {
      const nonceBuf = Buffer.alloc(8);
      nonceBuf.writeBigUInt64LE(BigInt(nonce));
      const hash = sha3_256
        .create()
        .update(challenge)
        .update(nonceBuf)
        .digest();
      // console.log("Nonce: ", nonce);
      // console.log("Hash: ", Array.from(hash.slice(0, 10)));
      if (hash.slice(0, difficulty).every((b) => b === 0)) {
        console.log("Found nonce: ", nonce);
        console.log("Hash: ", Array.from(hash));
        return nonce;
      }
    }
    throw new Error("No valid nonce found (increase difficulty?)");
  };

  const submitSolution = async (isSimulate: boolean, attempts: number) => {
    // Create base layer connection to get state account (since not cloned at beginning)
    const baseLayerConnection = new anchor.web3.Connection(
      "http://localhost:8899",
      "confirmed"
    );
    const baseLayerProvider = new anchor.AnchorProvider(
      baseLayerConnection,
      wallet,
      {}
    );
    const baseLayerProgram = new anchor.Program<MorCore>(
      program.idl,
      baseLayerProvider
    );

    miningPoolPda = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from(MINING_POOL_SEED),
        provider.wallet.publicKey.toBuffer(),
        TEST_TOKEN.toBuffer(),
      ],
      program.programId
    )[0];

    miningPoolRewardState = PublicKey.findProgramAddressSync(
      [
        Buffer.from(MINING_POOL_REWARD_SEED),
        payer.publicKey.toBuffer(),
        miningPoolPda.toBuffer(),
      ],
      program.programId
    )[0];

    minerPda = PublicKey.findProgramAddressSync(
      [
        Buffer.from(MINER_SEED),
        payer.publicKey.toBuffer(),
        miningPoolPda.toBuffer(),
      ],
      program.programId
    )[0];

    console.log("Miner PDA: ", minerPda);

    let minerAccount;
    try {
      minerAccount = await program.account.minerAccountPoolPda.fetch(minerPda);
      if (!minerAccount)
        throw new Error("Miner account is null/undefined from ER");
    } catch (err) {
      console.log("Failed to fetch miner account from ER, trying base layer");
      minerAccount = await baseLayerProgram.account.minerAccountPoolPda.fetch(
        minerPda
      );
      console.log("Miner Account fetched from base layer.");
    }
    console.log("Miner Account: ", minerAccount);
    const difficulty = minerAccount.difficulty;

    const epoch = await getEpoch();
    const challenge = generateChallenge(payer.publicKey, epoch);
    const nonce = findValidNonce(challenge, difficulty);

    const method = program.methods
      .submitSolution(new anchor.BN(nonce), new anchor.BN(epoch))
      .accountsStrict({
        miner: minerPda,
        authority: payer.publicKey,
        miningPoolPda: miningPoolPda,
        miningPoolRewardState: miningPoolRewardState,
        systemProgram: SystemProgram.programId,
      })
      .signers([payer]);
    if (isSimulate) {
      try {
        console.log(
          `Epoch: ${epoch} | Attempt: ${attempts} | Nonce: ${nonce} | Difficulty: ${difficulty}`
        );
        console.log("Challenge: ", Uint8Array.from(challenge));
        await method.simulate();
        return false;
      } catch (error) {
        console.log(error);
        return true;
      }
    } else {
      const tx = await method.rpc();
      console.log("Submitted solution with nonce:", nonce);
      console.log("Transaction signature:", tx);
    }
  };

  it("Submit a valid PoW solution", async () => {
    let attempts = 0;
    while (true) {
      const simulateFailed = await submitSolution(true, attempts);
      console.log("Simulate failed: ", simulateFailed);
      if (!simulateFailed) {
        // Valid solution found, now actually submit it
        await submitSolution(false, attempts);
        break;
      }
      attempts++;
    }
  });
});
