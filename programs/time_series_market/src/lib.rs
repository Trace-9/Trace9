use anchor_lang::prelude::*;

declare_id!("timeSeriM3mP9vK8JqF2nH5xY7wD4bC6eA8g");

#[program]
pub mod time_series_market {
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
        market_state.bump = ctx.bumps.market_state;
        Ok(())
    }

    pub fn create_market(
        ctx: Context<CreateMarket>,
        question: String,
        deadlines: Vec<i64>,
    ) -> Result<u64> {
        require!(
            question.len() > 0 && question.len() <= 500,
            MarketError::InvalidQuestion
        );
        require!(
            deadlines.len() >= 2 && deadlines.len() <= 12,
            MarketError::InvalidPeriodCount
        );

        // Verify deadlines are in ascending order
        for i in 1..deadlines.len() {
            require!(
                deadlines[i] > deadlines[i - 1],
                MarketError::DeadlinesNotAscending
            );
        }

        let market_state = &mut ctx.accounts.market_state;
        let market_id = market_state.market_counter;
        let market_account = &mut ctx.accounts.market_account;

        market_account.market_id = market_id;
        market_account.question = question.clone();
        market_account.success_pool = 0;
        market_account.failure_pool = 0;
        market_account.total_fees = 0;
        market_account.created_at = Clock::get()?.unix_timestamp;
        market_account.all_resolved = false;
        market_account.all_success = false;

        // Initialize periods
        for deadline in deadlines.iter() {
            market_account.periods.push(TimePeriod {
                deadline: *deadline,
                question_id: 0, // Will be set when oracle question is asked
                result: 0,
                resolved: false,
            });
        }

        market_state.market_counter = market_id.checked_add(1).unwrap();

        emit!(MarketCreated {
            market_id,
            question,
            period_count: deadlines.len() as u8,
        });

        Ok(market_id)
    }

    pub fn take_position(
        ctx: Context<TakePosition>,
        market_id: u64,
        predict_all_success: bool,
    ) -> Result<()> {
        let market_account = &mut ctx.accounts.market_account;
        require!(!market_account.all_resolved, MarketError::MarketResolved);

        let bet_amount = ctx.accounts.bettor.lamports();
        require!(bet_amount > 0, MarketError::ZeroBet);

        let market_state = &ctx.accounts.market_state;
        let fee = bet_amount
            .checked_mul(market_state.fee_percentage as u64)
            .and_then(|x| x.checked_div(10000))
            .ok_or(MarketError::Overflow)?;
        let net_amount = bet_amount.checked_sub(fee).ok_or(MarketError::Overflow)?;

        market_account.total_fees = market_account
            .total_fees
            .checked_add(fee)
            .ok_or(MarketError::Overflow)?;

        if predict_all_success {
            market_account.success_pool = market_account
                .success_pool
                .checked_add(net_amount)
                .ok_or(MarketError::Overflow)?;
        } else {
            market_account.failure_pool = market_account
                .failure_pool
                .checked_add(net_amount)
                .ok_or(MarketError::Overflow)?;
        }

        let position = &mut ctx.accounts.position;
        if predict_all_success {
            position.success_amount = position
                .success_amount
                .checked_add(net_amount)
                .ok_or(MarketError::Overflow)?;
        } else {
            position.failure_amount = position
                .failure_amount
                .checked_add(net_amount)
                .ok_or(MarketError::Overflow)?;
        }

        **ctx.accounts.bettor.to_account_info().try_borrow_mut_lamports()? -= bet_amount;
        **ctx.accounts.market_account.to_account_info().try_borrow_mut_lamports()? += bet_amount;

        emit!(PositionTaken {
            market_id,
            user: ctx.accounts.bettor.key(),
            predict_all_success,
            amount: net_amount,
        });

        Ok(())
    }

    pub fn resolve_period(
        ctx: Context<ResolvePeriod>,
        market_id: u64,
        period_index: u8,
    ) -> Result<()> {
        let market_account = &mut ctx.accounts.market_account;
        require!(
            (period_index as usize) < market_account.periods.len(),
            MarketError::InvalidPeriod
        );

        let period = &mut market_account.periods[period_index as usize];
        require!(!period.resolved, MarketError::PeriodAlreadyResolved);
        require!(
            Clock::get()?.unix_timestamp >= period.deadline,
            MarketError::TooEarly
        );

        let numeric_answer = ctx.accounts.oracle_answer.numeric_answer;
        require!(numeric_answer > 0, MarketError::OracleNotAnswered);

        period.result = numeric_answer;
        period.resolved = true;

        emit!(PeriodResolved {
            market_id,
            period_index,
            result: numeric_answer,
        });

        // Check if all periods resolved
        check_all_resolved(&mut market_account)?;

        Ok(())
    }

    fn check_all_resolved(market_account: &mut Account<MarketAccount>) -> Result<()> {
        let mut all_resolved = true;
        let mut all_success = true;

        for period in market_account.periods.iter() {
            if !period.resolved {
                all_resolved = false;
                break;
            }
            // Success criteria: result > 0
            if period.result == 0 {
                all_success = false;
            }
        }

        if all_resolved {
            market_account.all_resolved = true;
            market_account.all_success = all_success;

            emit!(MarketResolved {
                market_id: market_account.market_id,
                all_success,
            });
        }

        Ok(())
    }

    pub fn claim_winnings(ctx: Context<ClaimWinnings>, market_id: u64) -> Result<()> {
        let market_account = &ctx.accounts.market_account;
        require!(market_account.all_resolved, MarketError::NotAllResolved);

        let position = &mut ctx.accounts.position;
        require!(!position.claimed, MarketError::AlreadyClaimed);

        let winning_pool = if market_account.all_success {
            market_account.success_pool
        } else {
            market_account.failure_pool
        };
        let losing_pool = if market_account.all_success {
            market_account.failure_pool
        } else {
            market_account.success_pool
        };
        let user_winning_amount = if market_account.all_success {
            position.success_amount
        } else {
            position.failure_amount
        };

        require!(user_winning_amount > 0, MarketError::NotWinner);
        require!(winning_pool > 0, MarketError::NoWinnings);

        let payout = user_winning_amount
            .checked_add(
                user_winning_amount
                    .checked_mul(losing_pool)
                    .and_then(|x| x.checked_div(winning_pool))
                    .ok_or(MarketError::Overflow)?,
            )
            .ok_or(MarketError::Overflow)?;

        position.claimed = true;

        **ctx.accounts.market_account.to_account_info().try_borrow_mut_lamports()? -= payout;
        **ctx.accounts.winner.to_account_info().try_borrow_mut_lamports()? += payout;

        emit!(WinningsClaimed {
            market_id,
            user: ctx.accounts.winner.key(),
            payout,
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
#[instruction(market_id: u64, period_index: u8)]
pub struct ResolvePeriod<'info> {
    #[account(mut, seeds = [b"market", market_id.to_le_bytes().as_ref()], bump)]
    pub market_account: Account<'info, MarketAccount>,
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

#[account]
pub struct MarketState {
    pub authority: Pubkey,
    pub oracle_program: Pubkey,
    pub market_counter: u64,
    pub fee_percentage: u16,
    pub bump: u8,
}

impl MarketState {
    pub const LEN: usize = 32 + 32 + 8 + 2 + 1;
}

#[account]
pub struct MarketAccount {
    pub market_id: u64,
    pub question: String,
    pub periods: Vec<TimePeriod>,
    pub success_pool: u64,
    pub failure_pool: u64,
    pub total_fees: u64,
    pub created_at: i64,
    pub all_resolved: bool,
    pub all_success: bool,
}

impl MarketAccount {
    pub const LEN: usize = 8 + (4 + 500) + (4 + 12 * TimePeriod::LEN) + 8 + 8 + 8 + 8 + 1 + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TimePeriod {
    pub deadline: i64,
    pub question_id: u64,
    pub result: u64,
    pub resolved: bool,
}

impl TimePeriod {
    pub const LEN: usize = 8 + 8 + 8 + 1;
}

#[account]
pub struct Position {
    pub success_amount: u64,
    pub failure_amount: u64,
    pub claimed: bool,
}

impl Position {
    pub const LEN: usize = 8 + 8 + 1;
}

#[event]
pub struct MarketCreated {
    pub market_id: u64,
    pub question: String,
    pub period_count: u8,
}

#[event]
pub struct PositionTaken {
    pub market_id: u64,
    pub user: Pubkey,
    pub predict_all_success: bool,
    pub amount: u64,
}

#[event]
pub struct PeriodResolved {
    pub market_id: u64,
    pub period_index: u8,
    pub result: u64,
}

#[event]
pub struct MarketResolved {
    pub market_id: u64,
    pub all_success: bool,
}

#[event]
pub struct WinningsClaimed {
    pub market_id: u64,
    pub user: Pubkey,
    pub payout: u64,
}

#[error_code]
pub enum MarketError {
    #[msg("Invalid question")]
    InvalidQuestion,
    #[msg("Invalid period count")]
    InvalidPeriodCount,
    #[msg("Deadlines not ascending")]
    DeadlinesNotAscending,
    #[msg("Market resolved")]
    MarketResolved,
    #[msg("Zero bet")]
    ZeroBet,
    #[msg("Overflow")]
    Overflow,
    #[msg("Invalid period")]
    InvalidPeriod,
    #[msg("Period already resolved")]
    PeriodAlreadyResolved,
    #[msg("Too early")]
    TooEarly,
    #[msg("Oracle not answered")]
    OracleNotAnswered,
    #[msg("Not all resolved")]
    NotAllResolved,
    #[msg("Already claimed")]
    AlreadyClaimed,
    #[msg("Not winner")]
    NotWinner,
    #[msg("No winnings")]
    NoWinnings,
}

