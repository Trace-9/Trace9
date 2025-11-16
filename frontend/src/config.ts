import { PublicKey, Cluster } from '@solana/web3.js';
import { Trace9Config } from '@trace9/sdk';

// Program ID (update after deployment)
export const TRACE9_PROGRAM_ID = new PublicKey('trc9oRacL3mP9vK8JqF2nH5xY7wD4bC6eA8g');

// Solana Mainnet Configuration
export const TRACE9_CONFIG: Trace9Config = {
  programId: TRACE9_PROGRAM_ID,
  rpcUrl: 'https://api.mainnet-beta.solana.com',
  network: 'mainnet-beta',
};

// Solana Devnet Configuration (for testing)
export const DEVNET_CONFIG: Trace9Config = {
  programId: TRACE9_PROGRAM_ID,
  rpcUrl: 'https://api.devnet.solana.com',
  network: 'devnet',
};

// Solana network cluster configuration
export const SOLANA_MAINNET = {
  name: 'mainnet-beta' as Cluster,
  rpcUrl: 'https://api.mainnet-beta.solana.com',
  explorer: 'https://explorer.solana.com',
};

export const SOLANA_DEVNET = {
  name: 'devnet' as Cluster,
  rpcUrl: 'https://api.devnet.solana.com',
  explorer: 'https://explorer.solana.com/?cluster=devnet',
};

// Default network
export const DEFAULT_NETWORK = SOLANA_MAINNET;

