use anchor_lang::prelude::*;



declare_id!("DqUMdzRHwMPdKZYLsh5bisRACnGkdhkUTQLCw7AtqDUP");

#[program]
pub mod voting {
    use super::*;

    pub fn init_poll (ctx: Context<InitPoll>, _poll_id: u64, start: u64, end: u64, name: String, description: String) -> Result<()> {
        let poll = & mut ctx.accounts.poll_account;
        poll.poll_description = description;
        poll.poll_name = name;
        poll.poll_voting_end = end;
        poll.poll_voting_start = start;
        Ok(())
    }

    pub fn initialize_candidate (ctx: Context<InitializeCandidate>,_poll_id: u64, candidate: String) -> Result<()> {
        ctx.accounts.candidate_account.candidate_name = candidate;
        ctx.accounts.poll_account.poll_option_index += 1;
        Ok(())
    }

    pub fn vote (ctx: Context<Vote>, _poll_id: u64, _candidate: String) -> Result<()> {
        let candidate = &mut ctx.accounts.candidate_account;
        let current_time = Clock::get()?.unix_timestamp;

        if current_time > (ctx.accounts.poll_account.poll_voting_end as i64) {
            return Err(ErrorCode::VotingEnded.into());
        }

        if current_time < (ctx.accounts.poll_account.poll_voting_start as i64) {
            return Err(ErrorCode::VotingNotStarted.into());
        }

        candidate.candidate_vote += 1;
        Ok(())

    }
}


#[derive(Accounts)]
#[instruction(poll_id: u64)]
pub struct InitPoll<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init,
        payer = signer,
        space = 8 + PollAccount::INIT_SPACE,
        seeds = [b"poll".as_ref(),poll_id.to_le_bytes().as_ref()],
        bump
    )]
    pub poll_account: Account<'info, PollAccount>,
    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
#[instruction(poll_id: u64, candidate: String)]
pub struct InitializeCandidate<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"poll".as_ref(),poll_id.to_le_bytes().as_ref()],
        bump
    )]
    pub poll_account: Account<'info, PollAccount>,

    #[account(
        init,
        payer = signer,
        space = 8 + CandidateAccount::INIT_SPACE,
        seeds = [b"poll".as_ref(),poll_id.to_le_bytes().as_ref(), candidate.as_ref()],
        bump
    )]
    pub candidate_account: Account<'info, CandidateAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(poll_id: u64, candidate: String)]
pub struct Vote<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"poll".as_ref(),poll_id.to_le_bytes().as_ref()],
        bump
    )]
    pub poll_account: Account<'info, PollAccount>,

    #[account(
        mut,
        seeds = [b"poll".as_ref(),poll_id.to_le_bytes().as_ref(), candidate.as_ref()],
        bump
    )]
    pub candidate_account: Account<'info, CandidateAccount>,
    pub system_program: Program<'info, System>,
}


#[account]
#[derive(InitSpace)]
pub struct PollAccount {
    #[max_len(32)]
    pub poll_name: String,
    #[max_len(128)]
    pub poll_description: String,
    pub poll_voting_start: u64,
    pub poll_voting_end: u64,
    pub poll_option_index: u64,
}


#[account]
#[derive(InitSpace)]
pub struct CandidateAccount {
    #[max_len(32)]
    pub candidate_name: String,
    pub candidate_vote: u64,
}


#[error_code]
pub enum ErrorCode {
    #[msg(Voting has not started yet)]
    VotingNotStarted,
    #[msg("Voting has ended")]
    VotingEnded,
}