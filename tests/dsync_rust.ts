import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { IDL } from "../target/types/dsync_rust";
import { DsyncRust } from "../target/types/dsync_rust";
// import { IDL } from "../target/types/anchor_escrow";
import { PublicKey, SystemProgram, Transaction, Connection, Commitment } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createMint, createAccount, mintTo, getAccount } from "@solana/spl-token";
import NodeWallet from "@project-serum/anchor/dist/cjs/nodewallet";
import { assert, expect, should } from "chai";



describe("dsync_rust", () => {
  // Configure the client to use the local cluster.
  // Use Mainnet-fork for testing
  const commitment: Commitment = "confirmed";
  const connection = new Connection("https://rpc-mainnet-fork.epochs.studio", {
    commitment,
    wsEndpoint: "wss://rpc-mainnet-fork.epochs.studio/ws",
  });
  const options = anchor.AnchorProvider.defaultOptions();
  const wallet = NodeWallet.local();
  const provider = new anchor.AnchorProvider(connection, wallet, options);

  const CLIENT_SEED = "DSYNC_CLIENT";
  const VAULT_SEED = "DSYNC_VAULT";
  const AUTHORITY_SEED = "DSYNC_AUTHORITY";
  const JOB_SEED = "DSYNC_JOB";
  const SUBMISSION_SEED = "DSYNC_SUBMISSION";

  enum JobState {
    PENDING,
    PUBLISHED,
    ACTIVE,
    VALIDATED,
    COMPLETED,
    CANCELED,
  }
  anchor.setProvider(provider);
  // anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.DsyncRust as Program<DsyncRust>;

  let mintA = null as PublicKey;
  let mintB = null as PublicKey;
  
  let initializerTokenAccountA = null as PublicKey;
  let initializerTokenAccountB = null as PublicKey;
  
  let workerTokenAccountA = null as PublicKey;
  let workerTokenAccountB = null as PublicKey;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initializeClient().rpc();
    console.log("Your transaction signature", tx);
  });
});
