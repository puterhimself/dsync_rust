import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { DsyncRust } from "../target/types/dsync_rust";

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
