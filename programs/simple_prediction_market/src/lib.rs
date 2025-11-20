use anchor_lang::prelude::*;
use anchor_lang::system_program;

declare_id!("simpPredM3mP9vK8JqF2nH5xY7wD4bC6eA8g");

#[program]
pub mod simple_prediction_market {
    use super::*;

    /// Initialize the prediction market program
    pub fn initialize(
        ctx: Context<Initialize>,
        oracle_program: Pubkey,
        fee_percentage: u16, // Basis points (e.g., 200 = 2%)
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

    /// Create a new binary prediction market
    pub fn create_market(
        ctx: Context<CreateMarket>,
        question: String,
        resolution_time: i64,
    ) -> Result<u64> {
        require!(
            question.len() > 0 && question.len() <= 500,
            MarketError::InvalidQuestion
        );
        require!(
            resolution_time > Clock::get()?.unix_timestamp,
            MarketError::InvalidResolutionTime
        );

        let market_state = &mut ctx.accounts.market_state;
        let market_id = market_state.market_counter;
        let market_account = &mut ctx.accounts.market_account;

        // Store market data
        market_account.market_id = market_id;
        market_account.question = question.clone();
        market_account.resolution_time = resolution_time;
        market_account.yes_pool = 0;
        market_account.no_pool = 0;
        market_account.status = MarketStatus::Open;
        market_account.outcome = Outcome::Unresolved;
        market_account.total_fees = 0;
        market_account.created_at = Clock::get()?.unix_timestamp;
        market_account.creator = ctx.accounts.creator.key();

        // Increment market counter
        market_state.market_counter = market_id.checked_add(1).unwrap();

        emit!(MarketCreated {
            market_id,
            question,
            resolution_time,
            creator: ctx.accounts.creator.key(),
        });

        Ok(market_id)
    }

    /// Take a position on a market (YES or NO)
    pub fn take_position(
        ctx: Context<TakePosition>,
        market_id: u64,
        is_yes: bool,
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

        let market_state = &ctx.accounts.market_state;
        let fee_percentage = market_state.fee_percentage;
        let bet_amount = ctx.accounts.bettor.lamports();

        require!(bet_amount > 0, MarketError::ZeroBet);

        // Calculate fee (in basis points)
        let fee = bet_amount
            .checked_mul(fee_percentage as u64)
            .and_then(|x| x.checked_div(10000))
            .ok_or(MarketError::Overflow)?;
        let net_amount = bet_amount.checked_sub(fee).ok_or(MarketError::Overflow)?;

        // Update market pools
        market_account.total_fees = market_account
            .total_fees
            .checked_add(fee)
            .ok_or(MarketError::Overflow)?;

        if is_yes {
            market_account.yes_pool = market_account
                .yes_pool
                .checked_add(net_amount)
                .ok_or(MarketError::Overflow)?;
        } else {
            market_account.no_pool = market_account
                .no_pool
                .checked_add(net_amount)
                .ok_or(MarketError::Overflow)?;
        }

        // Update position
        let position = &mut ctx.accounts.position;
        if is_yes {
            position.yes_amount = position
                .yes_amount
                .checked_add(net_amount)
                .ok_or(MarketError::Overflow)?;
        } else {
            position.no_amount = position
                .no_amount
                .checked_add(net_amount)
                .ok_or(MarketError::Overflow)?;
        }

        // Transfer SOL from bettor to market account
        **ctx.accounts.bettor.to_account_info().try_borrow_mut_lamports()? -= bet_amount;
        **ctx.accounts.market_account.to_account_info().try_borrow_mut_lamports()? += bet_amount;

        emit!(PositionTaken {
            market_id,
            user: ctx.accounts.bettor.key(),
            is_yes,
            amount: net_amount,
        });

        Ok(())
    }

    /// Resolve market using oracle answer
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

        // Read oracle answer from the oracle program
        // Note: In a real implementation, you'd use CPI to call the oracle program
        // For now, we'll assume the answer is passed via context or stored
        // This is a simplified version - you'd need to fetch from oracle program
        
        // For now, we'll require the oracle answer to be provided
        // In production, this would be fetched via CPI from trace9 program
        let bool_answer = ctx.accounts.oracle_answer.bool_answer;
        require!(
            ctx.accounts.oracle_answer.confidence_score > 0,
            MarketError::OracleNotAnswered
        );

        market_account.outcome = if bool_answer {
            Outcome::Yes
        } else {
            Outcome::No
        };
        market_account.status = MarketStatus::Resolved;

        // Move fees to accumulated fees
        let market_state = &mut ctx.accounts.market_state;
        market_state.accumulated_fees = market_state
            .accumulated_fees
            .checked_add(market_account.total_fees)
            .ok_or(MarketError::Overflow)?;

        emit!(MarketResolved {
            market_id,
            outcome: market_account.outcome,
        });

        Ok(())
    }

    /// Claim winnings from a resolved market
    pub fn claim_winnings(ctx: Context<ClaimWinnings>, market_id: u64) -> Result<()> {
        let market_account = &ctx.accounts.market_account;
        require!(
            market_account.status == MarketStatus::Resolved,
            MarketError::NotResolved
        );

        let position = &mut ctx.accounts.position;
        require!(!position.claimed, MarketError::AlreadyClaimed);

        let total_pool = market_account
            .yes_pool
            .checked_add(market_account.no_pool)
            .ok_or(MarketError::Overflow)?;

        let winnings = if market_account.outcome == Outcome::Yes && position.yes_amount > 0 {
            if market_account.yes_pool == 0 {
                return Err(MarketError::NoWinnings.into());
            }
            position
                .yes_amount
                .checked_mul(total_pool)
                .and_then(|x| x.checked_div(market_account.yes_pool))
                .ok_or(MarketError::Overflow)?
        } else if market_account.outcome == Outcome::No && position.no_amount > 0 {
            if market_account.no_pool == 0 {
                return Err(MarketError::NoWinnings.into());
            }
            position
                .no_amount
                .checked_mul(total_pool)
                .and_then(|x| x.checked_div(market_account.no_pool))
                .ok_or(MarketError::Overflow)?
        } else {
            return Err(MarketError::NoWinnings.into());
        };

        require!(winnings > 0, MarketError::NoWinnings);

        position.claimed = true;

        // Transfer winnings
        **ctx.accounts.market_account.to_account_info().try_borrow_mut_lamports()? -= winnings;
        **ctx.accounts.winner.to_account_info().try_borrow_mut_lamports()? += winnings;

        emit!(WinningsClaimed {
            market_id,
            user: ctx.accounts.winner.key(),
            amount: winnings,
        });

        Ok(())
    }

    /// Cancel market if oracle hasn't answered (after 7 days)
    pub fn cancel_market(ctx: Context<CancelMarket>, market_id: u64) -> Result<()> {
        let market_account = &mut ctx.accounts.market_account;
        require!(
            market_account.status == MarketStatus::Open,
            MarketError::MarketNotOpen
        );

        let refund_period: i64 = 7 * 24 * 60 * 60; // 7 days
        require!(
            Clock::get()?.unix_timestamp
                >= market_account.resolution_time + refund_period,
            MarketError::TooEarlyToCancel
        );

        // Check oracle hasn't answered (simplified - would check oracle program in production)
        require!(
            ctx.accounts.oracle_answer.confidence_score == 0,
            MarketError::AlreadyAnswered
        );

        market_account.status = MarketStatus::Canceled;

        emit!(MarketCanceled { market_id });

        Ok(())
    }

    /// Claim refund from canceled market
    pub fn claim_refund(ctx: Context<ClaimRefund>, market_id: u64) -> Result<()> {
        let market_account = &mut ctx.accounts.market_account;
        require!(
            market_account.status == MarketStatus::Canceled,
            MarketError::NotCanceled
        );

        let position = &mut ctx.accounts.position;
        require!(!position.claimed, MarketError::AlreadyClaimed);

        let refund_amount = position
            .yes_amount
            .checked_add(position.no_amount)
            .ok_or(MarketError::Overflow)?;

        require!(refund_amount > 0, MarketError::NoPosition);

        position.claimed = true;

        // Transfer refund
        **ctx.accounts.market_account.to_account_info().try_borrow_mut_lamports()? -= refund_amount;
        **ctx.accounts.refundee.to_account_info().try_borrow_mut_lamports()? += refund_amount;

        Ok(())
    }

    /// Withdraw accumulated fees (authority only)
    pub fn withdraw_fees(ctx: Context<WithdrawFees>) -> Result<()> {
        let market_state = &mut ctx.accounts.market_state;
        require!(
            ctx.accounts.authority.key() == market_state.authority,
            MarketError::Unauthorized
        );

        let amount = market_state.accumulated_fees;
        require!(amount > 0, MarketError::NoFees);

        market_state.accumulated_fees = 0;

        // Transfer fees
        **ctx.accounts.market_state.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.authority.to_account_info().try_borrow_mut_lamports()? += amount;

        emit!(FeesWithdrawn {
            amount,
            authority: ctx.accounts.authority.key(),
        });

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
    /// Oracle answer account (from trace9 program)
    /// CHECK: This should be verified to come from the oracle program
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
#[instruction(market_id: u64)]
pub struct CancelMarket<'info> {
    #[account(mut, seeds = [b"market", market_id.to_le_bytes().as_ref()], bump)]
    pub market_account: Account<'info, MarketAccount>,
    /// Oracle answer account (from trace9 program)
    /// CHECK: This should be verified to come from the oracle program
    pub oracle_answer: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(market_id: u64)]
pub struct ClaimRefund<'info> {
    #[account(mut, seeds = [b"market", market_id.to_le_bytes().as_ref()], bump)]
    pub market_account: Account<'info, MarketAccount>,
    #[account(
        mut,
        seeds = [b"position", market_id.to_le_bytes().as_ref(), refundee.key().as_ref()],
        bump
    )]
    pub position: Account<'info, Position>,
    #[account(mut)]
    pub refundee: Signer<'info>,
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
    pub authority: Pubkey,        // 32 bytes
    pub oracle_program: Pubkey,    // 32 bytes
    pub market_counter: u64,       // 8 bytes
    pub fee_percentage: u16,       // 2 bytes (basis points)
    pub accumulated_fees: u64,     // 8 bytes
    pub bump: u8,                  // 1 byte
}

impl MarketState {
    pub const LEN: usize = 32 + 32 + 8 + 2 + 8 + 1;
}

#[account]
pub struct MarketAccount {
    pub market_id: u64,            // 8 bytes
    pub question: String,           // 4 + len bytes
    pub resolution_time: i64,      // 8 bytes
    pub yes_pool: u64,              // 8 bytes
    pub no_pool: u64,               // 8 bytes
    pub status: MarketStatus,       // 1 byte
    pub outcome: Outcome,           // 1 byte
    pub total_fees: u64,            // 8 bytes
    pub created_at: i64,            // 8 bytes
    pub creator: Pubkey,            // 32 bytes
}

impl MarketAccount {
    pub const LEN: usize = 8 + (4 + 500) + 8 + 8 + 8 + 1 + 1 + 8 + 8 + 32;
}

#[account]
pub struct Position {
    pub yes_amount: u64,           // 8 bytes
    pub no_amount: u64,            // 8 bytes
    pub claimed: bool,              // 1 byte
}

impl Position {
    pub const LEN: usize = 8 + 8 + 1;
}

// Oracle answer structure (matches trace9 program)
#[account]
pub struct OracleAnswer {
    pub question_id: u64,
    pub provider: Pubkey,
    pub confidence_score: u8,
    pub bool_answer: bool,
    pub numeric_answer: u64,
    pub timestamp: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum MarketStatus {
    Open,
    Closed,
    Resolved,
    Canceled,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Unresolved,
    Yes,
    No,
}

#[event]
pub struct MarketCreated {
    pub market_id: u64,
    pub question: String,
    pub resolution_time: i64,
    pub creator: Pubkey,
}

#[event]
pub struct PositionTaken {
    pub market_id: u64,
    pub user: Pubkey,
    pub is_yes: bool,
    pub amount: u64,
}

#[event]
pub struct MarketResolved {
    pub market_id: u64,
    pub outcome: Outcome,
}

#[event]
pub struct MarketCanceled {
    pub market_id: u64,
}

#[event]
pub struct WinningsClaimed {
    pub market_id: u64,
    pub user: Pubkey,
    pub amount: u64,
}

#[event]
pub struct FeesWithdrawn {
    pub amount: u64,
    pub authority: Pubkey,
}

#[error_code]
pub enum MarketError {
    #[msg("Invalid question")]
    InvalidQuestion,
    #[msg("Invalid resolution time")]
    InvalidResolutionTime,
    #[msg("Market not open")]
    MarketNotOpen,
    #[msg("Market expired")]
    MarketExpired,
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
    #[msg("Too early to cancel")]
    TooEarlyToCancel,
    #[msg("Already answered")]
    AlreadyAnswered,
    #[msg("Not canceled")]
    NotCanceled,
    #[msg("No position")]
    NoPosition,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("No fees")]
    NoFees,
}

