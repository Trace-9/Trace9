<div align="center">

# Trace9 Oracle SDK v1.0

**Permissionless oracle SDK on Solana Mainnet**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Solana](https://img.shields.io/badge/Chain-Solana_Mainnet-purple.svg)](https://solana.com/)
[![Anchor](https://img.shields.io/badge/Framework-Anchor-blue.svg)](https://www.anchor-lang.com/)
[![npm version](https://img.shields.io/npm/v/@trace9/sdk.svg)](https://www.npmjs.com/package/@trace9/sdk)
[![npm downloads](https://img.shields.io/npm/dm/@trace9/sdk.svg)](https://www.npmjs.com/package/@trace9/sdk)

</div>

---

## Overview

Trace9 Oracle is a **fully permissionless oracle SDK** built on Solana that enables trustless data feeds and prediction markets. No centralized oracles, no gatekeepers - just secure, verifiable oracle functionality powered by native SOL payments.

### Key Features

- **Permissionless Oracle** - Ask questions and get answers on-chain  
- **Prediction Markets** - Five market types: binary, multi-outcome, range, time-series, and conditional markets  
- **Native SOL Payments** - All payments use native SOL (no token contracts)  
- **Batch Operations** - Process multiple questions/answers in single transactions  
- **Payment Facilitator** - Batch SOL payments with platform fees  
- **Multi-Wallet Pool** - Parallel transaction execution across worker wallets  
- **Anchor Framework** - Built with Anchor for type-safe Solana programs  
- **TypeScript SDK** - Easy-to-use SDK for integration  
- **Solana Mainnet** - Deployed and ready for production  

---

## Quick Start

### Prerequisites

- Node.js 18+ and npm
- Rust and Cargo
- Solana CLI tools
- Anchor framework

### Installation

#### Install SDK from npm (Recommended)

```bash
npm install @trace9/sdk
```

Or with yarn:
```bash
yarn add @trace9/sdk
```

**📦 Package**: [@trace9/sdk on npm](https://www.npmjs.com/package/@trace9/sdk)

#### Install from Source

```bash
# Clone the repository
git clone https://github.com/Trace-9/Trace9.git
cd Trace9

# Install dependencies
npm install

# Install SDK dependencies
cd sdk && npm install && cd ..

# Build the program
anchor build
```

### Setup Solana Wallet

```bash
# Generate a new keypair (if needed)
solana-keygen new

# Set to devnet for testing
solana config set --url devnet

# Get some SOL for testing (devnet)
solana airdrop 2
```

---

## Project Structure

```
trace9/
│
├── programs/
│   ├── trace9/
│   │   └── src/
│   │       └── lib.rs              # Main Anchor program (oracle + batch ops)
│   ├── payment_facilitator/
│   │   └── src/
│   │       └── lib.rs              # Payment facilitator program
│   ├── simple_prediction_market/
│   │   └── src/
│   │       └── lib.rs              # Binary yes/no prediction markets
│   ├── multi_outcome_market/
│   │   └── src/
│   │       └── lib.rs              # Multi-outcome markets (2-10 outcomes)
│   ├── range_market/
│   │   └── src/
│   │       └── lib.rs              # Range-based markets
│   ├── time_series_market/
│   │   └── src/
│   │       └── lib.rs              # Multi-period time series markets
│   └── conditional_market/
│       └── src/
│           └── lib.rs              # Conditional markets (dependent on other markets)
│
├── sdk/
│   └── src/
│       ├── core/
│       │   ├── Trace9OracleClient.ts
│       │   ├── PaymentFacilitatorClient.ts
│       │   ├── MultiWalletPool.ts
│       │   ├── SimplePredictionMarketClient.ts
│       │   ├── MultiOutcomeMarketClient.ts
│       │   ├── RangeMarketClient.ts
│       │   ├── TimeSeriesMarketClient.ts
│       │   └── ConditionalMarketClient.ts
│       ├── types/
│       │   └── index.ts            # All types including prediction markets
│       └── utils/
│           └── constants.ts
│
├── tests/
│   └── trace9.ts                  # Anchor tests
│
├── frontend/
│   └── src/
│       └── config.ts              # Frontend configuration
│
├── Anchor.toml                    # Anchor configuration (all programs)
├── package.json
└── README.md
```

---

## Architecture

### Core Components

1. **Trace9 Program** (`programs/trace9/src/lib.rs`)
   - Oracle state management using PDAs
   - Question/answer functionality
   - Batch operations (ask/answer multiple questions)
   - Native SOL payment handling
   - Provider earnings tracking

2. **Payment Facilitator Program** (`programs/payment_facilitator/src/lib.rs`)
   - Batch SOL payment settlement
   - Platform fee collection (configurable basis points)
   - Payment replay prevention
   - Fee withdrawal and management

3. **Prediction Market Programs**
   - **SimplePredictionMarket** - Binary yes/no markets with oracle resolution
   - **MultiOutcomeMarket** - Markets with 2-10 outcomes (e.g., election results)
   - **RangeMarket** - Bet on whether a value falls within a specific range
   - **TimeSeriesMarket** - Multi-period markets (e.g., "Will BTC increase each month?")
   - **ConditionalMarket** - Markets dependent on other markets' outcomes

4. **TypeScript SDK** (`sdk/`)
   - `Trace9OracleClient` - Main client for oracle operations
   - `PaymentFacilitatorClient` - Payment facilitator client
   - `SimplePredictionMarketClient` - Binary prediction market client
   - `MultiOutcomeMarketClient` - Multi-outcome prediction market client (2-10 outcomes)
   - `RangeMarketClient` - Range-based prediction market client
   - `TimeSeriesMarketClient` - Time-series prediction market client
   - `ConditionalMarketClient` - Conditional prediction market client
   - `MultiWalletPool` - Multi-wallet pool for parallel transactions
   - Type definitions for questions, answers, markets, and state
   - Utility functions and constants

---

## Usage Examples

### Install the SDK

```bash
npm install @trace9/sdk
```

### Initialize the Oracle

```typescript
import { Trace9OracleClient } from '@trace9/sdk';
import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import { Wallet } from '@coral-xyz/anchor';

const connection = new Connection('https://api.mainnet-beta.solana.com');
const wallet = new Wallet(keypair); // Your wallet keypair

const client = new Trace9OracleClient({
  programId: new PublicKey('YOUR_PROGRAM_ID'),
  rpcUrl: 'https://api.mainnet-beta.solana.com',
  network: 'mainnet-beta',
}, wallet);

// Initialize oracle (authority only)
await client.initialize(oracleProviderPublicKey);
```

### Ask a Question

```typescript
import { QuestionType } from '@trace9/sdk';

const questionFee = await client.getQuestionFee();

const tx = await client.askQuestion({
  questionType: QuestionType.General,
  question: "What is the current price of BTC?",
  deadline: Math.floor(Date.now() / 1000) + 86400, // 24 hours
  fee: questionFee,
});

console.log(`Question asked! Transaction: ${tx}`);
```

### Provide an Answer

```typescript
// Oracle provider only
const tx = await client.provideAnswer({
  questionId: "0",
  textAnswer: "BTC is trading at $95,000",
  numericAnswer: 45000n,
  boolAnswer: false,
  confidenceScore: 95,
  dataSource: "CoinGecko API",
});

console.log(`Answer provided! Transaction: ${tx}`);
```

### Get Question with Answer

```typescript
const question = await client.getQuestion("0");

if (question) {
  console.log(`Question: ${question.questionText}`);
  if (question.answer) {
    console.log(`Answer: ${question.answer.numericAnswer}`);
    console.log(`Confidence: ${question.answer.confidenceScore}%`);
  }
}
```

### Batch Ask Questions

```typescript
// Ask multiple questions in a single transaction
const questionIds = await client.batchAskQuestions({
  questionTypes: [QuestionType.General, QuestionType.YesNo],
  questions: [
    "What is the price of BTC?",
    "Will Ethereum reach $5000 this year?"
  ],
  deadlines: [
    Math.floor(Date.now() / 1000) + 86400,
    Math.floor(Date.now() / 1000) + 2592000
  ]
});

console.log(`Batch questions asked: ${questionIds.join(', ')}`);
```

### Payment Facilitator

```typescript
import { PaymentFacilitatorClient } from '@trace9/sdk';
import { PublicKey } from '@solana/web3.js';
import { Wallet } from '@coral-xyz/anchor';

const paymentClient = new PaymentFacilitatorClient({
  programId: PAYMENT_FACILITATOR_PROGRAM_ID,
  rpcUrl: 'https://api.mainnet-beta.solana.com',
  network: 'mainnet-beta',
}, wallet);

// Settle a single payment
const paymentId = new Uint8Array(32); // Generate unique payment ID
await paymentClient.settlePayment({
  amount: 100_000_000n, // 0.1 SOL in lamports
  recipient: recipientPublicKey,
  paymentId: paymentId,
});

// Batch settle multiple payments
await paymentClient.batchSettlePayments({
  amounts: [100_000_000n, 50_000_000n],
  recipients: [recipient1, recipient2],
  paymentIds: [paymentId1, paymentId2],
});
```

### Multi-Wallet Pool

```typescript
import { MultiWalletPool } from '@trace9/sdk';
import { Connection } from '@solana/web3.js';

const connection = new Connection('https://api.mainnet-beta.solana.com');

const pool = new MultiWalletPool({
  masterSeed: 'your mnemonic phrase here',
  connection,
  walletCount: 10,
  autoFund: false,
});

await pool.initialize();

// Execute transactions in parallel
const transactions = [
  // ... your transactions
];

const results = await pool.executeParallelTransactions(transactions);
console.log(`Executed ${results.length} transactions in parallel`);
```

### Prediction Markets

#### Create a Binary Prediction Market

```typescript
import { SimplePredictionMarketClient } from '@trace9/sdk';
import { PublicKey } from '@solana/web3.js';
import { Wallet } from '@coral-xyz/anchor';

const marketClient = new SimplePredictionMarketClient({
  programId: SIMPLE_PREDICTION_MARKET_PROGRAM_ID,
  rpcUrl: 'https://api.mainnet-beta.solana.com',
  network: 'mainnet-beta',
}, wallet);

// Initialize the market program (first time only)
const oracleProgramId = new PublicKey('TRACE9_ORACLE_PROGRAM_ID');
await marketClient.initialize(oracleProgramId, 200); // 2% fee

// Create a new market
const marketId = await marketClient.createMarket({
  question: "Will Bitcoin reach $130,000 by end of 2025?",
  resolutionTime: Math.floor(Date.now() / 1000) + 2592000, // 30 days
});

console.log(`Market created with ID: ${marketId}`);
```

#### Take a Position

```typescript
// Bet YES on the market
const tx = await marketClient.takePosition(
  marketId,
  true, // true = YES, false = NO
  100_000_000n // 0.1 SOL in lamports
);

console.log(`Position taken! Transaction: ${tx}`);
```

#### Resolve Market

```typescript
// After resolution time, resolve using oracle answer
const oracleAnswerPDA = new PublicKey('ORACLE_ANSWER_PDA');
await marketClient.resolveMarket(marketId, oracleAnswerPDA);
```

#### Claim Winnings

```typescript
// Claim winnings after market is resolved
const winnings = await marketClient.calculateWinnings(marketId, userPublicKey);
if (winnings > 0n) {
  const tx = await marketClient.claimWinnings(marketId);
  console.log(`Winnings claimed! Transaction: ${tx}`);
}
```

#### Get Market Information

```typescript
// Get market details
const market = await marketClient.getMarket(marketId);
if (market) {
  console.log(`Question: ${market.question}`);
  console.log(`Yes Pool: ${market.yesPool} lamports`);
  console.log(`No Pool: ${market.noPool} lamports`);
  console.log(`Status: ${market.status}`);
}

// Get user position
const position = await marketClient.getPosition(marketId, userPublicKey);
if (position) {
  console.log(`Yes Amount: ${position.yesAmount}`);
  console.log(`No Amount: ${position.noAmount}`);
  console.log(`Claimed: ${position.claimed}`);
}
```

#### Multi-Outcome Market

```typescript
import { MultiOutcomeMarketClient } from '@trace9/sdk';

const multiClient = new MultiOutcomeMarketClient({
  programId: MULTI_OUTCOME_MARKET_PROGRAM_ID,
  rpcUrl: 'https://api.mainnet-beta.solana.com',
  network: 'mainnet-beta',
}, wallet);

// Initialize (first time only)
await multiClient.initialize(ORACLE_PROGRAM_ID, 200);

// Create market with multiple outcomes
const marketId = await multiClient.createMarket({
  question: "Who will win the 2024 US Presidential Election?",
  outcomeLabels: ["Candidate A", "Candidate B", "Candidate C", "Independent"],
  resolutionTime: Math.floor(Date.now() / 1000) + 2592000,
});

// Bet on outcome 0 (Candidate A)
await multiClient.takePosition(marketId, 0, 500_000_000n); // 0.5 SOL

// Resolve using oracle numeric answer (outcome index)
await multiClient.resolveMarket(marketId, oracleAnswerPDA);
```

#### Range Market

```typescript
import { RangeMarketClient } from '@trace9/sdk';

const rangeClient = new RangeMarketClient({
  programId: RANGE_MARKET_PROGRAM_ID,
  rpcUrl: 'https://api.mainnet-beta.solana.com',
  network: 'mainnet-beta',
}, wallet);

// Initialize (first time only)
await rangeClient.initialize(ORACLE_PROGRAM_ID, 200);

// Create range market
const marketId = await rangeClient.createMarket({
  question: "Will Bitcoin price be between $90,000 and $100,000?",
  lowerBound: 90_000n * 1_000_000_000n,
  upperBound: 100_000n * 1_000_000_000n,
  deadline: Math.floor(Date.now() / 1000) + 2592000,
});

// Bet IN-RANGE
await rangeClient.takePosition(marketId, true, 500_000_000n);

// Resolve using oracle numeric answer
await rangeClient.resolveMarket(marketId, oracleAnswerPDA);
```

#### Time-Series Market

```typescript
import { TimeSeriesMarketClient } from '@trace9/sdk';

const timeSeriesClient = new TimeSeriesMarketClient({
  programId: TIME_SERIES_MARKET_PROGRAM_ID,
  rpcUrl: 'https://api.mainnet-beta.solana.com',
  network: 'mainnet-beta',
}, wallet);

// Initialize (first time only)
await timeSeriesClient.initialize(ORACLE_PROGRAM_ID, 200);

// Create time-series market with multiple periods
const now = Math.floor(Date.now() / 1000);
const marketId = await timeSeriesClient.createMarket({
  question: "Will Bitcoin increase each month for 3 months?",
  deadlines: [
    now + 2592000,  // Month 1
    now + 5184000,  // Month 2
    now + 7776000,  // Month 3
  ],
});

// Bet that ALL periods will succeed
await timeSeriesClient.takePosition(marketId, true, 1_000_000_000n);

// Resolve each period sequentially as deadlines are reached
await timeSeriesClient.resolvePeriod(marketId, 0, oracleAnswerPDA1);
await timeSeriesClient.resolvePeriod(marketId, 1, oracleAnswerPDA2);
await timeSeriesClient.resolvePeriod(marketId, 2, oracleAnswerPDA3);
```

#### Conditional Market

```typescript
import { ConditionalMarketClient, SimplePredictionMarketClient } from '@trace9/sdk';

const conditionalClient = new ConditionalMarketClient({
  programId: CONDITIONAL_MARKET_PROGRAM_ID,
  rpcUrl: 'https://api.mainnet-beta.solana.com',
  network: 'mainnet-beta',
}, wallet);

// Initialize (first time only)
await conditionalClient.initialize(ORACLE_PROGRAM_ID, 200);

// First create a parent market
const simpleClient = new SimplePredictionMarketClient({
  programId: SIMPLE_MARKET_PROGRAM_ID,
  rpcUrl: 'https://api.mainnet-beta.solana.com',
  network: 'mainnet-beta',
}, wallet);

const parentMarketId = await simpleClient.createMarket({
  question: "Will Bitcoin reach $100,000?",
  resolutionTime: Math.floor(Date.now() / 1000) + 2592000,
});

const parentMarketPDA = await simpleClient.getMarketPublicKey(parentMarketId);

// Create conditional market dependent on parent
const conditionalMarketId = await conditionalClient.createMarket({
  question: "If BTC reaches $100k, will it stay above $100k for 30 days?",
  parentMarket: parentMarketPDA,
  requiredParentOutcome: 1, // 1 = YES, 0 = NO
});

// Bet on conditional market (only valid if parent resolves YES)
await conditionalClient.takePosition(conditionalMarketId, true, 500_000_000n);

// Check if parent condition is met
const parentMet = await conditionalClient.checkParentMarket(conditionalMarketId);

if (parentMet) {
  // Resolve conditional market
  await conditionalClient.resolveMarket(conditionalMarketId, oracleAnswerPDA);
} else {
  // Get refund if condition not met
  await conditionalClient.getRefund(conditionalMarketId);
}
```

---

## Testing

```bash
# Run Anchor tests
anchor test

# Run with specific validator
anchor test --skip-local-validator

# Run tests on devnet
anchor test --provider.cluster devnet
```

---

## Program Instructions

### Trace9 Oracle Program

- `initialize` - Initialize the oracle program (authority only)
- `ask_question` - Ask a question to the oracle (pay with SOL)
- `provide_answer` - Provide an answer (oracle provider only)
- `batch_ask_questions` - Ask multiple questions in one transaction
- `batch_provide_answers` - Provide answers to multiple questions
- `refund_question` - Refund unanswered question after 7 days
- `withdraw` - Withdraw provider earnings
- `set_oracle_fee` - Update oracle fee (authority only)
- `set_oracle_provider` - Update oracle provider (authority only)

### Payment Facilitator Program

- `initialize` - Initialize payment facilitator (authority only)
- `settle_payment` - Settle a single payment with platform fee
- `batch_settle_payments` - Settle multiple payments in one transaction
- `withdraw_fees` - Withdraw accumulated platform fees (authority only)
- `update_platform_fee` - Update platform fee percentage (authority only)

### Simple Prediction Market Program

- `initialize` - Initialize prediction market program (authority only)
- `create_market` - Create a new binary prediction market
- `take_position` - Take a YES or NO position on a market
- `resolve_market` - Resolve market using oracle answer
- `claim_winnings` - Claim winnings from resolved market
- `cancel_market` - Cancel market if oracle hasn't answered (after 7 days)
- `claim_refund` - Claim refund from canceled market
- `withdraw_fees` - Withdraw accumulated platform fees (authority only)

### Multi-Outcome Market Program

- `initialize` - Initialize multi-outcome market program
- `create_market` - Create market with 2-10 outcomes
- `take_position` - Bet on a specific outcome
- `resolve_market` - Resolve using oracle numeric answer (outcome index)
- `claim_winnings` - Claim winnings for winning outcome

### Range Market Program

- `initialize` - Initialize range market program
- `create_market` - Create market with lower/upper bounds
- `take_position` - Bet on in-range or out-of-range
- `resolve_market` - Resolve using oracle numeric answer (check if in range)
- `claim_winnings` - Claim winnings based on range outcome

### Time Series Market Program

- `initialize` - Initialize time series market program
- `create_market` - Create market with multiple time periods (2-12)
- `take_position` - Bet on all periods succeeding or any failing
- `resolve_period` - Resolve individual period using oracle
- `claim_winnings` - Claim winnings after all periods resolved

### Conditional Market Program

- `initialize` - Initialize conditional market program
- `create_market` - Create market dependent on parent market
- `take_position` - Take position in conditional market
- `check_parent_market` - Check if parent condition is met
- `resolve_market` - Resolve conditional market (if condition met)
- `claim_winnings` - Claim winnings from resolved market
- `get_refund` - Get refund if condition not met

---

## Network Information

### Solana Mainnet

- **RPC URL**: `https://api.mainnet-beta.solana.com`
- **Explorer**: `https://explorer.solana.com`
- **Network**: `mainnet-beta`

### Solana Devnet (Testing)

- **RPC URL**: `https://api.devnet.solana.com`
- **Explorer**: `https://explorer.solana.com/?cluster=devnet`
- **Network**: `devnet`

---

## Security

### Program Security

- **Anchor Framework** - Type-safe Solana program development
- **PDA-based State** - Secure account management using Program Derived Addresses
- **Access Control** - Role-based permissions (authority, oracle provider)
- **Input Validation** - All inputs validated before processing
- **Native SOL Handling** - Secure SOL transfers using System Program
- **Batch Processing** - Gas-efficient batch operations for multiple questions/payments
- **Payment Replay Prevention** - Unique payment IDs prevent double-spending
- **Platform Fees** - Configurable fee system for payment facilitator
- **Prediction Markets** - Five market types with oracle integration for resolution
- **Market Types** - Binary, multi-outcome, range, time-series, and conditional markets
- **Parimutuel Pools** - Automatic payout distribution based on pool sizes

---

## Documentation

### Getting Started

1. **Install Dependencies**: `npm install && cd sdk && npm install`
2. **Build Program**: `anchor build`
3. **Run Tests**: `anchor test`
4. **Deploy**: `anchor deploy --provider.cluster devnet`

### SDK Documentation

- **npm Package**: [@trace9/sdk](https://www.npmjs.com/package/@trace9/sdk)
- **SDK README**: See `sdk/README.md` for detailed SDK usage and API documentation
- **Installation**: `npm install @trace9/sdk`

---

## Contributing

We welcome contributions! This is open-source MIT licensed software.

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

---

## License

**MIT License** - See [LICENSE](./LICENSE) for details.

Use it, fork it, modify it - whatever you want! All programs are fully permissionless and open source.

---

## Why Trace9?

### vs. Traditional Oracles

| Feature | Trace9 | Chainlink | Pyth |
|---------|--------|-----------|------|
| **Permissionless** | Yes - Anyone can ask | No - Whitelisted nodes | Limited |
| **Native Payments** | SOL | LINK tokens | Pyth tokens |
| **Prediction Markets** | 5 market types | Limited | Limited |
| **Batch Operations** | Yes - Built-in | Limited | Limited |
| **Parallel Execution** | Multi-wallet pool | Sequential | Sequential |
| **Solana Native** | Built for Solana | EVM only | Solana |
| **Low Fees** | ~$0.00025 per tx | Higher gas | Low fees |
| **Open Source** | MIT License | Proprietary | Proprietary |

---

## Packages

### npm Package

The Trace9 SDK is available on npm:

```bash
npm install @trace9/sdk
```

- **Package**: [@trace9/sdk](https://www.npmjs.com/package/@trace9/sdk)
- **Version**: [![npm version](https://img.shields.io/npm/v/@trace9/sdk.svg)](https://www.npmjs.com/package/@trace9/sdk)
- **Downloads**: [![npm downloads](https://img.shields.io/npm/dm/@trace9/sdk.svg)](https://www.npmjs.com/package/@trace9/sdk)

---

## Support & Community

- **GitHub Issues**: [Report bugs & request features](https://github.com/Trace-9/Trace9/issues)
- **npm Package**: [@trace9/sdk](https://www.npmjs.com/package/@trace9/sdk)
- **Documentation**: See `docs/` directory for detailed guides

---

<div align="center">

**Trace9 Oracle v1.0** - Permissionless oracle & prediction markets on Solana

Permissionless | Native SOL | Prediction Markets | Solana Mainnet | Open Source

Built by the community, for the community

</div>

