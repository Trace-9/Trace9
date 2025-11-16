# Trace9 Oracle SDK

TypeScript SDK for interacting with the Trace9 Oracle program on Solana.

## Installation

```bash
npm install @trace9/sdk
```

## Quick Start

```typescript
import { Trace9OracleClient, QuestionType, SOLANA_MAINNET } from '@trace9/sdk';
import { Connection, Keypair } from '@solana/web3.js';
import { Wallet } from '@coral-xyz/anchor';

// Setup connection and wallet
const connection = new Connection(SOLANA_MAINNET.rpcUrl);
const wallet = new Wallet(keypair);

// Initialize client
const client = new Trace9OracleClient({
  programId: TRACE9_PROGRAM_ID,
  rpcUrl: SOLANA_MAINNET.rpcUrl,
  network: SOLANA_MAINNET.network,
}, wallet);

// Ask a question
const fee = await client.getQuestionFee();
const tx = await client.askQuestion({
  questionType: QuestionType.General,
  question: "What is the price of BTC?",
  deadline: Math.floor(Date.now() / 1000) + 86400,
  fee,
});
```

## API Reference

### Trace9 Oracle Client

Main client class for interacting with the Trace9 Oracle program.

#### Methods

- `initialize(oracleProvider: PublicKey)` - Initialize the oracle program
- `askQuestion(params: AskQuestionParams)` - Ask a question to the oracle
- `provideAnswer(params: ProvideAnswerParams)` - Provide an answer (provider only)
- `refundQuestion(questionId: string)` - Refund unanswered question
- `withdraw()` - Withdraw provider earnings
- `getOracleState()` - Get current oracle state
- `getQuestion(questionId: string)` - Get question with answer if available
- `getQuestionFee()` - Get current question fee
- `setOracleFee(newFee: bigint)` - Update oracle fee (authority only)
- `setOracleProvider(newProvider: PublicKey)` - Update oracle provider (authority only)

## Types

See `src/types/index.ts` for all TypeScript type definitions.

## Constants

See `src/utils/constants.ts` for network configurations and constants.

