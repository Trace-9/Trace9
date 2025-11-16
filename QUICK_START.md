# Trace9 Oracle - Quick Start Guide

Get started with Trace9 Oracle in 5 minutes!

## Prerequisites

- Node.js 18+
- Rust and Cargo
- Solana CLI
- Anchor Framework

## Installation

```bash
# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

# Install Anchor
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install latest
avm use latest

# Verify installations
solana --version
anchor --version
```

## Setup

```bash
# Clone and navigate to project
cd trace9

# Install dependencies
npm install
cd sdk && npm install && cd ..

# Set Solana to devnet (for testing)
solana config set --url devnet

# Generate keypair if needed
solana-keygen new

# Get test SOL
solana airdrop 2
```

## Build & Test

```bash
# Build the Anchor program
anchor build

# Run tests
anchor test
```

## Deploy to Devnet

```bash
# Deploy to devnet
anchor deploy --provider.cluster devnet

# Update program ID in Anchor.toml and lib.rs after deployment
```

## Use the SDK

```typescript
import { Trace9OracleClient, QuestionType, SOLANA_DEVNET } from '@trace9/sdk';
import { Keypair, Connection } from '@solana/web3.js';
import { Wallet } from '@coral-xyz/anchor';

// Setup
const keypair = Keypair.generate();
const wallet = new Wallet(keypair);
const connection = new Connection(SOLANA_DEVNET.rpcUrl);

// Initialize client
const client = new Trace9OracleClient({
  programId: YOUR_PROGRAM_ID,
  rpcUrl: SOLANA_DEVNET.rpcUrl,
  network: SOLANA_DEVNET.network,
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

## Next Steps

- Read the [README.md](./README.md) for detailed documentation
- Check out the [SDK README](./sdk/README.md) for API reference
- Explore the test files in `tests/` directory

