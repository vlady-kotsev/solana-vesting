#![allow(clippy::result_large_err)]

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface},
};

declare_id!("3aZrnm9MmScGeMX49eoqoUeAdwbw21ktmaKwLi16k9gs");

#[program]
pub mod tokenvesting {
    use super::*;

    pub fn create_vesting_account(
        ctx: Context<CreateVestingAccount>,
        company_name: String,
    ) -> Result<()> {
        *ctx.accounts.vesting_account = VestingAccount {
            owner: ctx.accounts.signer.key(),
            mint: ctx.accounts.mint.key(),
            treasury_token_account: ctx.accounts.treasury_token_account.key(),
            company_name,
            treasury_bump: ctx.bumps.treasury_token_account,
            bump: ctx.bumps.vesting_account,
        };
        Ok(())
    }

    pub fn create_employee_account(
        ctx: Context<CreateEmployeeAccount>,
        start_time: i64,
        end_time: i64,
        cliff_time: i64,
        total_ammount: u64,
    ) -> Result<()> {
        *ctx.accounts.employee_account = EmployeeAccount {
            beneficiary: ctx.accounts.beneficiary.key(),
            start_time,
            end_time,
            cliff_time,
            vesting_account: ctx.accounts.vesting_account.key(),
            total_ammount,
            total_withdrawn: 0,
            bump: ctx.bumps.employee_account,
        };
        Ok(())
    }

    pub fn claim_tokens(ctx: Context<ClaimTokens>, company_name: String) -> Result<()> {
        let employee_account = &mut ctx.accounts.employee_account;

        let time_now = Clock::get()?.unix_timestamp;

        if time_now <= employee_account.cliff_time {
            return Err(ErrorCode::CliffTimeNotPassed.into());
        }

        let time_passed = time_now.saturating_sub(employee_account.start_time);

        let total_time = employee_account
            .end_time
            .saturating_sub(employee_account.start_time);

        if total_time == 0 {
            return Err(ErrorCode::InvalidVestingTime.into());
        }

        let claimable_amount = if time_now > employee_account.end_time {
            employee_account.total_ammount
        } else {
            match employee_account
                .total_ammount
                .checked_mul(time_passed as u64)
            {
                Some(product) => product / total_time as u64,
                None => return Err(ErrorCode::InvalidVestingAmount.into()),
            }
        };

        let available_withdraw_tokens =
            claimable_amount.saturating_sub(employee_account.total_withdrawn);

        if available_withdraw_tokens == 0 {
            return Err(ErrorCode::NoTokensToWithdraw.into());
        }

        let cpi_transfer_accounts = token_interface::TransferChecked {
            from: ctx.accounts.treasury_token_account.to_account_info(),
            to: ctx.accounts.employee_token_account.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            authority: ctx.accounts.treasury_token_account.to_account_info(),
        };

        let signer_seeds: &[&[&[u8]]] = &[&[
            b"vesting_treasury",
            company_name.as_ref(),
            &[ctx.accounts.vesting_account.treasury_bump],
        ]];

        let cpi_context = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            cpi_transfer_accounts,
        )
        .with_signer(signer_seeds);
        let decimals = ctx.accounts.mint.decimals;
        token_interface::transfer_checked(cpi_context, available_withdraw_tokens, decimals)?;
        employee_account.total_withdrawn += available_withdraw_tokens;
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(company_name: String)]
pub struct CreateVestingAccount<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
      init,
      space = 8 + VestingAccount::INIT_SPACE,
      payer = signer,
      seeds = [company_name.as_ref()],
      bump
    )]
    pub vesting_account: Account<'info, VestingAccount>,
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
      init,
      payer = signer,
      token::mint = mint,
      token::authority = treasury_token_account,
      seeds = [  b"vesting_treasury",company_name.as_bytes(),],
      bump
    )]
    pub treasury_token_account: InterfaceAccount<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct CreateEmployeeAccount<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    pub beneficiary: SystemAccount<'info>,
    #[account(
      has_one = owner,
    )]
    pub vesting_account: Account<'info, VestingAccount>,
    #[account(
    init,
    payer = owner,
    space = 8 + EmployeeAccount::INIT_SPACE,
    seeds = [b"employee_vesting", beneficiary.key().as_ref(), vesting_account.key().as_ref()],
    bump
  )]
    pub employee_account: Account<'info, EmployeeAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(company_name: String)]
pub struct ClaimTokens<'info> {
    #[account(mut)]
    pub beneficiary: Signer<'info>,
    #[account(
    mut,
    seeds = [b"employee_vesting", beneficiary.key().as_ref(), vesting_account.key().as_ref()],
    bump = employee_account.bump,
    has_one = vesting_account,
    has_one = beneficiary,
  )]
    pub employee_account: Account<'info, EmployeeAccount>,
    #[account(
      mut,
      seeds = [company_name.as_ref()],
      bump = vesting_account.bump,
      has_one = mint,
      has_one = treasury_token_account
    )]
    pub vesting_account: Account<'info, VestingAccount>,
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub treasury_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
      init_if_needed,
      payer = beneficiary,
      associated_token::mint = mint,
      associated_token::authority = beneficiary,
      associated_token::token_program = token_program
    )]
    pub employee_token_account: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[account]
#[derive(InitSpace)]
pub struct VestingAccount {
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub treasury_token_account: Pubkey,
    #[max_len(64)]
    pub company_name: String,
    pub treasury_bump: u8,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct EmployeeAccount {
    pub beneficiary: Pubkey,
    pub start_time: i64,
    pub end_time: i64,
    pub cliff_time: i64,
    pub vesting_account: Pubkey,
    pub total_ammount: u64,
    pub total_withdrawn: u64,
    pub bump: u8,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Cliff time still not passed!")]
    CliffTimeNotPassed,
    #[msg("Invalid vesting time!")]
    InvalidVestingTime,
    #[msg("Invalid vesting amount!")]
    InvalidVestingAmount,
    #[msg("No tokens to withdraw!")]
    NoTokensToWithdraw,
}
