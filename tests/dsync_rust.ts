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
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.DsyncRust as Program<DsyncRust>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
