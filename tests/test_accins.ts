import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorProvider } from "@coral-xyz/anchor";
import { TestAccins } from "../target/types/test_accins";
import { expect } from "chai";
import { SystemProgram, PublicKey, Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";

describe("test_accins", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env()); // provider set
  const provider = anchor.getProvider() as AnchorProvider;

  const program = anchor.workspace.testAccins as Program<TestAccins>;

  const wallet = provider.wallet as anchor.Wallet;

  const depositLamports  = new anchor.BN(200_000_000); // 0.2 SOL
  const withdrawLamports = new anchor.BN(100_000_000); // 0.1 SOL

  const receiver = Keypair.generate();

  // need to get the PDA
  const [vaultPda] = PublicKey.findProgramAddressSync(

    [Buffer.from("vault"), wallet.publicKey.toBuffer()], // seeds
    program.programId // program id

  );

  const getBal = (pk: PublicKey) =>
      provider.connection.getBalance(pk, { commitment: "confirmed" });

  it("init_vault → deposit → withdraw (with stepwise assertions)", async () => {

    // currently using depracated confirmTransaction version!, update later
    // airdrops
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(wallet.publicKey, 2 * LAMPORTS_PER_SOL),
      "confirmed"
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(receiver.publicKey, 0.001 * LAMPORTS_PER_SOL),
      "confirmed"
    );

    const startVault = await getBal([vaultPda][0]);
    const startRecv  = await getBal(receiver.publicKey);

    // init vault
    await program.methods.initVault()
      .accounts({ sender: wallet.publicKey })
      .rpc();

    const afterInitVault = await getBal([vaultPda][0]);
    expect(afterInitVault).to.be.greaterThan(startVault); // rent got funded

    // deposit to vault
    await program.methods.deposit(depositLamports)
      .accounts({ sender: wallet.publicKey })
      .rpc();

    const afterDepositVault = await getBal([vaultPda][0]);
    expect(afterDepositVault).to.be.greaterThan(afterInitVault); // increased by ~deposit

    // withdraw from vault
    await program.methods.withdraw(withdrawLamports)
      .accounts({
        payer: wallet.publicKey,
        receiver: receiver.publicKey,
      })
      .rpc();

    const afterWithdrawVault = await getBal([vaultPda][0]);
    const afterRecv = await getBal(receiver.publicKey);

    // vault decreased by ~withdraw
    expect(afterWithdrawVault).to.equal(afterDepositVault - withdrawLamports.toNumber());
    // receiver increased by ~withdraw
    expect(afterRecv).to.equal(startRecv + withdrawLamports.toNumber());


  })

});

// rust program vs ts program diffs, rust needs the declare id, it needs the program entry, needs custom account structs, context structs, err code.
// the idl tries to mitigate this issue, allows for us to read data in a more digestible manner
