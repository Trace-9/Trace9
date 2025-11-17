use anchor_lang::prelude::*;
use anchor_lang::system_program;

declare_id!("payFaciL3mP9vK8JqF2nH5xY7wD4bC6eA8g");

#[program]
pub mod payment_facilitator {
    use super::*;

    /// Initialize the payment facilitator
    pub fn initialize(ctx: Context<Initialize>, platform_fee_bps: u16) -> Result<()> {
        require!(platform_fee_bps <= 1000, PaymentFacilitatorError::InvalidFee); // Max 10%
        
        let facilitator = &mut ctx.accounts.facilitator;
        facilitator.authority = ctx.accounts.authority.key();
        facilitator.platform_fee_bps = platform_fee_bps;
        facilitator.accumulated_fees = 0;
        facilitator.bump = ctx.bumps.facilitator;
        
        Ok(())
    }

    /// Settle a single payment
    pub fn settle_payment(
        ctx: Context<SettlePayment>,
        amount: u64,
        payment_id: [u8; 32],
    ) -> Result<()> {
        require!(amount > 0, PaymentFacilitatorError::InvalidAmount);
        
        let facilitator = &mut ctx.accounts.facilitator;
        
        // Check if payment already used
        require!(
            !facilitator.used_payments.contains(&payment_id),
            PaymentFacilitatorError::PaymentUsed
        );
        
        facilitator.used_payments.push(payment_id);
        
        // Calculate fee
        let fee = (amount as u128)
            .checked_mul(facilitator.platform_fee_bps as u128)
            .and_then(|f| f.checked_div(10000))
            .ok_or(PaymentFacilitatorError::Overflow)? as u64;
        
        let recipient_amount = amount.checked_sub(fee).ok_or(PaymentFacilitatorError::Overflow)?;
        
        // Transfer to recipient
        anchor_lang::solana_program::program::invoke(
            &anchor_lang::solana_program::system_instruction::transfer(
                ctx.accounts.payer.key,
                ctx.accounts.recipient.key,
                recipient_amount,
            ),
            &[
                ctx.accounts.payer.to_account_info(),
                ctx.accounts.recipient.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;
        
        // Transfer fee to facilitator
        if fee > 0 {
            anchor_lang::solana_program::program::invoke(
                &anchor_lang::solana_program::system_instruction::transfer(
                    ctx.accounts.payer.key,
                    ctx.accounts.facilitator.key,
                    fee,
                ),
                &[
                    ctx.accounts.payer.to_account_info(),
                    ctx.accounts.facilitator.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
            )?;
            
            facilitator.accumulated_fees = facilitator
                .accumulated_fees
                .checked_add(fee)
                .ok_or(PaymentFacilitatorError::Overflow)?;
        }
        
        emit!(PaymentSettled {
            payer: ctx.accounts.payer.key(),
            recipient: ctx.accounts.recipient.key(),
            amount,
            fee,
            payment_id,
        });
        
        Ok(())
    }

    /// Batch settle multiple payments
    pub fn batch_settle_payments(
        ctx: Context<BatchSettlePayments>,
        amounts: Vec<u64>,
        recipients: Vec<Pubkey>,
        payment_ids: Vec<[u8; 32]>,
    ) -> Result<()> {
        require!(
            amounts.len() == recipients.len() && amounts.len() == payment_ids.len(),
            PaymentFacilitatorError::InvalidBatch
        );
        require!(amounts.len() > 0 && amounts.len() <= 20, PaymentFacilitatorError::InvalidBatchSize);
        
        let facilitator = &mut ctx.accounts.facilitator;
        let mut total_fee = 0u64;
        
        for i in 0..amounts.len() {
            require!(amounts[i] > 0, PaymentFacilitatorError::InvalidAmount);
            require!(
                !facilitator.used_payments.contains(&payment_ids[i]),
                PaymentFacilitatorError::PaymentUsed
            );
            
            facilitator.used_payments.push(payment_ids[i]);
            
            let fee = (amounts[i] as u128)
                .checked_mul(facilitator.platform_fee_bps as u128)
                .and_then(|f| f.checked_div(10000))
                .ok_or(PaymentFacilitatorError::Overflow)? as u64;
            
            let recipient_amount = amounts[i]
                .checked_sub(fee)
                .ok_or(PaymentFacilitatorError::Overflow)?;
            
            total_fee = total_fee.checked_add(fee).ok_or(PaymentFacilitatorError::Overflow)?;
            
            // Transfer to recipient
            anchor_lang::solana_program::program::invoke(
                &anchor_lang::solana_program::system_instruction::transfer(
                    ctx.accounts.payer.key,
                    &recipients[i],
                    recipient_amount,
                ),
                &[
                    ctx.accounts.payer.to_account_info(),
                    ctx.accounts.recipients[i].to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
            )?;
        }
        
        // Transfer all fees at once
        if total_fee > 0 {
            anchor_lang::solana_program::program::invoke(
                &anchor_lang::solana_program::system_instruction::transfer(
                    ctx.accounts.payer.key,
                    ctx.accounts.facilitator.key,
                    total_fee,
                ),
                &[
                    ctx.accounts.payer.to_account_info(),
                    ctx.accounts.facilitator.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
            )?;
            
            facilitator.accumulated_fees = facilitator
                .accumulated_fees
                .checked_add(total_fee)
                .ok_or(PaymentFacilitatorError::Overflow)?;
        }
        
        emit!(BatchPaymentsSettled {
            payer: ctx.accounts.payer.key(),
            count: amounts.len() as u8,
            total_amount: amounts.iter().sum(),
            total_fee,
        });
        
        Ok(())
    }

    /// Withdraw accumulated fees (authority only)
    pub fn withdraw_fees(ctx: Context<WithdrawFees>) -> Result<()> {
        require!(
            ctx.accounts.authority.key() == ctx.accounts.facilitator.authority,
            PaymentFacilitatorError::Unauthorized
        );
        
        let facilitator = &mut ctx.accounts.facilitator;
        let amount = facilitator.accumulated_fees;
        require!(amount > 0, PaymentFacilitatorError::NoFees);
        
        facilitator.accumulated_fees = 0;
        
        // Transfer fees to authority
        **ctx.accounts.facilitator.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.authority.to_account_info().try_borrow_mut_lamports()? += amount;
        
        emit!(FeesWithdrawn {
            to: ctx.accounts.authority.key(),
            amount,
        });
        
        Ok(())
    }

    /// Update platform fee (authority only)
    pub fn update_platform_fee(ctx: Context<UpdatePlatformFee>, new_fee_bps: u16) -> Result<()> {
        require!(
            ctx.accounts.authority.key() == ctx.accounts.facilitator.authority,
            PaymentFacilitatorError::Unauthorized
        );
        require!(new_fee_bps <= 1000, PaymentFacilitatorError::InvalidFee);
        
        let old_fee = ctx.accounts.facilitator.platform_fee_bps;
        ctx.accounts.facilitator.platform_fee_bps = new_fee_bps;
        
        emit!(PlatformFeeUpdated {
            old_fee,
            new_fee: new_fee_bps,
        });
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + PaymentFacilitator::LEN,
        seeds = [b"payment_facilitator"],
        bump
    )]
    pub facilitator: Account<'info, PaymentFacilitator>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SettlePayment<'info> {
    #[account(mut, seeds = [b"payment_facilitator"], bump = facilitator.bump)]
    pub facilitator: Account<'info, PaymentFacilitator>,
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: Recipient can be any account
    #[account(mut)]
    pub recipient: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BatchSettlePayments<'info> {
    #[account(mut, seeds = [b"payment_facilitator"], bump = facilitator.bump)]
    pub facilitator: Account<'info, PaymentFacilitator>,
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: Recipients can be any accounts
    /// Note: This is simplified - in production you'd need to handle variable recipients differently
    pub recipients: Vec<UncheckedAccount<'info>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawFees<'info> {
    #[account(mut, seeds = [b"payment_facilitator"], bump = facilitator.bump)]
    pub facilitator: Account<'info, PaymentFacilitator>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdatePlatformFee<'info> {
    #[account(mut, seeds = [b"payment_facilitator"], bump = facilitator.bump)]
    pub facilitator: Account<'info, PaymentFacilitator>,
    pub authority: Signer<'info>,
}

#[account]
pub struct PaymentFacilitator {
    pub authority: Pubkey,           // 32 bytes
    pub platform_fee_bps: u16,       // 2 bytes (basis points, e.g., 100 = 1%)
    pub accumulated_fees: u64,       // 8 bytes
    pub used_payments: Vec<[u8; 32]>, // Variable length
    pub bump: u8,                     // 1 byte
}

impl PaymentFacilitator {
    pub const LEN: usize = 32 + 2 + 8 + 4 + (32 * 100) + 1; // Space for up to 100 used payments
}

#[event]
pub struct PaymentSettled {
    pub payer: Pubkey,
    pub recipient: Pubkey,
    pub amount: u64,
    pub fee: u64,
    pub payment_id: [u8; 32],
}

#[event]
pub struct BatchPaymentsSettled {
    pub payer: Pubkey,
    pub count: u8,
    pub total_amount: u64,
    pub total_fee: u64,
}

#[event]
pub struct FeesWithdrawn {
    pub to: Pubkey,
    pub amount: u64,
}

#[event]
pub struct PlatformFeeUpdated {
    pub old_fee: u16,
    pub new_fee: u16,
}

#[error_code]
pub enum PaymentFacilitatorError {
    #[msg("Invalid fee")]
    InvalidFee,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Payment already used")]
    PaymentUsed,
    #[msg("Overflow")]
    Overflow,
    #[msg("Invalid batch")]
    InvalidBatch,
    #[msg("Invalid batch size")]
    InvalidBatchSize,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("No fees to withdraw")]
    NoFees,
}

