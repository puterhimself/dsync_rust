use core::slice::SlicePattern;

use anchor_lang::prelude::*;
use anchor_spl::token::{
    self, spl_token::instruction::AuthorityType, CloseAccount, Mint, SetAuthority, Token,
    TokenAccount, Transfer,
};
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

const CLIENT_SEED: &str = "DSYNC_CLIENT";
const VAULT_SEED: &str = "DSYNC_VAULT";
const AUTHORITY_SEED: &str = "DSYNC_AUTHORITY";
const JOB_SEED: &str = "DSYNC_JOB";

#[derive(Clone, Debug, PartialEq, AnchorSerialize, AnchorDeserialize, Copy)]
pub enum JobState {
    PENDING,
    PUBLISHED,
    ACTIVE,
    VALIDATED,
    COMPLETED,
    CANCELED,
}

#[program]
pub mod dsync_rust {
    use super::*;

    // const (TOKEN_AUTHORITY, _bump) = Pubkey::find_program_address(&[AUTHORITY_SEED.as_bytes()], ctx.program_id);

    pub fn initialize_client(ctx: Context<InitializeClient>) -> Result<()> {
        let client = &mut ctx.accounts.client;
        client.owner = *ctx.accounts.owner.key;
        client.bump = *ctx.bumps.get(CLIENT_SEED).unwrap();
        client.job_count = 0;
        Ok(())
    }

    pub fn initialize_job(
        ctx: Context<InitializeJob>,
        description: String,
        price: u64,
        deadline: i64,
    ) -> Result<()> {
        let _task = &mut ctx.accounts.task;
        let _client = &mut ctx.accounts.client;
        _client.job_count += 1;

        _task.bump = *ctx.bumps.get(JOB_SEED).unwrap();
        _task.index = _client.job_count;
        _task.client = _client.owner;
        _task.state = JobState::PENDING;
        _task.job_description = description;
        _task.publish_date = Clock::get().unwrap().unix_timestamp;
        _task.deadline = deadline;
        _task.price = price;
        _task.is_native = false;
        _task.currency = ctx.accounts.currency.to_account_info().key.clone();
        _task.client_token_account = ctx
            .accounts
            .client_token_account
            .to_account_info()
            .key
            .clone();
        _task.vault_token_account = ctx.accounts.vault.to_account_info().key.clone();

        Ok(())
    }

    pub fn publish_job(ctx: Context<PublishJob>) -> Result<()> {
        let _task = &mut ctx.accounts.task;
        _task.state = JobState::PUBLISHED;

        token::transfer(
            ctx.accounts.into_transfer_to_pda_context(),
            ctx.accounts.task.price,
        )?;
        // token::set_authority(
        //     ctx.accounts.into_set_authority_context(),
        //     AuthorityType::AccountOwner,
        //     Some(*ctx.program_id),
        // )?;
        Ok(())
    }

    pub fn cancel_job(ctx: Context<CancelJob>) -> Result<()> {
        let _task = &mut ctx.accounts.task;
        _task.state = JobState::CANCELED;

        token::transfer(
            ctx.accounts.into_transfer_to_client_context(),
            ctx.accounts.task.price,
        )?;
        // token::set_authority(
        //     ctx.accounts.into_set_authority_context(),
        //     AuthorityType::AccountOwner,
        //     Some(*ctx.program_id),
        // )?;
        Ok(())
    }

    

}

#[derive(Accounts)]
pub struct InitializeClient<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        init,
        seeds = [CLIENT_SEED.as_bytes()],
        bump,
        payer = owner,
        space = Client::SPACE
    )]
    pub client: Box<Account<'info, Client>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeJob<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut)]
    pub client: Box<Account<'info, Client>>,
    pub currency: Account<'info, Mint>,
    #[account(
        init,
        seeds = [JOB_SEED.as_bytes(), owner.key.as_ref(), &client.job_count.to_le_bytes()],
        bump,
        payer = owner,
        space = Job::SPACE,
        // authority = task.key
    )]
    pub task: Box<Account<'info, Job>>,
    #[account(
        init,
        payer = owner,
        seeds = [VAULT_SEED.as_bytes(), &task.job_id.as_ref()],
        bump,
        token::mint = currency,
        token::authority = vault
    )]
    pub vault: Box<Account<'info, TokenAccount>>,
    pub client_token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    pub system_program: Program<'info, System>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct PublishJob<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut)]
    pub task: Box<Account<'info, Job>>,
    #[account(mut)]
    pub vault: Box<Account<'info, TokenAccount>>,
    pub client_token_account: Box<Account<'info, TokenAccount>>,
    pub currency: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CancelJob<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub task: Box<Account<'info, Job>>,
    #[account(mut)]
    pub vault: Box<Account<'info, TokenAccount>>,
    pub client_token_account: Box<Account<'info, TokenAccount>>,
    pub currency: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct StartJob<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub task: Box<Account<'info, Job>>,
    // #[account(mut)]
    // pub vault: Box<Account<'info, TokenAccount>>,
    // pub client_token_account: Box<Account<'info, TokenAccount>>,
    // pub currency: Account<'info, Mint>,
    // pub token_program: Program<'info, Token>,
}

#[account]
pub struct Client {
    pub bump: u8,
    pub owner: Pubkey,
    pub job_count: u64,
}

#[account]
pub struct Job {
    pub bump: u8,
    pub index: u64,
    pub client: Pubkey,
    // pub client_account: Pubkey,
    pub job_id: String,
    pub state: JobState,
    pub job_description: String,

    pub winner: Pubkey,
    pub validator: Pubkey,
    pub submission_count: u64,

    pub price: u64,
    pub currency: Pubkey,
    pub is_native: bool,

    pub publish_date: i64,
    pub deadline: i64,

    pub client_token_account: Pubkey,
    pub validator_token_account: Pubkey,
    pub winner_token_account: Pubkey,
    pub vault_token_account: Pubkey,
}

impl Default for JobState {
    fn default() -> Self {
        JobState::PENDING
    }
}

impl<'info> PublishJob<'info> {
    fn into_transfer_to_pda_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.client_token_account.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.vault.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    // fn into_set_authority_context(&self) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
    //     let cpi_accounts = SetAuthority {
    //         account_or_mint: self.vault.to_account_info(),
    //         current_authority: self.owner.to_account_info(),
    //     };
    //     CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    // }
}

impl<'info> CancelJob<'info> {
    fn into_transfer_to_client_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.client_token_account.to_account_info(),
            authority: self.vault.to_account_info(),
        };
        let seeds = vec![
            VAULT_SEED.as_bytes(),
            &self.task.job_id.as_ref(),
            &self.task.bump.to_le_bytes(),
        ];
        CpiContext::new_with_signer(
            self.token_program.to_account_info(), 
            cpi_accounts,
            &[seeds.as_slice()]
        )
    }
}

impl Client {
    const SPACE: usize = 32 + 1 + 8 + 8;
}

impl Job {
    // const SPACE: usize = 1 + 8 + 32 + 32 + 32 + 32 + (4 + 10) + 8 + (1 + 32) + (4 + 20) + 8 + 8 + 8 + 1 + (32 * 5) + 8;
    const SPACE: usize = 1 + (4 + 10) + (1 + 32) + (4 + 20) + 1 + (32 * 5 + 4) + (8 * 6);
    // const SEEDS = [owner.key.as_ref(), CLIENT_SEED.as_bytes(), &client.job_count.to_le_bytes()]
}
