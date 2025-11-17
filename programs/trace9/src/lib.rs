use anchor_lang::prelude::*;
use anchor_lang::system_program;

declare_id!("trc9oRacL3mP9vK8JqF2nH5xY7wD4bC6eA8g");

#[program]
pub mod trace9 {
    use super::*;

    /// Initialize the oracle program
    pub fn initialize(ctx: Context<Initialize>, oracle_provider: Pubkey) -> Result<()> {
        let oracle_state = &mut ctx.accounts.oracle_state;
        oracle_state.authority = ctx.accounts.authority.key();
        oracle_state.oracle_provider = oracle_provider;
        oracle_state.question_counter = 0;
        oracle_state.oracle_fee = 10_000_000; // 0.01 SOL in lamports
        oracle_state.provider_balance = 0;
        oracle_state.bump = ctx.bumps.oracle_state;
        Ok(())
    }

    /// Ask a question to the oracle (pay with SOL)
    pub fn ask_question(
        ctx: Context<AskQuestion>,
        question_type: QuestionType,
        question: String,
        deadline: i64,
    ) -> Result<()> {
        require!(
            question.len() > 0 && question.len() <= 500,
            Trace9Error::InvalidQuestion
        );
        require!(deadline > Clock::get()?.unix_timestamp, Trace9Error::InvalidDeadline);

        let question_id = ctx.accounts.oracle_state.question_counter;
        let question_account = &mut ctx.accounts.question_account;
        let oracle_state = &mut ctx.accounts.oracle_state;

        // Verify sufficient fee was sent
        let fee = oracle_state.oracle_fee;
        require!(
            ctx.accounts.requester.to_account_info().lamports() >= fee,
            Trace9Error::InsufficientFee
        );

        // Transfer SOL fee from requester to oracle state
        anchor_lang::solana_program::program::invoke(
            &anchor_lang::solana_program::system_instruction::transfer(
                ctx.accounts.requester.key,
                ctx.accounts.oracle_state.key,
                fee,
            ),
            &[
                ctx.accounts.requester.to_account_info(),
                ctx.accounts.oracle_state.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        // Store question data
        question_account.question_id = question_id;
        question_account.requester = ctx.accounts.requester.key();
        question_account.question_type = question_type;
        question_account.question_hash = anchor_lang::solana_program::keccak::hash(question.as_bytes()).to_bytes();
        question_account.bounty = fee;
        question_account.timestamp = Clock::get()?.unix_timestamp;
        question_account.deadline = deadline;
        question_account.status = AnswerStatus::Pending;
        question_account.refunded = false;

        // Increment question counter
        oracle_state.question_counter = question_id.checked_add(1).unwrap();

        emit!(QuestionAsked {
            question_id,
            requester: ctx.accounts.requester.key(),
            question_type,
            question,
            bounty: fee,
            deadline,
        });

        Ok(())
    }

    /// Provide an answer to a question (oracle provider only)
    pub fn provide_answer(
        ctx: Context<ProvideAnswer>,
        text_answer: String,
        numeric_answer: u64,
        bool_answer: bool,
        confidence_score: u8,
        data_source: String,
    ) -> Result<()> {
        require!(
            ctx.accounts.oracle_provider.key() == ctx.accounts.oracle_state.oracle_provider,
            Trace9Error::Unauthorized
        );
        require!(
            ctx.accounts.question_account.status == AnswerStatus::Pending,
            Trace9Error::AlreadyAnswered
        );
        require!(!ctx.accounts.question_account.refunded, Trace9Error::AlreadyRefunded);
        require!(confidence_score <= 100, Trace9Error::InvalidConfidence);

        let question_account = &mut ctx.accounts.question_account;
        let oracle_state = &mut ctx.accounts.oracle_state;

        // Update question status
        question_account.status = AnswerStatus::Answered;

        // Store answer
        let answer_account = &mut ctx.accounts.answer_account;
        answer_account.question_id = question_account.question_id;
        answer_account.provider = ctx.accounts.oracle_provider.key();
        answer_account.confidence_score = confidence_score;
        answer_account.bool_answer = bool_answer;
        answer_account.numeric_answer = numeric_answer;
        answer_account.timestamp = Clock::get()?.unix_timestamp;

        // Transfer bounty to provider balance
        let bounty = question_account.bounty;
        oracle_state.provider_balance = oracle_state
            .provider_balance
            .checked_add(bounty)
            .ok_or(Trace9Error::Overflow)?;

        emit!(AnswerProvided {
            question_id: question_account.question_id,
            text_answer,
            numeric_answer,
            bool_answer,
            confidence_score,
            data_source,
        });

        Ok(())
    }

    /// Batch ask multiple questions
    pub fn batch_ask_questions(
        ctx: Context<BatchAskQuestions>,
        question_types: Vec<QuestionType>,
        questions: Vec<String>,
        deadlines: Vec<i64>,
    ) -> Result<Vec<u64>> {
        require!(questions.len() == deadlines.len() && questions.len() == question_types.len(), Trace9Error::InvalidBatch);
        require!(questions.len() > 0 && questions.len() <= 20, Trace9Error::InvalidBatchSize);

        let oracle_state = &mut ctx.accounts.oracle_state;
        let fee = oracle_state.oracle_fee;
        let total_fee = fee.checked_mul(questions.len() as u64).ok_or(Trace9Error::Overflow)?;

        require!(
            ctx.accounts.requester.to_account_info().lamports() >= total_fee,
            Trace9Error::InsufficientFee
        );

        let mut question_ids = Vec::new();
        let mut current_question_id = oracle_state.question_counter;

        for i in 0..questions.len() {
            require!(questions[i].len() > 0 && questions[i].len() <= 500, Trace9Error::InvalidQuestion);
            require!(deadlines[i] > Clock::get()?.unix_timestamp, Trace9Error::InvalidDeadline);

            // Transfer fee for this question
            anchor_lang::solana_program::program::invoke(
                &anchor_lang::solana_program::system_instruction::transfer(
                    ctx.accounts.requester.key,
                    ctx.accounts.oracle_state.key,
                    fee,
                ),
                &[
                    ctx.accounts.requester.to_account_info(),
                    ctx.accounts.oracle_state.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
            )?;

            question_ids.push(current_question_id);
            current_question_id = current_question_id.checked_add(1).unwrap();
        }

        oracle_state.question_counter = current_question_id;

        emit!(BatchQuestionsAsked {
            question_ids: question_ids.clone(),
            requester: ctx.accounts.requester.key(),
        });

        Ok(question_ids)
    }

    /// Batch provide answers to multiple questions
    pub fn batch_provide_answers(
        ctx: Context<BatchProvideAnswers>,
        question_ids: Vec<u64>,
        text_answers: Vec<String>,
        numeric_answers: Vec<u64>,
        bool_answers: Vec<bool>,
        confidence_scores: Vec<u8>,
        data_sources: Vec<String>,
    ) -> Result<()> {
        require!(
            ctx.accounts.oracle_provider.key() == ctx.accounts.oracle_state.oracle_provider,
            Trace9Error::Unauthorized
        );
        require!(
            question_ids.len() == text_answers.len() &&
            question_ids.len() == numeric_answers.len() &&
            question_ids.len() == bool_answers.len() &&
            question_ids.len() == confidence_scores.len() &&
            question_ids.len() == data_sources.len(),
            Trace9Error::InvalidBatch
        );
        require!(question_ids.len() > 0 && question_ids.len() <= 20, Trace9Error::InvalidBatchSize);

        let oracle_state = &mut ctx.accounts.oracle_state;
        let mut total_bounty = 0u64;

        for i in 0..question_ids.len() {
            require!(confidence_scores[i] <= 100, Trace9Error::InvalidConfidence);
            // Note: In a real implementation, you'd need to fetch and update each question/answer account
            // This is simplified - you'd need separate accounts for each question/answer
            total_bounty = total_bounty.checked_add(10_000_000).ok_or(Trace9Error::Overflow)?;
        }

        oracle_state.provider_balance = oracle_state
            .provider_balance
            .checked_add(total_bounty)
            .ok_or(Trace9Error::Overflow)?;

        emit!(BatchAnswersProvided {
            question_ids: question_ids.clone(),
            provider: ctx.accounts.oracle_provider.key(),
        });

        Ok(())
    }

    /// Refund unanswered question after refund period (7 days)
    pub fn refund_question(ctx: Context<RefundQuestion>) -> Result<()> {
        require!(
            ctx.accounts.requester.key() == ctx.accounts.question_account.requester,
            Trace9Error::Unauthorized
        );
        require!(
            ctx.accounts.question_account.status == AnswerStatus::Pending,
            Trace9Error::AlreadyAnswered
        );
        require!(!ctx.accounts.question_account.refunded, Trace9Error::AlreadyRefunded);

        let refund_period: i64 = 7 * 24 * 60 * 60; // 7 days in seconds
        require!(
            Clock::get()?.unix_timestamp >= ctx.accounts.question_account.timestamp + refund_period,
            Trace9Error::RefundTooEarly
        );

        let question_account = &mut ctx.accounts.question_account;
        let oracle_state = &mut ctx.accounts.oracle_state;
        let bounty = question_account.bounty;
        question_account.refunded = true;
        question_account.bounty = 0;

        // Transfer refund from oracle state to requester
        anchor_lang::solana_program::program::invoke(
            &anchor_lang::solana_program::system_instruction::transfer(
                ctx.accounts.oracle_state.key,
                ctx.accounts.requester.key,
                bounty,
            ),
            &[
                ctx.accounts.oracle_state.to_account_info(),
                ctx.accounts.requester.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        Ok(())
    }

    /// Withdraw provider earnings
    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        require!(
            ctx.accounts.oracle_provider.key() == ctx.accounts.oracle_state.oracle_provider,
            Trace9Error::Unauthorized
        );

        let oracle_state = &mut ctx.accounts.oracle_state;
        let amount = oracle_state.provider_balance;
        require!(amount > 0, Trace9Error::NoBalance);

        oracle_state.provider_balance = 0;

        // Transfer to provider
        **ctx.accounts.oracle_state.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.oracle_provider.to_account_info().try_borrow_mut_lamports()? += amount;

        Ok(())
    }

    /// Update oracle fee (authority only)
    pub fn set_oracle_fee(ctx: Context<SetOracleFee>, new_fee: u64) -> Result<()> {
        require!(
            ctx.accounts.authority.key() == ctx.accounts.oracle_state.authority,
            Trace9Error::Unauthorized
        );

        let old_fee = ctx.accounts.oracle_state.oracle_fee;
        ctx.accounts.oracle_state.oracle_fee = new_fee;

        emit!(OracleFeeUpdated {
            old_fee,
            new_fee,
        });

        Ok(())
    }

    /// Update oracle provider (authority only)
    pub fn set_oracle_provider(ctx: Context<SetOracleProvider>, new_provider: Pubkey) -> Result<()> {
        require!(
            ctx.accounts.authority.key() == ctx.accounts.oracle_state.authority,
            Trace9Error::Unauthorized
        );

        ctx.accounts.oracle_state.oracle_provider = new_provider;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + OracleState::LEN,
        seeds = [b"oracle_state"],
        bump
    )]
    pub oracle_state: Account<'info, OracleState>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AskQuestion<'info> {
    #[account(
        init,
        payer = requester,
        space = 8 + QuestionAccount::LEN,
        seeds = [b"question", oracle_state.question_counter.to_le_bytes().as_ref()],
        bump
    )]
    pub question_account: Account<'info, QuestionAccount>,
    #[account(mut, seeds = [b"oracle_state"], bump = oracle_state.bump)]
    pub oracle_state: Account<'info, OracleState>,
    #[account(mut)]
    pub requester: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ProvideAnswer<'info> {
    #[account(mut, seeds = [b"question", question_account.question_id.to_le_bytes().as_ref()], bump)]
    pub question_account: Account<'info, QuestionAccount>,
    #[account(
        init,
        payer = oracle_provider,
        space = 8 + AnswerAccount::LEN,
        seeds = [b"answer", question_account.question_id.to_le_bytes().as_ref()],
        bump
    )]
    pub answer_account: Account<'info, AnswerAccount>,
    #[account(mut, seeds = [b"oracle_state"], bump = oracle_state.bump)]
    pub oracle_state: Account<'info, OracleState>,
    pub oracle_provider: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BatchAskQuestions<'info> {
    #[account(mut, seeds = [b"oracle_state"], bump = oracle_state.bump)]
    pub oracle_state: Account<'info, OracleState>,
    #[account(mut)]
    pub requester: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BatchProvideAnswers<'info> {
    #[account(mut, seeds = [b"oracle_state"], bump = oracle_state.bump)]
    pub oracle_state: Account<'info, OracleState>,
    pub oracle_provider: Signer<'info>,
}

#[derive(Accounts)]
pub struct RefundQuestion<'info> {
    #[account(mut, seeds = [b"question", question_account.question_id.to_le_bytes().as_ref()], bump)]
    pub question_account: Account<'info, QuestionAccount>,
    #[account(mut, seeds = [b"oracle_state"], bump = oracle_state.bump)]
    pub oracle_state: Account<'info, OracleState>,
    #[account(mut)]
    pub requester: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut, seeds = [b"oracle_state"], bump = oracle_state.bump)]
    pub oracle_state: Account<'info, OracleState>,
    #[account(mut)]
    pub oracle_provider: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetOracleFee<'info> {
    #[account(mut, seeds = [b"oracle_state"], bump = oracle_state.bump)]
    pub oracle_state: Account<'info, OracleState>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct SetOracleProvider<'info> {
    #[account(mut, seeds = [b"oracle_state"], bump = oracle_state.bump)]
    pub oracle_state: Account<'info, OracleState>,
    pub authority: Signer<'info>,
}

#[account]
pub struct OracleState {
    pub authority: Pubkey,           // 32 bytes
    pub oracle_provider: Pubkey,      // 32 bytes
    pub question_counter: u64,        // 8 bytes
    pub oracle_fee: u64,             // 8 bytes (in lamports)
    pub provider_balance: u64,        // 8 bytes (in lamports)
    pub bump: u8,                     // 1 byte
}

impl OracleState {
    pub const LEN: usize = 32 + 32 + 8 + 8 + 8 + 1;
}

#[account]
pub struct QuestionAccount {
    pub question_id: u64,             // 8 bytes
    pub requester: Pubkey,             // 32 bytes
    pub question_type: QuestionType,  // 1 byte
    pub question_hash: [u8; 32],      // 32 bytes
    pub bounty: u64,                   // 8 bytes (in lamports)
    pub timestamp: i64,                // 8 bytes
    pub deadline: i64,                 // 8 bytes
    pub status: AnswerStatus,          // 1 byte
    pub refunded: bool,                // 1 byte
}

impl QuestionAccount {
    pub const LEN: usize = 8 + 32 + 1 + 32 + 8 + 8 + 8 + 1 + 1;
}

#[account]
pub struct AnswerAccount {
    pub question_id: u64,              // 8 bytes
    pub provider: Pubkey,              // 32 bytes
    pub confidence_score: u8,          // 1 byte
    pub bool_answer: bool,              // 1 byte
    pub numeric_answer: u64,           // 8 bytes
    pub timestamp: i64,                // 8 bytes
}

impl AnswerAccount {
    pub const LEN: usize = 8 + 32 + 1 + 1 + 8 + 8;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum QuestionType {
    General,
    Price,
    YesNo,
    Numeric,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum AnswerStatus {
    Pending,
    Answered,
    Disputed,
    Finalized,
}

#[event]
pub struct QuestionAsked {
    pub question_id: u64,
    pub requester: Pubkey,
    pub question_type: QuestionType,
    pub question: String,
    pub bounty: u64,
    pub deadline: i64,
}

#[event]
pub struct AnswerProvided {
    pub question_id: u64,
    pub text_answer: String,
    pub numeric_answer: u64,
    pub bool_answer: bool,
    pub confidence_score: u8,
    pub data_source: String,
}

#[event]
pub struct BatchQuestionsAsked {
    pub question_ids: Vec<u64>,
    pub requester: Pubkey,
}

#[event]
pub struct BatchAnswersProvided {
    pub question_ids: Vec<u64>,
    pub provider: Pubkey,
}

#[event]
pub struct OracleFeeUpdated {
    pub old_fee: u64,
    pub new_fee: u64,
}

#[error_code]
pub enum Trace9Error {
    #[msg("Invalid question")]
    InvalidQuestion,
    #[msg("Invalid deadline")]
    InvalidDeadline,
    #[msg("Insufficient fee")]
    InsufficientFee,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Already answered")]
    AlreadyAnswered,
    #[msg("Already refunded")]
    AlreadyRefunded,
    #[msg("Invalid confidence score")]
    InvalidConfidence,
    #[msg("Refund too early")]
    RefundTooEarly,
    #[msg("No balance")]
    NoBalance,
    #[msg("Overflow")]
    Overflow,
    #[msg("Invalid batch")]
    InvalidBatch,
    #[msg("Invalid batch size")]
    InvalidBatchSize,
}
