import { PublicKey, Cluster } from '@solana/web3.js';

// Program ID (will be updated after deployment)
export const TRACE9_PROGRAM_ID = new PublicKey('trc9oRacL3mP9vK8JqF2nH5xY7wD4bC6eA8g');

// Solana network configurations
export const SOLANA_MAINNET = {
  rpcUrl: 'https://api.mainnet-beta.solana.com',
  network: 'mainnet-beta' as Cluster,
};

export const SOLANA_DEVNET = {
  rpcUrl: 'https://api.devnet.solana.com',
  network: 'devnet' as Cluster,
};

// Default oracle fee: 0.01 SOL = 10,000,000 lamports
export const DEFAULT_ORACLE_FEE = 10_000_000n;

// Refund period: 7 days in seconds
export const REFUND_PERIOD_SECONDS = 7 * 24 * 60 * 60;

// PDA seeds
export const ORACLE_STATE_SEED = 'oracle_state';
export const QUESTION_SEED = 'question';
export const ANSWER_SEED = 'answer';

