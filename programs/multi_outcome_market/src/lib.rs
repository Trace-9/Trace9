use anchor_lang::prelude::*;

declare_id!("multOutcM3mP9vK8JqF2nH5xY7wD4bC6eA8g");

#[program]
pub mod multi_outcome_market {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        oracle_program: Pubkey,
        fee_percentage: u16,
    ) -> Result<()> {
        let market_state = &mut ctx.accounts.market_state;
        market_state.authority = ctx.accounts.authority.key();
        market_state.oracle_program = oracle_program;
        market_state.market_counter = 0;
        market_state.fee_percentage = fee_percentage;
        market_state.accumulated_fees = 0;
        market_state.bump = ctx.bumps.market_state;
        Ok(())
    }

    pub fn create_market(
        ctx: Context<CreateMarket>,
        question: String,
        outcome_labels: Vec<String>,
        resolution_time: i64,
    ) -> Result<u64> {
        require!(
            question.len() > 0 && question.len() <= 500,
            MarketError::InvalidQuestion
        );
        require!(
            outcome_labels.len() >= 2 && outcome_labels.len() <= 10,
            MarketError::InvalidOutcomeCount
        );
        require!(
            resolution_time > Clock::get()?.unix_timestamp,
            MarketError::InvalidResolutionTime
        );

        let market_state = &mut ctx.accounts.market_state;
        let market_id = market_state.market_counter;
        let market_account = &mut ctx.accounts.market_account;

        market_account.market_id = market_id;
        market_account.question = question.clone();
        market_account.resolution_time = resolution_time;
        market_account.num_outcomes = outcome_labels.len() as u8;
        market_account.status = MarketStatus::Open;
        market_account.total_pool = 0;
        market_account.total_fees = 0;
        market_account.created_at = Clock::get()?.unix_timestamp;

        // Store outcome labels
        for (i, label) in outcome_labels.iter().enumerate() {
            require!(
                label.len() > 0 && label.len() <= 100,
                MarketError::InvalidOutcomeLabel
            );
            market_account.outcome_labels.push(label.clone());
            market_account.outcome_pools.push(0);
        }

        market_state.market_counter = market_id.checked_add(1).unwrap();

        emit!(MultiOutcomeMarketCreated {
            market_id,
            question,
            num_outcomes: outcome_labels.len() as u8,
            resolution_time,
        });

        Ok(market_id)
    }

    pub fn take_position(
        ctx: Context<TakePosition>,
        market_id: u64,
        outcome: u8,
    ) -> Result<()> {
        let market_account = &mut ctx.accounts.market_account;
        require!(
            market_account.status == MarketStatus::Open,
            MarketError::MarketNotOpen
        );
        require!(
            Clock::get()?.unix_timestamp < market_account.resolution_time,
            MarketError::MarketExpired
        );
        require!(outcome < market_account.num_outcomes, MarketError::InvalidOutcome);

        let bet_amount = ctx.accounts.bettor.lamports();
        require!(bet_amount > 0, MarketError::ZeroBet);

        let market_state = &ctx.accounts.market_state;
        let fee = bet_amount
            .checked_mul(market_state.fee_percentage as u64)
            .and_then(|x| x.checked_div(10000))
            .ok_or(MarketError::Overflow)?;
        let net_amount = bet_amount.checked_sub(fee).ok_or(MarketError::Overflow)?;

        market_account.outcome_pools[outcome as usize] = market_account.outcome_pools
            [outcome as usize]
            .checked_add(net_amount)
            .ok_or(MarketError::Overflow)?;
        market_account.total_pool = market_account
            .total_pool
            .checked_add(net_amount)
            .ok_or(MarketError::Overflow)?;
        market_account.total_fees = market_account
            .total_fees
            .checked_add(fee)
            .ok_or(MarketError::Overflow)?;

        let position = &mut ctx.accounts.position;
        if position.amounts.len() <= outcome as usize {
            position.amounts.resize((outcome + 1) as usize, 0);
        }
        position.amounts[outcome as usize] = position.amounts[outcome as usize]
            .checked_add(net_amount)
            .ok_or(MarketError::Overflow)?;

        **ctx.accounts.bettor.to_account_info().try_borrow_mut_lamports()? -= bet_amount;
        **ctx.accounts.market_account.to_account_info().try_borrow_mut_lamports()? += bet_amount;

        emit!(OutcomePositionTaken {
            market_id,
            user: ctx.accounts.bettor.key(),
            outcome,
            amount: net_amount,
        });

        Ok(())
    }

    pub fn resolve_market(ctx: Context<ResolveMarket>, market_id: u64) -> Result<()> {
        let market_account = &mut ctx.accounts.market_account;
        require!(
            market_account.status == MarketStatus::Open,
            MarketError::MarketNotOpen
        );
        require!(
            Clock::get()?.unix_timestamp >= market_account.resolution_time,
            MarketError::TooEarly
        );

        // Oracle provides numeric answer as winning outcome index
        let winning_outcome = ctx.accounts.oracle_answer.numeric_answer as u8;
        require!(
            winning_outcome < market_account.num_outcomes,
            MarketError::InvalidOutcome
        );
        require!(
            ctx.accounts.oracle_answer.confidence_score > 0,
            MarketError::OracleNotAnswered
        );

        market_account.status = MarketStatus::Resolved;
        market_account.winning_outcome = winning_outcome;

        let market_state = &mut ctx.accounts.market_state;
        market_state.accumulated_fees = market_state
            .accumulated_fees
            .checked_add(market_account.total_fees)
            .ok_or(MarketError::Overflow)?;

        emit!(MultiOutcomeMarketResolved {
            market_id,
            winning_outcome,
        });

        Ok(())
    }

    pub fn claim_winnings(ctx: Context<ClaimWinnings>, market_id: u64) -> Result<()> {
        let market_account = &ctx.accounts.market_account;
        require!(
            market_account.status == MarketStatus::Resolved,
            MarketError::NotResolved
        );

        let position = &mut ctx.accounts.position;
        require!(!position.claimed, MarketError::AlreadyClaimed);

        let winning_amount = if position.amounts.len() > market_account.winning_outcome as usize {
            position.amounts[market_account.winning_outcome as usize]
        } else {
            0
        };

        require!(winning_amount > 0, MarketError::NoWinnings);

        let winning_pool = market_account.outcome_pools[market_account.winning_outcome as usize];
        require!(winning_pool > 0, MarketError::NoWinnings);

        let payout = winning_amount
            .checked_mul(market_account.total_pool)
            .and_then(|x| x.checked_div(winning_pool))
            .ok_or(MarketError::Overflow)?;

        position.claimed = true;

        **ctx.accounts.market_account.to_account_info().try_borrow_mut_lamports()? -= payout;
        **ctx.accounts.winner.to_account_info().try_borrow_mut_lamports()? += payout;

        emit!(MultiOutcomeWinningsClaimed {
            market_id,
            user: ctx.accounts.winner.key(),
            amount: payout,
        });

        Ok(())
    }

    pub fn withdraw_fees(ctx: Context<WithdrawFees>) -> Result<()> {
        let market_state = &mut ctx.accounts.market_state;
        require!(
            ctx.accounts.authority.key() == market_state.authority,
            MarketError::Unauthorized
        );

        let amount = market_state.accumulated_fees;
        require!(amount > 0, MarketError::NoFees);

        market_state.accumulated_fees = 0;

        **ctx.accounts.market_state.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.authority.to_account_info().try_borrow_mut_lamports()? += amount;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + MarketState::LEN,
        seeds = [b"market_state"],
        bump
    )]
    pub market_state: Account<'info, MarketState>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateMarket<'info> {
    #[account(
        init,
        payer = creator,
        space = 8 + MarketAccount::LEN,
        seeds = [b"market", market_state.market_counter.to_le_bytes().as_ref()],
        bump
    )]
    pub market_account: Account<'info, MarketAccount>,
    #[account(mut, seeds = [b"market_state"], bump = market_state.bump)]
    pub market_state: Account<'info, MarketState>,
    #[account(mut)]
    pub creator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(market_id: u64)]
pub struct TakePosition<'info> {
    #[account(mut, seeds = [b"market", market_id.to_le_bytes().as_ref()], bump)]
    pub market_account: Account<'info, MarketAccount>,
    #[account(
        init_if_needed,
        payer = bettor,
        space = 8 + Position::LEN,
        seeds = [b"position", market_id.to_le_bytes().as_ref(), bettor.key().as_ref()],
        bump
    )]
    pub position: Account<'info, Position>,
    #[account(seeds = [b"market_state"], bump = market_state.bump)]
    pub market_state: Account<'info, MarketState>,
    #[account(mut)]
    pub bettor: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(market_id: u64)]
pub struct ResolveMarket<'info> {
    #[account(mut, seeds = [b"market", market_id.to_le_bytes().as_ref()], bump)]
    pub market_account: Account<'info, MarketAccount>,
    #[account(mut, seeds = [b"market_state"], bump = market_state.bump)]
    pub market_state: Account<'info, MarketState>,
    /// CHECK: Oracle answer from trace9 program
    pub oracle_answer: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(market_id: u64)]
pub struct ClaimWinnings<'info> {
    #[account(mut, seeds = [b"market", market_id.to_le_bytes().as_ref()], bump)]
    pub market_account: Account<'info, MarketAccount>,
    #[account(
        mut,
        seeds = [b"position", market_id.to_le_bytes().as_ref(), winner.key().as_ref()],
        bump
    )]
    pub position: Account<'info, Position>,
    #[account(mut)]
    pub winner: Signer<'info>,
}

#[derive(Accounts)]
pub struct WithdrawFees<'info> {
    #[account(mut, seeds = [b"market_state"], bump = market_state.bump)]
    pub market_state: Account<'info, MarketState>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

#[account]
pub struct MarketState {
    pub authority: Pubkey,
    pub oracle_program: Pubkey,
    pub market_counter: u64,
    pub fee_percentage: u16,
    pub accumulated_fees: u64,
    pub bump: u8,
}

impl MarketState {
    pub const LEN: usize = 32 + 32 + 8 + 2 + 8 + 1;
}

#[account]
pub struct MarketAccount {
    pub market_id: u64,
    pub question: String,
    pub resolution_time: i64,
    pub num_outcomes: u8,
    pub outcome_labels: Vec<String>,
    pub outcome_pools: Vec<u64>,
    pub status: MarketStatus,
    pub winning_outcome: u8,
    pub total_pool: u64,
    pub total_fees: u64,
    pub created_at: i64,
}

impl MarketAccount {
    pub const LEN: usize = 8 + (4 + 500) + 8 + 1 + (4 + 10 * (4 + 100)) + (4 + 10 * 8) + 1 + 1 + 8 + 8 + 8;
}

#[account]
pub struct Position {
    pub amounts: Vec<u64>,
    pub claimed: bool,
}

impl Position {
    pub const LEN: usize = 4 + (10 * 8) + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum MarketStatus {
    Open,
    Closed,
    Resolved,
    Canceled,
}

#[event]
pub struct MultiOutcomeMarketCreated {
    pub market_id: u64,
    pub question: String,
    pub num_outcomes: u8,
    pub resolution_time: i64,
}

#[event]
pub struct OutcomePositionTaken {
    pub market_id: u64,
    pub user: Pubkey,
    pub outcome: u8,
    pub amount: u64,
}

#[event]
pub struct MultiOutcomeMarketResolved {
    pub market_id: u64,
    pub winning_outcome: u8,
}

#[event]
pub struct MultiOutcomeWinningsClaimed {
    pub market_id: u64,
    pub user: Pubkey,
    pub amount: u64,
}

#[error_code]
pub enum MarketError {
    #[msg("Invalid question")]
    InvalidQuestion,
    #[msg("Invalid outcome count")]
    InvalidOutcomeCount,
    #[msg("Invalid resolution time")]
    InvalidResolutionTime,
    #[msg("Invalid outcome label")]
    InvalidOutcomeLabel,
    #[msg("Market not open")]
    MarketNotOpen,
    #[msg("Market expired")]
    MarketExpired,
    #[msg("Invalid outcome")]
    InvalidOutcome,
    #[msg("Zero bet")]
    ZeroBet,
    #[msg("Overflow")]
    Overflow,
    #[msg("Too early")]
    TooEarly,
    #[msg("Oracle not answered")]
    OracleNotAnswered,
    #[msg("Not resolved")]
    NotResolved,
    #[msg("Already claimed")]
    AlreadyClaimed,
    #[msg("No winnings")]
    NoWinnings,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("No fees")]
    NoFees,
}

