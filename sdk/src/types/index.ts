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

// Prediction Market Types

export enum MarketStatus {
  Open = 0,
  Closed = 1,
  Resolved = 2,
  Canceled = 3,
}

export enum Outcome {
  Unresolved = 0,
  Yes = 1,
  No = 2,
}

export interface SimpleMarket {
  marketId: bigint;
  question: string;
  resolutionTime: number;
  yesPool: bigint;
  noPool: bigint;
  status: MarketStatus;
  outcome: Outcome;
  totalFees: bigint;
  createdAt: number;
  creator: PublicKey;
}

export interface SimplePosition {
  yesAmount: bigint;
  noAmount: bigint;
  claimed: boolean;
}

export interface MultiOutcomeMarket {
  marketId: bigint;
  question: string;
  resolutionTime: number;
  numOutcomes: number;
  outcomeLabels: string[];
  outcomePools: bigint[];
  status: MarketStatus;
  winningOutcome: number;
  totalPool: bigint;
  totalFees: bigint;
  createdAt: number;
}

export interface MultiOutcomePosition {
  amounts: bigint[];
  claimed: boolean;
}

export interface RangeMarket {
  marketId: bigint;
  question: string;
  lowerBound: bigint;
  upperBound: bigint;
  inRangePool: bigint;
  outRangePool: bigint;
  totalFees: bigint;
  createdAt: number;
  deadline: number;
  resolvedAt: number;
  resolved: boolean;
  inRange: boolean;
}

export interface RangePosition {
  inRangeAmount: bigint;
  outRangeAmount: bigint;
  claimed: boolean;
}

export interface TimeSeriesMarket {
  marketId: bigint;
  question: string;
  periods: TimePeriod[];
  successPool: bigint;
  failurePool: bigint;
  totalFees: bigint;
  createdAt: number;
  allResolved: boolean;
  allSuccess: boolean;
}

export interface TimePeriod {
  deadline: number;
  questionId: bigint;
  result: bigint;
  resolved: boolean;
}

export interface TimeSeriesPosition {
  successAmount: bigint;
  failureAmount: bigint;
  claimed: boolean;
}

export interface ConditionalMarket {
  marketId: bigint;
  question: string;
  parentMarket: PublicKey;
  requiredParentOutcome: number;
  yesPool: bigint;
  noPool: bigint;
  totalFees: bigint;
  createdAt: number;
  resolvedAt: number;
  status: ConditionalMarketStatus;
  finalOutcome: boolean;
}

export enum ConditionalMarketStatus {
  Active = 0,
  ParentUnresolved = 1,
  ConditionNotMet = 2,
  Resolved = 3,
  Cancelled = 4,
}

export interface ConditionalPosition {
  yesAmount: bigint;
  noAmount: bigint;
  claimed: boolean;
}

export interface CreateSimpleMarketParams {
  question: string;
  resolutionTime: number; // Unix timestamp
}

export interface CreateMultiOutcomeMarketParams {
  question: string;
  outcomeLabels: string[];
  resolutionTime: number;
}

export interface CreateRangeMarketParams {
  question: string;
  lowerBound: bigint;
  upperBound: bigint;
  deadline: number;
}

export interface CreateTimeSeriesMarketParams {
  question: string;
  deadlines: number[]; // Array of Unix timestamps
}

export interface CreateConditionalMarketParams {
  question: string;
  parentMarket: PublicKey;
  requiredParentOutcome: number; // 0 = NO, 1 = YES
}

