/**
 * Example: Solana Wallet Adapter Setup for Trace9 Frontend
 * 
 * This file demonstrates how to setup wallet connections
 * with @solana/wallet-adapter-react for Trace9 Oracle frontend integration.
 * 
 * Install required packages:
 * npm install @solana/wallet-adapter-react @solana/wallet-adapter-react-ui @solana/wallet-adapter-wallets @solana/wallet-adapter-base
 */

import React, { FC, ReactNode, useMemo } from 'react';
import { ConnectionProvider, WalletProvider } from '@solana/wallet-adapter-react';
import { WalletAdapterNetwork } from '@solana/wallet-adapter-base';
import { WalletModalProvider } from '@solana/wallet-adapter-react-ui';
import {
  PhantomWalletAdapter,
  SolflareWalletAdapter,
  TorusWalletAdapter,
} from '@solana/wallet-adapter-wallets';
import { clusterApiUrl } from '@solana/web3.js';
import { TRACE9_CONFIG, DEFAULT_NETWORK } from './config';

// Import wallet adapter CSS
import '@solana/wallet-adapter-react-ui/styles.css';

interface WalletContextProviderProps {
  children: ReactNode;
}

export const WalletContextProvider: FC<WalletContextProviderProps> = ({ children }) => {
  // Use mainnet-beta or devnet
  const network = DEFAULT_NETWORK.name as WalletAdapterNetwork;
  
  // Use the RPC endpoint from DEFAULT_NETWORK config, fallback to clusterApiUrl
  const endpoint = useMemo(() => {
    return DEFAULT_NETWORK.rpcUrl || clusterApiUrl(network);
  }, [network]);

  // Initialize wallet adapters
  const wallets = useMemo(
    () => [
      new PhantomWalletAdapter(),
      new SolflareWalletAdapter(),
      new TorusWalletAdapter(),
    ],
    []
  );

  return (
    <ConnectionProvider endpoint={endpoint}>
      <WalletProvider wallets={wallets} autoConnect>
        <WalletModalProvider>
          {children}
        </WalletModalProvider>
      </WalletProvider>
    </ConnectionProvider>
  );
};

/**
 * Example usage in your App component:
 * 
 * import { WalletContextProvider } from './wallet-setup';
 * import { useWallet } from '@solana/wallet-adapter-react';
 * import { Trace9OracleClient } from '@trace9/sdk';
 * import { Wallet } from '@coral-xyz/anchor';
 * 
 * function App() {
 *   const { publicKey, signTransaction, signAllTransactions } = useWallet();
 *   
 *   // Create wallet adapter for Anchor
 *   const wallet = useMemo(() => {
 *     if (!publicKey || !signTransaction) return null;
 *     return {
 *       publicKey,
 *       signTransaction,
 *       signAllTransactions,
 *     } as Wallet;
 *   }, [publicKey, signTransaction, signAllTransactions]);
 *   
 *   // Initialize Trace9 client when wallet is connected
 *   const client = useMemo(() => {
 *     if (!wallet) return null;
 *     return new Trace9OracleClient(TRACE9_CONFIG, wallet);
 *   }, [wallet]);
 *   
 *   return (
 *     <WalletContextProvider>
 *       <YourAppContent />
 *     </WalletContextProvider>
 *   );
 * }
 */

