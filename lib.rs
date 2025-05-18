use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

#[program]
pub mod solana_deposit_program {
    use super::*;

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        if ctx.accounts.user_account.owner == Pubkey::default() {
            ctx.accounts.user_account.owner = ctx.accounts.user.key();
        } else {
            require!(
                ctx.accounts.user_account.owner == ctx.accounts.user.key(),
                ErrorCode::Unauthorized
            );
        }

        let cpi_ctx = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user.to_account_info(),
                to: ctx.accounts.user_account.to_account_info(),
            },
        );
        transfer(cpi_ctx, amount)?;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        require!(
            ctx.accounts.user_account.owner == ctx.accounts.user.key(),
            ErrorCode::Unauthorized
        );

        let account_info = ctx.accounts.user_account.to_account_info();
        let rent = Rent::get()?;
        let rent_exempt = rent.minimum_balance(account_info.data_len());
        let available_balance = account_info
            .lamports()
            .checked_sub(rent_exempt)
            .ok_or(ErrorCode::InsufficientFunds)?;

        require!(available_balance >= amount, ErrorCode::InsufficientFunds);

        let seeds = &[
            b"user_account",
            ctx.accounts.user.key().as_ref(),
            &[ctx.bumps.user_account],
        ];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: account_info.clone(),
                to: ctx.accounts.user.to_account_info(),
            },
            &[&seeds[..]],
        );
        transfer(cpi_ctx, amount)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + 32,
        seeds = [b"user_account", user.key().as_ref()],
        bump,
    )]
    pub user_account: Account<'info, UserAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [b"user_account", user.key().as_ref()],
        bump,
        has_one = user,
    )]
    pub user_account: Account<'info, UserAccount>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct UserAccount {
    pub owner: Pubkey,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient funds")]
    InsufficientFunds,
    #[msg("Unauthorized")]
    Unauthorized,
}
