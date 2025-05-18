import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolanaDepositProgram } from "../target/types/solana_deposit_program";
import { assert } from "chai";

describe("solana-deposit-program", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace
    .SolanaDepositProgram as Program<SolanaDepositProgram>;
  const user = anchor.web3.Keypair.generate();

  before(async () => {
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(
        user.publicKey,
        10000000000 // 10 SOL
      ),
      "confirmed"
    );
  });

  it("Deposit SOL", async () => {
    const [userAccountPDA] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("user_account"), user.publicKey.toBuffer()],
      program.programId
    );

    const depositAmount = new anchor.BN(100000000); // 0.1 SOL
    await program.methods
      .deposit(depositAmount)
      .accounts({
        user: user.publicKey,
        userAccount: userAccountPDA,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    const account = await program.account.userAccount.fetch(userAccountPDA);
    assert.isTrue(account.owner.equals(user.publicKey));

    const balance = await provider.connection.getBalance(userAccountPDA);
    const rentExempt = await provider.connection.getMinimumBalanceForRentExemption(
      40 // 8 (discriminator) + 32 (owner)
    );
    assert.equal(balance, rentExempt + depositAmount.toNumber());
  });

  it("Withdraw SOL", async () => {
    const [userAccountPDA] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("user_account"), user.publicKey.toBuffer()],
      program.programId
    );

    const withdrawAmount = new anchor.BN(50000000); // 0.05 SOL
    await program.methods
      .withdraw(withdrawAmount)
      .accounts({
        user: user.publicKey,
        userAccount: userAccountPDA,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    const balance = await provider.connection.getBalance(userAccountPDA);
    const rentExempt = await provider.connection.getMinimumBalanceForRentExemption(40);
    assert.equal(balance, rentExempt + 50000000);
  });
});
