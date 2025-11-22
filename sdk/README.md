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

### Simple Prediction Market Client

Binary yes/no prediction market client.

#### Methods

- `initialize(oracleProgram: PublicKey, feePercentage?: number)` - Initialize the market program
- `createMarket(params: CreateSimpleMarketParams)` - Create a new binary market
- `takePosition(marketId: bigint, isYes: boolean, amount: bigint)` - Take YES or NO position
- `resolveMarket(marketId: bigint, oracleAnswerPDA: PublicKey)` - Resolve market using oracle answer
- `claimWinnings(marketId: bigint)` - Claim winnings from resolved market
- `getMarket(marketId: bigint)` - Get market details
- `getPosition(marketId: bigint, user: PublicKey)` - Get user position
- `calculateWinnings(marketId: bigint, user: PublicKey)` - Calculate potential winnings
- `getMarketPublicKey(marketId: bigint)` - Get market PDA (for conditional markets)

### Multi-Outcome Market Client

Prediction market client for markets with 2-10 outcomes (e.g., elections, tournaments).

#### Methods

- `initialize(oracleProgram: PublicKey, feePercentage?: number)` - Initialize the market program
- `createMarket(params: CreateMultiOutcomeMarketParams)` - Create market with multiple outcomes
- `takePosition(marketId: bigint, outcomeIndex: number, amount: bigint)` - Bet on specific outcome
- `resolveMarket(marketId: bigint, oracleAnswerPDA: PublicKey)` - Resolve using numeric answer (outcome index)
- `claimWinnings(marketId: bigint)` - Claim winnings for winning outcome
- `getMarket(marketId: bigint)` - Get market details
- `getPosition(marketId: bigint, user: PublicKey)` - Get user position
- `calculateWinnings(marketId: bigint, user: PublicKey)` - Calculate parimutuel winnings

### Range Market Client

Range-based prediction market client for betting on numeric value ranges.

#### Methods

- `initialize(oracleProgram: PublicKey, feePercentage?: number)` - Initialize the market program
- `createMarket(params: CreateRangeMarketParams)` - Create market with lower/upper bounds
- `takePosition(marketId: bigint, inRange: boolean, amount: bigint)` - Bet IN-RANGE or OUT-RANGE
- `resolveMarket(marketId: bigint, oracleAnswerPDA: PublicKey)` - Resolve using numeric oracle value
- `claimWinnings(marketId: bigint)` - Claim winnings based on range outcome
- `getMarket(marketId: bigint)` - Get market details
- `getPosition(marketId: bigint, user: PublicKey)` - Get user position
- `calculateWinnings(marketId: bigint, user: PublicKey)` - Calculate winnings

### Time-Series Market Client

Multi-period time-series prediction market client.

#### Methods

- `initialize(oracleProgram: PublicKey, feePercentage?: number)` - Initialize the market program
- `createMarket(params: CreateTimeSeriesMarketParams)` - Create market with multiple deadlines
- `takePosition(marketId: bigint, allSucceed: boolean, amount: bigint)` - Bet ALL-SUCCEED or ANY-FAIL
- `resolvePeriod(marketId: bigint, periodIndex: number, oracleAnswerPDA: PublicKey)` - Resolve individual period
- `claimWinnings(marketId: bigint)` - Claim winnings after all periods resolved
- `getMarket(marketId: bigint)` - Get market details with period status
- `getPosition(marketId: bigint, user: PublicKey)` - Get user position
- `calculateWinnings(marketId: bigint, user: PublicKey)` - Calculate winnings

### Conditional Market Client

Conditional prediction market client for markets dependent on parent markets.

#### Methods

- `initialize(oracleProgram: PublicKey, feePercentage?: number)` - Initialize the market program
- `createMarket(params: CreateConditionalMarketParams)` - Create market dependent on parent market
- `takePosition(marketId: bigint, isYes: boolean, amount: bigint)` - Take YES/NO position
- `checkParentMarket(marketId: bigint)` - Check if parent condition is met
- `resolveMarket(marketId: bigint, oracleAnswerPDA: PublicKey)` - Resolve conditional market
- `claimWinnings(marketId: bigint)` - Claim winnings from resolved market
- `getRefund(marketId: bigint)` - Get refund if parent condition not met
- `getMarket(marketId: bigint)` - Get market details
- `getPosition(marketId: bigint, user: PublicKey)` - Get user position
- `calculateWinnings(marketId: bigint, user: PublicKey)` - Calculate winnings

### Payment Facilitator Client

Client for batch SOL payment settlement with platform fees.

#### Methods

- `initialize(feeBasisPoints: number)` - Initialize payment facilitator
- `settlePayment(params: SettlePaymentParams)` - Settle a single payment
- `batchSettlePayments(params: BatchSettlePaymentsParams)` - Settle multiple payments
- `withdrawFees()` - Withdraw accumulated platform fees (authority only)
- `updatePlatformFee(newFeeBasisPoints: number)` - Update platform fee (authority only)

### Multi-Wallet Pool

Utility for parallel transaction execution across multiple worker wallets.

#### Methods

- `initialize()` - Initialize the wallet pool
- `executeParallelTransactions(transactions: Transaction[])` - Execute transactions in parallel
- `getWallet(index: number)` - Get wallet at index
- `fundWallet(index: number, amount: bigint)` - Fund a specific wallet

## Types

See `src/types/index.ts` for all TypeScript type definitions including:
- `QuestionType`, `AnswerStatus`, `MarketStatus`, `Outcome`
- `Question`, `Answer`, `QuestionWithAnswer`
- `SimpleMarket`, `SimplePosition`
- `MultiOutcomeMarket`, `MultiOutcomePosition`
- `RangeMarket`, `RangePosition`
- `TimeSeriesMarket`, `TimeSeriesPosition`, `TimePeriod`
- `ConditionalMarket`, `ConditionalPosition`, `ConditionalMarketStatus`
- All create market parameter types

## Constants

See `src/utils/constants.ts` for network configurations and constants.

