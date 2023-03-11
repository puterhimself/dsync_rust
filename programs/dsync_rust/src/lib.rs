use anchor_lang::prelude::*;
use anchor_spl::token::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

const CLIENT_SEED: &str = "DSYNC_CLIENT";
const VAULT_SEED: &str = "DSYNC_VAULT";
const JOB_SEED: &str = "DSYNC_JOB";

#[program]
pub mod dsync_rust {
    use super::*;

    pub fn initialize_client(ctx: Context<InitializeClient>) -> Result<()> {
        let client = &mut ctx.accounts.client;
        client.owner = *ctx.accounts.owner.key;
        client.bump = *ctx.bumps.get(CLIENT_SEED).unwrap();
        client.job_count = 0;
        Ok(())
    }

    pub fn initialize_job(ctx: Context<InitializeJob>) -> Result<()> {
        let task = &mut ctx.accounts.task;
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
        seeds = [JOB_SEED.as_bytes().as_ref(), owner.key.as_ref(), &client.job_count.to_le_bytes()],
        bump,
        payer = owner,
        space = Job::SPACE
    )]
    pub task: Box<Account<'info, Job>>,
    #[account(
        init,
        seeds = [VAULT_SEED.as_bytes().as_ref(), &task.job_id.as_ref()],
        bump,
        payer = owner,
        token::mint = currency,
        token::authority = task
    )]
    pub vault: Box<Account<'info, TokenAccount>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub system_program: Program<'info, System>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct Client {
    pub owner: Pubkey,
    pub bump: u8,
    pub job_count: u64,
}

impl Client {
    const SPACE: usize = 32 + 1 + 8 + 8;
}

#[account]
pub struct Job {
    pub bump: u8,
    pub index: u64,
    pub client: Pubkey,
    // pub client_account: Pubkey,
    pub winner: Pubkey,
    pub validator: Pubkey,
    pub job_id: String,
    pub submission_count: u64,
    pub state: JobState,
    pub job_description: String,
    pub publish_date: u64,
    pub deadline: u64,
    pub price: u64,
    pub is_native: bool,
    pub currency: Pubkey,
    pub client_token_account: Pubkey,
    pub validator_token_account: Pubkey,
    pub winner_token_account: Pubkey,
    pub valut_token_account: Pubkey,
}

impl Job {
    // const SPACE: usize = 1 + 8 + 32 + 32 + 32 + 32 + (4 + 10) + 8 + (1 + 32) + (4 + 20) + 8 + 8 + 8 + 1 + (32 * 5) + 8;
    const SPACE: usize = 1 + (4 + 10) + (1 + 32) + (4 + 20) + 1 + (32 * 5 + 4) + (8 * 6);
    // const SEEDS = [owner.key.as_ref(), CLIENT_SEED.as_bytes(), &client.job_count.to_le_bytes()]
}

#[derive(Clone, Debug, PartialEq, AnchorSerialize, AnchorDeserialize, Copy)]
pub enum JobState {
    PENDING,
    PUBLISHED,
    ACTIVE,
    VALIDATED,
    COMPLETED,
    CANCELED,
}

impl Default for JobState {
    fn default() -> Self {
        JobState::PENDING
    }
}
