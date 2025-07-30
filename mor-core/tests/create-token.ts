import * as anchor from "@coral-xyz/anchor";
import { getKeypairFromFile } from "@solana-developers/helpers";
import {
  createMint,
  getMint,
  getAccount,
  mintTo,
  getOrCreateAssociatedTokenAccount,
} from "@solana/spl-token";
import { PublicKey } from "@solana/web3.js";
import { assert } from "chai";

describe("bpl_base_program", () => {
  anchor.setProvider(anchor.AnchorProvider.env());

  const wallet = anchor.Wallet.local(); // Or your wallet setup

  let rpcHttpUrl = "http://127.0.0.1:8899";
  let rpcWsUrl = "ws://127.0.0.1:8900";

  const connection = new anchor.web3.Connection(rpcHttpUrl, {
    wsEndpoint: rpcWsUrl,
  });
  const provider = new anchor.AnchorProvider(connection, wallet, {
    preflightCommitment: "processed",
  });

  const TEST_TOKEN = new PublicKey(
    "tesnw8eLyAwSR5oGzGogWcAuJhp4pynBzjKvs6kvw9T"
  );

  it.only("Create Test Token", async () => {
    try {
      const testTokenKeypair = await getKeypairFromFile(
        "./target/deploy/test-token.json"
      );

      const mint = await createMint(
        provider.connection,
        wallet.payer,
        wallet.publicKey,
        null,
        0,
        testTokenKeypair
      );
      console.log("Token Minted");
      console.log("Test Token: ", mint.toBase58());
      const mintInfo = await getMint(connection, TEST_TOKEN);
      assert.ok(mintInfo.address.equals(TEST_TOKEN));
    } catch (error) {
      console.error(error);
    }
  });

  it.only("Increase Test Token supply", async () => {
    try {
      const initialMintInfo = await getMint(connection, TEST_TOKEN);

      // Amount to mint (e.g. 1000 tokens)
      const amountToMint = BigInt(1000 * 10 ** initialMintInfo.decimals);

      // Mint new tokens to the mint authority (wallet.publicKey in this case)
      // First, create a token account for the mint authority if it doesn't exist
      const tokenAccount = await getOrCreateAssociatedTokenAccount(
        connection,
        wallet.payer,
        TEST_TOKEN,
        wallet.publicKey
      );

      // Mint additional tokens to the token account
      const mintToSignature = await mintTo(
        connection,
        wallet.payer,
        TEST_TOKEN,
        tokenAccount.address,
        wallet.publicKey, // mint authority
        BigInt(amountToMint)
      );

      console.log("Mint transaction signature:", mintToSignature);

      // Get the updated supply
      const updatedMintInfo = await getMint(connection, TEST_TOKEN);
      console.log("Updated supply:", updatedMintInfo.supply.toString());

      // Check the balance of the token account
      const tokenAccountInfo = await getAccount(
        connection,
        tokenAccount.address
      );
      console.log("Token account balance:", tokenAccountInfo.amount.toString());
    } catch (error) {
      console.error("Error increasing Test Token supply:", error);
      throw error;
    }
  });
});
