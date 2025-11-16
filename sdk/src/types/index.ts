import { PublicKey } from '@solana/web3.js';

export enum QuestionType {
  General = 0,
  Price = 1,
  YesNo = 2,
  Numeric = 3,
}

export enum AnswerStatus {
  Pending = 0,
  Answered = 1,
  Disputed = 2,
  Finalized = 3,
}

export interface Question {
  questionId: string;
  requester: PublicKey;
  questionType: QuestionType;
  questionHash: Uint8Array;
  bounty: bigint;
  timestamp: number;
  deadline: number;
  status: AnswerStatus;
  refunded: boolean;
  questionText?: string;
}

export interface Answer {
  questionId: string;
  provider: PublicKey;
  confidenceScore: number;
  boolAnswer: boolean;
  numericAnswer: bigint;
  timestamp: number;
  textAnswer?: string;
  dataSource?: string;
}

export interface QuestionWithAnswer extends Question {
  answer?: Answer;
}

export interface Trace9Config {
  programId: PublicKey;
  rpcUrl: string;
  network: 'mainnet-beta' | 'devnet' | 'testnet';
}

export interface OracleState {
  authority: PublicKey;
  oracleProvider: PublicKey;
  questionCounter: bigint;
  oracleFee: bigint;
  providerBalance: bigint;
}

export interface AskQuestionParams {
  questionType: QuestionType;
  question: string;
  deadline: number; // Unix timestamp
  fee: bigint; // SOL in lamports
}

export interface ProvideAnswerParams {
  questionId: string;
  textAnswer: string;
  numericAnswer: bigint;
  boolAnswer: boolean;
  confidenceScore: number;
  dataSource: string;
}

