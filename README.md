<div align="center">

# Trace9 Oracle SDK v1.0

**Permissionless oracle SDK on Solana Mainnet**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Solana](https://img.shields.io/badge/Chain-Solana_Mainnet-purple.svg)](https://solana.com/)
[![Anchor](https://img.shields.io/badge/Framework-Anchor-blue.svg)](https://www.anchor-lang.com/)

</div>

---

## Overview

Trace9 Oracle is a **fully permissionless oracle SDK** built on Solana that enables trustless data feeds and prediction markets. No centralized oracles, no gatekeepers - just secure, verifiable oracle functionality powered by native SOL payments.

### Key Features

- **Permissionless Oracle** - Ask questions and get answers on-chain  
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

```bash
# Clone the repository
git clone <repository-url>
cd trace9

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
│   └── payment_facilitator/
│       └── src/
│           └── lib.rs              # Payment facilitator program
│
├── sdk/
│   └── src/
│       ├── core/
│       │   ├── Trace9OracleClient.ts
│       │   ├── PaymentFacilitatorClient.ts
│       │   └── MultiWalletPool.ts
│       ├── types/
│       │   └── index.ts
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
├── Anchor.toml                    # Anchor configuration
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

3. **TypeScript SDK** (`sdk/`)
   - `Trace9OracleClient` - Main client for oracle operations
   - `PaymentFacilitatorClient` - Payment facilitator client
   - `MultiWalletPool` - Multi-wallet pool for parallel transactions
   - Type definitions for questions, answers, and state
   - Utility functions and constants

---

## Usage Examples

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
  textAnswer: "BTC is trading at $45,000",
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

---

## Documentation

### Getting Started

1. **Install Dependencies**: `npm install && cd sdk && npm install`
2. **Build Program**: `anchor build`
3. **Run Tests**: `anchor test`
4. **Deploy**: `anchor deploy --provider.cluster devnet`

### SDK Documentation

See `sdk/README.md` for detailed SDK usage and API documentation.

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
| **Batch Operations** | Yes - Built-in | Limited | Limited |
| **Parallel Execution** | Multi-wallet pool | Sequential | Sequential |
| **Solana Native** | Built for Solana | EVM only | Solana |
| **Low Fees** | ~$0.00025 per tx | Higher gas | Low fees |
| **Open Source** | MIT License | Proprietary | Proprietary |

---

## Support & Community

- **GitHub Issues**: [Report bugs & request features](https://github.com/trace9/trace9/issues)
- **Documentation**: See `docs/` directory for detailed guides

---

<div align="center">

**Trace9 Oracle v1.0** - Permissionless oracle on Solana

Permissionless | Native SOL | Solana Mainnet | Open Source

Built by the community, for the community

</div>

