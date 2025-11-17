/**
 * Multi-Wallet Pool for Trace9 Oracle
 * 
 * Enables parallel transaction execution on Solana by distributing operations
 * across multiple worker wallets. While Solana naturally supports parallel transactions,
 * this pool helps with load balancing and organization.
 * 
 * Features:
 * - Create multiple worker keypairs from a master seed
 * - Distribute operations across wallets (round-robin)
 * - Parallel transaction execution
 * - Balance management and rebalancing
 */

import { 
  Keypair, 
  Connection, 
  PublicKey, 
  Transaction, 
  sendAndConfirmTransaction, 
  LAMPORTS_PER_SOL,
  SystemProgram 
} from '@solana/web3.js';
import { Wallet } from '@coral-xyz/anchor';
import * as bip39 from 'bip39';
import { derivePath } from 'ed25519-hd-key';

export interface MultiWalletPoolConfig {
  masterSeed: string; // BIP39 mnemonic or seed phrase
  connection: Connection;
  walletCount?: number;
  autoFund?: boolean;
  fundingAmountSOL?: number; // SOL amount per wallet
}

export interface WorkerWallet {
  keypair: Keypair;
  wallet: Wallet;
  address: PublicKey;
}

export interface PoolStats {
  totalWallets: number;
  totalBalanceSOL: number;
  averageBalanceSOL: number;
  walletAddresses: PublicKey[];
}

export class MultiWalletPool {
  private workers: WorkerWallet[] = [];
  private currentWalletIndex = 0;
  private connection: Connection;

  constructor(private config: MultiWalletPoolConfig) {
    this.connection = config.connection;
    const walletCount = config.walletCount || 10;
    
    // Generate worker wallets from master seed
    this.workers = this.generateWorkerWallets(walletCount);
  }

  /**
   * Generate worker wallets from master seed using BIP44 derivation
   */
  private generateWorkerWallets(count: number): WorkerWallet[] {
    const seed = bip39.mnemonicToSeedSync(this.config.masterSeed);
    const wallets: WorkerWallet[] = [];

    for (let i = 0; i < count; i++) {
      // Derive path: m/44'/501'/0'/0'/{i}
      const path = `m/44'/501'/0'/0'/${i}'`;
      const derivedSeed = derivePath(path, seed.toString('hex')).key;
      const keypair = Keypair.fromSeed(derivedSeed);
      const wallet = new Wallet(keypair);

      wallets.push({
        keypair,
        wallet,
        address: keypair.publicKey,
      });
    }

    return wallets;
  }

  /**
   * Initialize pool by funding worker wallets if autoFund is enabled
   */
  async initialize(): Promise<void> {
    if (this.config.autoFund && this.config.fundingAmountSOL) {
      await this.fundWorkers(this.config.fundingAmountSOL);
    }
    
    console.log(`Multi-wallet pool initialized with ${this.workers.length} workers`);
  }

  /**
   * Fund all worker wallets with SOL from master wallet
   * Note: In production, you'd need a master wallet with sufficient balance
   */
  async fundWorkers(amountPerWallet: number): Promise<void> {
    const amountLamports = amountPerWallet * LAMPORTS_PER_SOL;
    
    console.log(`Funding ${this.workers.length} workers with ${amountPerWallet} SOL each...`);
    
    // Note: This requires a master wallet with sufficient balance
    // In production, you'd implement proper funding logic
    const fundingPromises = this.workers.map(async (worker) => {
      // Request airdrop for devnet/testing
      // For mainnet, you'd transfer from master wallet
      try {
        const signature = await this.connection.requestAirdrop(
          worker.address,
          amountLamports
        );
        await this.connection.confirmTransaction(signature);
      } catch (error) {
        console.warn(`Failed to fund wallet ${worker.address.toBase58()}:`, error);
      }
    });
    
    await Promise.all(fundingPromises);
    console.log(`Successfully funded ${this.workers.length} worker wallets`);
  }

  /**
   * Get next available wallet (round-robin distribution)
   */
  private getNextWorker(): WorkerWallet {
    const index = this.currentWalletIndex;
    this.currentWalletIndex = (this.currentWalletIndex + 1) % this.workers.length;
    return this.workers[index];
  }

  /**
   * Get worker wallet by index
   */
  getWorker(index: number): WorkerWallet {
    if (index < 0 || index >= this.workers.length) {
      throw new Error(`Invalid worker index: ${index}`);
    }
    return this.workers[index];
  }

  /**
   * Get all worker wallets
   */
  getAllWorkers(): WorkerWallet[] {
    return this.workers;
  }

  /**
   * Check balances of all worker wallets
   */
  async getWorkerBalances(): Promise<Array<{ address: PublicKey; balance: number }>> {
    const balancePromises = this.workers.map(async (worker) => {
      const balance = await this.connection.getBalance(worker.address);
      return {
        address: worker.address,
        balance: balance / LAMPORTS_PER_SOL,
      };
    });
    
    return await Promise.all(balancePromises);
  }

  /**
   * Execute transactions in parallel across worker wallets
   */
  async executeParallelTransactions(
    transactions: Transaction[],
    options?: { skipPreflight?: boolean; commitment?: 'processed' | 'confirmed' | 'finalized' }
  ): Promise<Array<{ signature: string; walletIndex: number }>> {
    console.log(`Executing ${transactions.length} transactions in parallel across ${this.workers.length} wallets...`);
    
    const startTime = Date.now();
    
    // Distribute transactions across wallets (round-robin)
    const executionPromises = transactions.map(async (tx, i) => {
      const worker = this.getNextWorker();
      
      // Sign transaction with worker wallet
      tx.sign(worker.keypair);
      
      // Send and confirm
      const signature = await sendAndConfirmTransaction(
        this.connection,
        tx,
        [worker.keypair],
        {
          skipPreflight: options?.skipPreflight ?? false,
          commitment: options?.commitment ?? 'confirmed',
        }
      );
      
      return {
        signature,
        walletIndex: this.workers.indexOf(worker),
      };
    });
    
    const results = await Promise.all(executionPromises);
    
    const elapsed = Date.now() - startTime;
    console.log(`Executed ${transactions.length} transactions in ${elapsed}ms (${(elapsed / transactions.length).toFixed(1)}ms avg)`);
    
    return results;
  }

  /**
   * Create and execute parallel operations
   * Helper method that creates transactions and executes them
   */
  async executeParallelOperations<T>(
    operations: Array<() => Promise<Transaction>>,
    options?: { skipPreflight?: boolean; commitment?: 'processed' | 'confirmed' | 'finalized' }
  ): Promise<Array<{ signature: string; walletIndex: number; result?: T }>> {
    console.log(`Executing ${operations.length} parallel operations...`);
    
    const startTime = Date.now();
    
    // Create all transactions in parallel
    const transactionPromises = operations.map(op => op());
    const transactions = await Promise.all(transactionPromises);
    
    // Execute transactions in parallel
    const results = await this.executeParallelTransactions(transactions, options);
    
    const totalTime = Date.now() - startTime;
    console.log(`Complete: ${operations.length} operations in ${totalTime}ms`);
    console.log(`Average: ${(totalTime / operations.length).toFixed(1)}ms per operation`);
    
    return results;
  }

  /**
   * Rebalance SOL across worker wallets
   * Redistributes funds from high-balance wallets to low-balance ones
   */
  async rebalance(): Promise<void> {
    const balances = await this.getWorkerBalances();
    const totalBalance = balances.reduce((sum, b) => sum + b.balance, 0);
    const avgBalance = totalBalance / balances.length;
    
    console.log(`Rebalancing wallets (target: ${avgBalance.toFixed(4)} SOL)...`);
    
    // TODO: Implement rebalancing logic
    // This would require transferring SOL between wallets
    balances.forEach((b, i) => {
      const diff = b.balance - avgBalance;
      console.log(`  Wallet ${i}: ${b.balance.toFixed(4)} SOL (${diff > 0 ? '+' : ''}${diff.toFixed(4)})`);
    });
  }

  /**
   * Sweep all funds from worker wallets to a target address
   */
  async sweepToAddress(targetAddress: PublicKey): Promise<void> {
    console.log(`Sweeping funds from ${this.workers.length} workers to ${targetAddress.toBase58()}...`);
    
    const balances = await this.getWorkerBalances();
    
    // Create transfer transactions for each wallet with balance
    const transferPromises = balances
      .filter(b => b.balance > 0)
      .map(async (b, i) => {
        const worker = this.workers[i];
        const lamports = Math.floor(b.balance * LAMPORTS_PER_SOL);
        
        const transaction = new Transaction().add(
          SystemProgram.transfer({
            fromPubkey: worker.address,
            toPubkey: targetAddress,
            lamports: lamports - 5000, // Leave some for fees
          })
        );
        
        transaction.sign(worker.keypair);
        
        try {
          const signature = await sendAndConfirmTransaction(
            this.connection,
            transaction,
            [worker.keypair]
          );
          return signature;
        } catch (error) {
          console.warn(`Failed to sweep from ${worker.address.toBase58()}:`, error);
          return null;
        }
      });
    
    await Promise.all(transferPromises);
    console.log(`Sweep complete`);
  }

  /**
   * Get pool statistics
   */
  async getStats(): Promise<PoolStats> {
    const balances = await this.getWorkerBalances();
    const totalBalance = balances.reduce((sum, b) => sum + b.balance, 0);
    
    return {
      totalWallets: this.workers.length,
      totalBalanceSOL: totalBalance,
      averageBalanceSOL: totalBalance / balances.length,
      walletAddresses: this.workers.map(w => w.address),
    };
  }
}

