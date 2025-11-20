use anchor_lang::prelude::*;

declare_id!("condMarkM3mP9vK8JqF2nH5xY7wD4bC6eA8g");

#[program]
pub mod conditional_market {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, fee_percentage: u16) -> Result<()> {
        let market_state = &mut ctx.accounts.market_state;
        market_state.authority = ctx.accounts.authority.key();
        market_state.market_counter = 0;
        market_state.fee_percentage = fee_percentage;
        market_state.bump = ctx.bumps.market_state;
        Ok(())
    }

    pub fn create_market(
        ctx: Context<CreateMarket>,
        question: String,
        parent_market: Pubkey,
        required_parent_outcome: u8,
    ) -> Result<u64> {
        require!(
            question.len() > 0 && question.len() <= 500,
            MarketError::InvalidQuestion
        );
        require!(
            parent_market != Pubkey::default(),
            MarketError::InvalidParentMarket
        );

        let market_state = &mut ctx.accounts.market_state;
        let market_id = market_state.market_counter;
        let market_account = &mut ctx.accounts.market_account;

        market_account.market_id = market_id;
        market_account.question = question.clone();
        market_account.parent_market = parent_market;
        market_account.required_parent_outcome = required_parent_outcome;
        market_account.yes_pool = 0;
        market_account.no_pool = 0;
        market_account.total_fees = 0;
        market_account.created_at = Clock::get()?.unix_timestamp;
        market_account.resolved_at = 0;
        market_account.status = MarketStatus::Active;
        market_account.final_outcome = false;

        market_state.market_counter = market_id.checked_add(1).unwrap();

        emit!(MarketCreated {
            market_id,
            question,
            parent_market,
            required_outcome: required_parent_outcome,
        });

        Ok(market_id)
    }

    pub fn take_position(
        ctx: Context<TakePosition>,
        market_id: u64,
        prediction: bool,
    ) -> Result<()> {
        let market_account = &mut ctx.accounts.market_account;
        require!(
            market_account.status == MarketStatus::Active,
            MarketError::MarketNotActive
        );

        let bet_amount = ctx.accounts.bettor.lamports();
        require!(bet_amount > 0, MarketError::ZeroBet);

        let market_state = &ctx.accounts.market_state;
        let fee = bet_amount
            .checked_mul(market_state.fee_percentage as u64)
            .and_then(|x| x.checked_div(10000))
            .ok_or(MarketError::Overflow)?;
        let bet_amount_net = bet_amount.checked_sub(fee).ok_or(MarketError::Overflow)?;

        market_account.total_fees = market_account
            .total_fees
            .checked_add(fee)
            .ok_or(MarketError::Overflow)?;

        let position = &mut ctx.accounts.position;
        if prediction {
            market_account.yes_pool = market_account
                .yes_pool
                .checked_add(bet_amount_net)
                .ok_or(MarketError::Overflow)?;
            position.yes_amount = position
                .yes_amount
                .checked_add(bet_amount_net)
                .ok_or(MarketError::Overflow)?;
        } else {
            market_account.no_pool = market_account
                .no_pool
                .checked_add(bet_amount_net)
                .ok_or(MarketError::Overflow)?;
            position.no_amount = position
                .no_amount
                .checked_add(bet_amount_net)
                .ok_or(MarketError::Overflow)?;
        }

        **ctx.accounts.bettor.to_account_info().try_borrow_mut_lamports()? -= bet_amount;
        **ctx.accounts.market_account.to_account_info().try_borrow_mut_lamports()? += bet_amount;

        emit!(PositionTaken {
            market_id,
            user: ctx.accounts.bettor.key(),
            prediction,
            amount: bet_amount_net,
        });

        Ok(())
    }

    pub fn check_parent_market(ctx: Context<CheckParentMarket>, market_id: u64) -> Result<()> {
        let market_account = &mut ctx.accounts.market_account;
        require!(
            market_account.status == MarketStatus::Active,
            MarketError::MarketNotActive
        );

        // Check parent market outcome (simplified - would use CPI in production)
        // For now, we assume parent market outcome is provided
        let parent_outcome = ctx.accounts.parent_market_outcome.outcome;
        let parent_resolved = ctx.accounts.parent_market_outcome.resolved;

        require!(parent_resolved, MarketError::ParentNotResolved);

        let condition_met = (parent_outcome == 1 && market_account.required_parent_outcome == 1)
            || (parent_outcome == 0 && market_account.required_parent_outcome == 0);

        if condition_met {
            market_account.status = MarketStatus::ParentUnresolved;
        } else {
            market_account.status = MarketStatus::ConditionNotMet;
            // Refund all participants
            // Note: In production, would need to track all participants for refunds
        }

        emit!(ParentResolved {
            market_id,
            condition_met,
        });

        Ok(())
    }

    pub fn resolve_market(
        ctx: Context<ResolveMarket>,
        market_id: u64,
        outcome: bool,
    ) -> Result<()> {
        let market_account = &mut ctx.accounts.market_account;
        require!(
            market_account.status == MarketStatus::ParentUnresolved
                || market_account.status == MarketStatus::Active,
            MarketError::CannotResolve
        );

        market_account.final_outcome = outcome;
        market_account.resolved_at = Clock::get()?.unix_timestamp;
        market_account.status = MarketStatus::Resolved;

        emit!(MarketResolved {
            market_id,
            outcome,
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

        let winning_pool = if market_account.final_outcome {
            market_account.yes_pool
        } else {
            market_account.no_pool
        };
        let losing_pool = if market_account.final_outcome {
            market_account.no_pool
        } else {
            market_account.yes_pool
        };
        let user_winning_amount = if market_account.final_outcome {
            position.yes_amount
        } else {
            position.no_amount
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
            amount: payout,
        });

        Ok(())
    }

    pub fn get_refund(ctx: Context<GetRefund>, market_id: u64) -> Result<()> {
        let market_account = &mut ctx.accounts.market_account;
        require!(
            market_account.status == MarketStatus::ConditionNotMet
                || market_account.status == MarketStatus::Cancelled,
            MarketError::NotCancelled
        );

        let position = &mut ctx.accounts.position;
        require!(!position.claimed, MarketError::AlreadyClaimed);

        let refund_amount = position
            .yes_amount
            .checked_add(position.no_amount)
            .ok_or(MarketError::Overflow)?;

        require!(refund_amount > 0, MarketError::NoPosition);

        position.claimed = true;

        **ctx.accounts.market_account.to_account_info().try_borrow_mut_lamports()? -= refund_amount;
        **ctx.accounts.refundee.to_account_info().try_borrow_mut_lamports()? += refund_amount;

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
pub struct CheckParentMarket<'info> {
    #[account(mut, seeds = [b"market", market_id.to_le_bytes().as_ref()], bump)]
    pub market_account: Account<'info, MarketAccount>,
    /// CHECK: Parent market outcome account
    pub parent_market_outcome: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(market_id: u64)]
pub struct ResolveMarket<'info> {
    #[account(mut, seeds = [b"market", market_id.to_le_bytes().as_ref()], bump)]
    pub market_account: Account<'info, MarketAccount>,
    pub authority: Signer<'info>,
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
pub struct GetRefund<'info> {
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

#[account]
pub struct MarketState {
    pub authority: Pubkey,
    pub market_counter: u64,
    pub fee_percentage: u16,
    pub bump: u8,
}

impl MarketState {
    pub const LEN: usize = 32 + 8 + 2 + 1;
}

#[account]
pub struct MarketAccount {
    pub market_id: u64,
    pub question: String,
    pub parent_market: Pubkey,
    pub required_parent_outcome: u8,
    pub yes_pool: u64,
    pub no_pool: u64,
    pub total_fees: u64,
    pub created_at: i64,
    pub resolved_at: i64,
    pub status: MarketStatus,
    pub final_outcome: bool,
}

impl MarketAccount {
    pub const LEN: usize = 8 + (4 + 500) + 32 + 1 + 8 + 8 + 8 + 8 + 8 + 1 + 1;
}

#[account]
pub struct Position {
    pub yes_amount: u64,
    pub no_amount: u64,
    pub claimed: bool,
}

impl Position {
    pub const LEN: usize = 8 + 8 + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum MarketStatus {
    Active,
    ParentUnresolved,
    ConditionNotMet,
    Resolved,
    Cancelled,
}

#[event]
pub struct MarketCreated {
    pub market_id: u64,
    pub question: String,
    pub parent_market: Pubkey,
    pub required_outcome: u8,
}

#[event]
pub struct PositionTaken {
    pub market_id: u64,
    pub user: Pubkey,
    pub prediction: bool,
    pub amount: u64,
}

#[event]
pub struct ParentResolved {
    pub market_id: u64,
    pub condition_met: bool,
}

#[event]
pub struct MarketResolved {
    pub market_id: u64,
    pub outcome: bool,
}

#[event]
pub struct WinningsClaimed {
    pub market_id: u64,
    pub user: Pubkey,
    pub amount: u64,
}

#[error_code]
pub enum MarketError {
    #[msg("Invalid question")]
    InvalidQuestion,
    #[msg("Invalid parent market")]
    InvalidParentMarket,
    #[msg("Market not active")]
    MarketNotActive,
    #[msg("Zero bet")]
    ZeroBet,
    #[msg("Overflow")]
    Overflow,
    #[msg("Parent not resolved")]
    ParentNotResolved,
    #[msg("Cannot resolve")]
    CannotResolve,
    #[msg("Not resolved")]
    NotResolved,
    #[msg("Already claimed")]
    AlreadyClaimed,
    #[msg("Not winner")]
    NotWinner,
    #[msg("No winnings")]
    NoWinnings,
    #[msg("Not cancelled")]
    NotCancelled,
    #[msg("No position")]
    NoPosition,
}

