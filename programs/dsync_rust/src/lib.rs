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
const SUBMISSION_SEED: &str = "DSYNC_SUBMISSION";

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

    pub fn initialize_client(ctx: Context<InitializeClient>, _client: Pubkey) -> Result<()> {
        let client = &mut ctx.accounts.client;
        client.owner = _client; // added for convinience otherwise just bump with seed is enough.
        client.job_count = 0;
        client.bump = *ctx.bumps.get(CLIENT_SEED).unwrap();
        Ok(())
    }

    pub fn initialize_job(
        ctx: Context<InitializeJob>,
        _description: String,
        _price: u64,
        _is_native: bool,
        _deadline: i64,
    ) -> Result<()> {
        let _client = &mut ctx.accounts.client;
        let _task = &mut ctx.accounts.job;
        let _valut = &mut ctx.accounts.vault;

        _client.job_count += 1;

        _task.bump = *ctx.bumps.get(JOB_SEED).unwrap();
        _task.index = _client.job_count;
        _task.client = _client.owner;

        _task.state = JobState::PENDING;
        _task.job_description = _description;

        _task.validator = _client.owner;
        _task.submission_count = 0;
        _task.price = _price;
        _task.currency = ctx.accounts.currency.to_account_info().key.clone();
        _task.is_native = _is_native;

        _task.publish_date = Clock::get().unwrap().unix_timestamp;
        _task.deadline = _deadline;
        _task.vault_token_account = ctx.accounts.vault.to_account_info().key.clone();

        let (vault_authority, _vault_authority_bump) =
            Pubkey::find_program_address(&[AUTHORITY_SEED.as_bytes()], ctx.program_id);

        token::set_authority(
            ctx.accounts.set_authority(),
            AuthorityType::AccountOwner,
            Some(vault_authority),
        )?;
        // _task.client_token_account = ctx
        //     .accounts
        //     .client_token_account
        //     .to_account_info()
        //     .key
        //     .clone();

        Ok(())
    }

    pub fn publish_job(ctx: Context<PublishJob>) -> Result<()> {
        let _task = &mut ctx.accounts.job;
        _task.state = JobState::PUBLISHED;

        token::transfer(
            ctx.accounts.into_transfer_to_pda_context(),
            ctx.accounts.job.price,
        )?;


        Ok(())
    }

    pub fn cancel_job(ctx: Context<CancelJob>) -> Result<()> {
        // let _task = &mut ctx.accounts.job;
        let (_vault_authority, vault_authority_bump) = Pubkey::find_program_address(&[AUTHORITY_SEED.as_bytes()], ctx.program_id);
        let bumps = &[vault_authority_bump];
        let authority_seed = &[AUTHORITY_SEED.as_bytes().as_ref(), bumps][..];
        let authority_seeds = authority_seed;



        if ctx.accounts.job.state == JobState::PUBLISHED || ctx.accounts.job.state == JobState::PENDING{
            token::transfer(
                ctx.accounts.into_transfer_to_client_context()
                .with_signer(&[&authority_seeds[..]]),
                ctx.accounts.job.price,
            )?;
        }
        else{
            panic!("DSYNC_ERROR: Cannot cancel an active job")
        }
        ctx.accounts.job.state = JobState::CANCELED;

        // token::set_authority(
        //     ctx.accounts.into_set_authority_context(),
        //     AuthorityType::AccountOwner,
        //     Some(*ctx.program_id),
        // )?;
        Ok(())
    }

    pub fn start_job(ctx: Context<StartJob>, _worker: Pubkey) -> Result<()> {
        let _task = &mut ctx.accounts.job;
        let _sub = &mut ctx.accounts.submission;

        if _task.state == JobState::CANCELED {
            panic!("DSYNC_ERROR: Cannot start a canceled job")
        }
        if _task.state == JobState::COMPLETED  || _task.state == JobState::VALIDATED{
            panic!("DSYNC_ERROR: Job Done!, try starting another job")
        }
        if _task.state != JobState::ACTIVE {
            _task.state = JobState::ACTIVE;
        }

        _sub.bump = *ctx.bumps.get(SUBMISSION_SEED).unwrap();
        _sub.job = _task.to_account_info().key.clone();
        _sub.worker = _worker;
        _sub.submission_started = Clock::get().unwrap().unix_timestamp;

        Ok(())
    }

    pub fn publish_submission(ctx: Context<PublishSubmission>, _hash: String) -> Result<()> {
        let _sub = &mut ctx.accounts.submission;
        
        _sub.submission_date = Clock::get().unwrap().unix_timestamp;
        _sub.submission_hash = _hash;


        Ok(())
    }

    pub fn validate_submission(ctx: Context<ValidateSubmission>) -> Result<()> {
        let _task = &mut ctx.accounts.job;
        let _sub = &mut ctx.accounts.submission;

        if _task.state == JobState::CANCELED {
            panic!("DSYNC_ERROR: Cannot Validate a canceled job")
        }
        if _task.state == JobState::COMPLETED  || _task.state == JobState::VALIDATED{
            panic!("DSYNC_ERROR: Job Done!, try starting another job")
        }

        _task.state = JobState::VALIDATED;
        _task.winner = _sub.worker;
        _task.winning_submission = _sub.to_account_info().key.clone();

        Ok(())
    }

    pub fn calim_reward(ctx: Context<ClaimReward>) -> Result<()> {
        let (_vault_authority, vault_authority_bump) = Pubkey::find_program_address(&[AUTHORITY_SEED.as_bytes()], ctx.program_id);
        let bumps = &[vault_authority_bump];
        let authority_seed = &[AUTHORITY_SEED.as_bytes().as_ref(), bumps][..];
        let authority_seeds = authority_seed;
        // let _task = &mut ctx.accounts.job;
        // let _sub = &mut ctx.accounts.submission;

        // _task.state = JobState::COMPLETED;
        // _sub.is_claimed = true;
        // let seeds = vec![
        //     VAULT_SEED.as_bytes(),
        //     ctx.accounts.job.to_account_info().key.clone().as_ref(),
        // ];

        token::transfer(
            ctx.accounts
            .into_transfer_to_worker_context()
            .with_signer(&[&authority_seeds[..]]),
            ctx.accounts.job.price,
        )?;
        Ok(())
    }

}

#[derive(Accounts)]
#[instruction(_client: Pubkey)]
pub struct InitializeClient<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init,
        seeds = [CLIENT_SEED.as_bytes().as_ref(), _client.as_ref()],
        bump,
        payer = signer,
        space = Client::SPACE
    )]
    pub client: Box<Account<'info, Client>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
// #[instruction(currency: Pubkey)]
pub struct InitializeJob<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut, seeds = [CLIENT_SEED.as_bytes().as_ref(), client.owner.as_ref()], bump=client.bump)]
    pub client: Box<Account<'info, Client>>,
    #[account(
        init,
        seeds = [JOB_SEED.as_bytes().as_ref(), client.owner.as_ref(), &client.job_count.to_le_bytes()],
        bump,
        payer = signer,
        space = Job::SPACE,
    )]
    pub job: Box<Account<'info, Job>>,
    #[account(
        init,
        seeds = [VAULT_SEED.as_bytes().as_ref(), &job.to_account_info().key.clone().as_ref()],
        bump,
        payer = signer,
        token::mint = currency,
        token::authority = signer,
    )]
    pub vault: Box<Account<'info, TokenAccount>>,

    #[account(constraint = vault.mint == currency.to_account_info().key.clone())]
    pub currency: Account<'info, Mint>,
    // pub client_token_account: Box<Account<'info, TokenAccount>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub system_program: Program<'info, System>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct PublishJob<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut, seeds = [JOB_SEED.as_bytes().as_ref(), job.client.as_ref(), &job.index.to_le_bytes()], bump=job.bump)]
    pub job: Box<Account<'info, Job>>,
    #[account(
        mut, 
        seeds = [VAULT_SEED.as_bytes().as_ref(), &job.to_account_info().key.clone().as_ref()], 
        bump=job.vault_bump,
        constraint = vault.mint == currency.to_account_info().key.clone()
    )]
    pub vault: Box<Account<'info, TokenAccount>>,
    
    #[account(
        mut, 
        constraint = 
            client_token_account.mint == vault.mint
    )]
    pub client_token_account: Box<Account<'info, TokenAccount>>,
    pub currency: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CancelJob<'info> {
    #[account(mut, constraint = job.client == *signer.key)]
    pub signer: Signer<'info>,
    #[account(mut, seeds = [JOB_SEED.as_bytes().as_ref(), job.client.as_ref(), &job.index.to_le_bytes()], bump=job.bump)]
    pub job: Box<Account<'info, Job>>,
    #[account(
        mut, 
        seeds = [VAULT_SEED.as_bytes().as_ref(), &job.to_account_info().key.clone().as_ref()], 
        bump=job.vault_bump,
        constraint = vault.mint == currency.to_account_info().key.clone()
    )]
    pub vault: Box<Account<'info, TokenAccount>>,
    
    #[account(
        mut, 
        constraint = 
        client_token_account.mint == vault.mint
    )]
    pub client_token_account: Box<Account<'info, TokenAccount>>,
    pub currency: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(worker: Pubkey)]
pub struct StartJob<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut, seeds = [JOB_SEED.as_bytes().as_ref(), job.client.as_ref(), &job.index.to_le_bytes()], bump=job.bump)]
    pub job: Box<Account<'info, Job>>,
    #[account(
        init,
        seeds = [SUBMISSION_SEED.as_bytes().as_ref(), worker.as_ref(), &job.to_account_info().key.clone().as_ref()],
        bump,
        payer = signer,
        space = Submission::SPACE
    )]
    pub submission: Box<Account<'info, Submission>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PublishSubmission<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut, seeds = [SUBMISSION_SEED.as_bytes().as_ref(), &submission.worker.as_ref(), &submission.job.as_ref()], bump=submission.bump)]
    pub submission: Box<Account<'info, Submission>>,
}

#[derive(Accounts)]
pub struct ValidateSubmission<'info> {
    #[account(mut, constraint = job.validator == *signer.key)]
    pub signer: Signer<'info>,
    #[account(mut, seeds = [SUBMISSION_SEED.as_bytes().as_ref(), &submission.worker.as_ref(), &submission.job.as_ref()], bump=submission.bump)]
    pub submission: Box<Account<'info, Submission>>,
    #[account(mut, seeds = [JOB_SEED.as_bytes().as_ref(), job.client.as_ref(), &job.index.to_le_bytes()], bump=job.bump)]
    pub job: Box<Account<'info, Job>>,
}

#[derive(Accounts)]
pub struct ClaimReward<'info> {
    #[account(mut, constraint = submission.worker == *signer.key)]
    pub signer: Signer<'info>,

    #[account(mut, 
        seeds = [JOB_SEED.as_bytes().as_ref(), job.client.as_ref(), &job.index.to_le_bytes()], 
        bump=job.bump,
        constraint = job.state == JobState::VALIDATED
    )]
    pub job: Box<Account<'info, Job>>,
    
    #[account(mut, 
        seeds = [SUBMISSION_SEED.as_bytes().as_ref(), &submission.worker.as_ref(), &submission.job.as_ref()], 
        bump=submission.bump,
        constraint = submission.job == job.to_account_info().key.clone(),
        constraint = submission.is_winner == true
    )]
    pub submission: Box<Account<'info, Submission>>,
    
    #[account(
        mut, 
        seeds = [VAULT_SEED.as_bytes().as_ref(), submission.job.as_ref()], 
        bump=job.vault_bump,
        constraint = vault.mint == job.currency.clone()
    )]
    pub vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut, 
        constraint = winning_token_account.mint == vault.mint,
        constraint = winning_token_account.owner == signer.key.clone()
    )]
    pub winning_token_account: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct Client {
    pub bump: u8,
    pub owner: Pubkey,
    pub job_count: u64,
    /*
     *EXTRA VALUES that can be added
     * pub client_name: String,
     * pub total_jobs: u64,
     * pub total_staked: u128,
     */
}

#[account]
pub struct Job {
    pub bump: u8,       // 1 byte
    pub index: u64,     // 8 bytes
    pub client: Pubkey, // 32 bytes
    // pub client_account: Pubkey,
    // pub job_id: String, // 4 + 1 byte = 1 char | 4+(length of string) = 14 bytes
    pub state: JobState,         // 1 + 4 + 9 = 14 bytes
    pub job_description: String, // 4 + 1 byte = 1 char | 4+(length of string) = 54 bytes

    pub winner: Pubkey,        // 32 bytes
    pub validator: Pubkey,     // 32 bytes
    pub submission_count: u64, // 8 bytes
    pub winning_submission: Pubkey, // 32 bytes

    pub price: u64,       // 8 bytes
    pub currency: Pubkey, // 32 bytes
    pub is_native: bool,  // 1 byte

    pub publish_date: i64, // 8 bytes
    pub deadline: i64,     // 8 bytes

    pub vault_token_account: Pubkey, // 32 bytes
    pub vault_bump: u8,             // 1 byte
    // pub client_token_account: Pubkey,   // 32 bytes
    // pub validator_token_account: Pubkey,    // 32 bytes
    // pub winner_token_account: Pubkey,  // 32 bytes
}

#[account]
pub struct Submission {
    pub bump: u8,
    pub job: Pubkey,
    pub worker: Pubkey,
    pub submission_started: i64,
    // pub submission_id: String,
    pub submission_hash: String,
    pub submission_date: i64,

    pub is_validated: bool,
    pub is_winner: bool,
}

impl Default for JobState {
    fn default() -> Self {
        JobState::PENDING
    }
}

impl<'info> InitializeJob<'info> {
    fn set_authority(&self) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_accounts = SetAuthority {
            account_or_mint: self.vault.to_account_info(),
            current_authority: self.signer.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
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
}

impl<'info> CancelJob<'info> {
    fn into_transfer_to_client_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.client_token_account.to_account_info(),
            authority: self.vault.to_account_info().clone(),
        };
        // let seeds = vec![
        //     VAULT_SEED.as_bytes().as_ref(),
        //     self.job.to_account_info().key.clone().as_ref(),
        // ];
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
        // .with_signer(
        //     &[
        //         &[
        //             VAULT_SEED.as_bytes().as_ref(),
        //             self.job.to_account_info().key.clone().as_ref(), 
        //             &[self.job.vault_bump]
        //         ]
        //     ]
        // )
    }
}

impl<'info> ClaimReward<'info> {
    fn into_transfer_to_worker_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.winning_token_account.to_account_info(),
            authority: self.vault.to_account_info(),
        };
        // let seeds= vec![
        //     VAULT_SEED.as_bytes(),
        //     self.submission.job.as_ref(),
        // ];
        // let seeds = vec![seeds.as_slice()];
        CpiContext::new(
            self.token_program.to_account_info(),
            cpi_accounts,
        )
    }
}

impl Client {
    const SPACE: usize = 32 + 1 + 8 + 8;
}

impl Job {
    const SPACE: usize = 8 + (6 * 32) + (8 * 5) + 14 + 14 + 54 + 1 + 1 +1 + 10;
    // const SPACE: usize = 1 + 8 + 32 + 32 + 32 + 32 + (4 + 10) + 8 + (1 + 32) + (4 + 20) + 8 + 8 + 8 + 1 + (32 * 5) + 8;
    // const SPACE: usize = 1 + (4 + 10) + (1 + 32) + (4 + 20) + 1 + (32 * 5 + 4) + (8 * 6);
    // const SEEDS = [owner.key.as_ref(), CLIENT_SEED.as_bytes().as_ref(), &client.job_count.to_le_bytes()]
}

impl Submission {
    const SPACE: usize = 1 + 32 + 32 + (4 + 50) + 8;
}
