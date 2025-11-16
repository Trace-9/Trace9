import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Trace9 } from "../target/types/trace9";
import { PublicKey, Keypair, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { expect } from "chai";

describe("trace9", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Trace9 as Program<Trace9>;
  
  const authority = provider.wallet;
  const oracleProvider = Keypair.generate();
  const requester = Keypair.generate();

  let oracleStatePDA: PublicKey;
  let oracleBump: number;

  before(async () => {
    // Airdrop SOL to test accounts
    const airdropAmount = 2 * LAMPORTS_PER_SOL;
    await provider.connection.requestAirdrop(oracleProvider.publicKey, airdropAmount);
    await provider.connection.requestAirdrop(requester.publicKey, airdropAmount);
    
    // Wait for airdrops to confirm
    await new Promise(resolve => setTimeout(resolve, 1000));

    // Find oracle state PDA
    [oracleStatePDA, oracleBump] = await PublicKey.findProgramAddress(
      [Buffer.from("oracle_state")],
      program.programId
    );
  });

  it("Initializes the oracle", async () => {
    try {
      const tx = await program.methods
        .initialize(oracleProvider.publicKey)
        .accounts({
          oracleState: oracleStatePDA,
          authority: authority.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      console.log("Initialize transaction:", tx);

      // Fetch and verify oracle state
      const state = await program.account.oracleState.fetch(oracleStatePDA);
      expect(state.authority.toString()).to.equal(authority.publicKey.toString());
      expect(state.oracleProvider.toString()).to.equal(oracleProvider.publicKey.toString());
      expect(state.questionCounter.toNumber()).to.equal(0);
      expect(state.oracleFee.toNumber()).to.equal(10_000_000); // 0.01 SOL
    } catch (error) {
      // If already initialized, that's okay
      if (error.message && error.message.includes("already in use")) {
        console.log("Oracle already initialized, continuing...");
      } else {
        throw error;
      }
    }
  });

  it("Asks a question", async () => {
    const questionType = { general: {} };
    const question = "What is the price of BTC?";
    const deadline = Math.floor(Date.now() / 1000) + 86400; // 24 hours from now

    // Get current question counter
    const state = await program.account.oracleState.fetch(oracleStatePDA);
    const questionId = state.questionCounter.toNumber();

    // Find question PDA
    const questionIdBuffer = Buffer.allocUnsafe(8);
    questionIdBuffer.writeBigUInt64LE(BigInt(questionId), 0);
    
    const [questionPDA] = await PublicKey.findProgramAddress(
      [Buffer.from("question"), questionIdBuffer],
      program.programId
    );

    const tx = await program.methods
      .askQuestion(questionType, question, new anchor.BN(deadline))
      .accounts({
        questionAccount: questionPDA,
        oracleState: oracleStatePDA,
        requester: requester.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([requester])
      .rpc();

    console.log("Ask question transaction:", tx);

    // Fetch and verify question
    const questionAccount = await program.account.questionAccount.fetch(questionPDA);
    expect(questionAccount.requester.toString()).to.equal(requester.publicKey.toString());
    expect(questionAccount.status).to.deep.equal({ pending: {} });
    expect(questionAccount.bounty.toNumber()).to.equal(10_000_000);
  });

  it("Provides an answer", async () => {
    // Get current question counter
    const state = await program.account.oracleState.fetch(oracleStatePDA);
    const questionId = state.questionCounter.toNumber() - 1; // Last question

    const questionIdBuffer = Buffer.allocUnsafe(8);
    questionIdBuffer.writeBigUInt64LE(BigInt(questionId), 0);
    
    const [questionPDA] = await PublicKey.findProgramAddress(
      [Buffer.from("question"), questionIdBuffer],
      program.programId
    );

    const [answerPDA] = await PublicKey.findProgramAddress(
      [Buffer.from("answer"), questionIdBuffer],
      program.programId
    );

    const tx = await program.methods
      .provideAnswer(
        "BTC is trading at $45,000",
        new anchor.BN(45000),
        false,
        95,
        "CoinGecko API"
      )
      .accounts({
        questionAccount: questionPDA,
        answerAccount: answerPDA,
        oracleState: oracleStatePDA,
        oracleProvider: oracleProvider.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([oracleProvider])
      .rpc();

    console.log("Provide answer transaction:", tx);

    // Fetch and verify answer
    const answerAccount = await program.account.answerAccount.fetch(answerPDA);
    expect(answerAccount.provider.toString()).to.equal(oracleProvider.publicKey.toString());
    expect(answerAccount.confidenceScore).to.equal(95);
    expect(answerAccount.numericAnswer.toNumber()).to.equal(45000);

    // Verify question status updated
    const questionAccount = await program.account.questionAccount.fetch(questionPDA);
    expect(questionAccount.status).to.deep.equal({ answered: {} });
  });

  it("Retrieves question with answer", async () => {
    const state = await program.account.oracleState.fetch(oracleStatePDA);
    const questionId = state.questionCounter.toNumber() - 1;

    const questionIdBuffer = Buffer.allocUnsafe(8);
    questionIdBuffer.writeBigUInt64LE(BigInt(questionId), 0);
    
    const [questionPDA] = await PublicKey.findProgramAddress(
      [Buffer.from("question"), questionIdBuffer],
      program.programId
    );

    const [answerPDA] = await PublicKey.findProgramAddress(
      [Buffer.from("answer"), questionIdBuffer],
      program.programId
    );

    const questionAccount = await program.account.questionAccount.fetch(questionPDA);
    const answerAccount = await program.account.answerAccount.fetch(answerPDA);

    expect(questionAccount.status).to.deep.equal({ answered: {} });
    expect(answerAccount.numericAnswer.toNumber()).to.equal(45000);
  });
});

