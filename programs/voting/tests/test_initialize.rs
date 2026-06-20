#![cfg(feature = "test-sbf")]

use anchor_lang::prelude::*;
use anchor_lang::system_program;
use solana_program_test::*;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
    instruction::{AccountMeta, Instruction},
    clock::Clock,
    sysvar,
};
use voting::{
    accounts::{InitPoll, InitializeCandidate, Vote},
    instruction as voting_ix,
    PollAccount,
    CandidateAccount,
};

fn voting_program_test() -> ProgramTest {
    ProgramTest::new("voting", voting::ID, processor!(voting::entry))
}

fn poll_pda(poll_id: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"poll", &poll_id.to_le_bytes()],
        &voting::ID,
    )
}

fn candidate_pda(poll_id: u64, candidate: &str) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"poll", &poll_id.to_le_bytes(), candidate.as_bytes()],
        &voting::ID,
    )
}

// --- init_poll tests ---

#[tokio::test]
async fn test_init_poll_success() {
    let mut ctx = voting_program_test().start_with_context().await;
    let payer = ctx.payer.insecure_clone();

    let poll_id: u64 = 1;
    let (poll_pda, _bump) = poll_pda(poll_id);

    let start: u64 = 0;
    let end: u64 = u64::MAX;
    let name = "Test Poll".to_string();
    let description = "A simple test poll".to_string();

    let ix = Instruction {
        program_id: voting::ID,
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(poll_pda, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: voting_ix::InitPoll {
            poll_id,
            start,
            end,
            name: name.clone(),
            description: description.clone(),
        }
        .data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[&payer],
        ctx.last_blockhash,
    );

    ctx.banks_client.process_transaction(tx).await.unwrap();

    let account = ctx.banks_client.get_account(poll_pda).await.unwrap().unwrap();
    let poll: PollAccount =
        PollAccount::try_deserialize(&mut account.data.as_ref()).unwrap();

    assert_eq!(poll.poll_name, name);
    assert_eq!(poll.poll_description, description);
    assert_eq!(poll.poll_voting_start, start);
    assert_eq!(poll.poll_voting_end, end);
    assert_eq!(poll.poll_option_index, 0);
}

#[tokio::test]
async fn test_init_poll_duplicate_fails() {
    let mut ctx = voting_program_test().start_with_context().await;
    let payer = ctx.payer.insecure_clone();

    let poll_id: u64 = 99;
    let (poll_pda, _) = poll_pda(poll_id);

    let make_ix = || Instruction {
        program_id: voting::ID,
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(poll_pda, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: voting_ix::InitPoll {
            poll_id,
            start: 0,
            end: u64::MAX,
            name: "Dup".to_string(),
            description: "dup".to_string(),
        }
        .data(),
    };

    let tx1 = Transaction::new_signed_with_payer(
        &[make_ix()],
        Some(&payer.pubkey()),
        &[&payer],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx1).await.unwrap();

    ctx.last_blockhash = ctx.banks_client.get_latest_blockhash().await.unwrap();

    let tx2 = Transaction::new_signed_with_payer(
        &[make_ix()],
        Some(&payer.pubkey()),
        &[&payer],
        ctx.last_blockhash,
    );

    let result = ctx.banks_client.process_transaction(tx2).await;
    assert!(result.is_err(), "Re-initializing the same poll PDA should fail");
}

// --- initialize_candidate tests ---

#[tokio::test]
async fn test_initialize_candidate_success() {
    let mut ctx = voting_program_test().start_with_context().await;
    let payer = ctx.payer.insecure_clone();

    let poll_id: u64 = 2;
    let (poll_pda, _) = poll_pda(poll_id);
    let candidate_name = "Alice".to_string();
    let (cand_pda, _) = candidate_pda(poll_id, &candidate_name);

    // Create poll first
    let init_ix = Instruction {
        program_id: voting::ID,
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(poll_pda, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: voting_ix::InitPoll {
            poll_id,
            start: 0,
            end: u64::MAX,
            name: "Poll 2".to_string(),
            description: "desc".to_string(),
        }
        .data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[init_ix],
        Some(&payer.pubkey()),
        &[&payer],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.unwrap();
    ctx.last_blockhash = ctx.banks_client.get_latest_blockhash().await.unwrap();

    // Add candidate
    let cand_ix = Instruction {
        program_id: voting::ID,
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(poll_pda, false),
            AccountMeta::new(cand_pda, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: voting_ix::InitializeCandidate {
            poll_id,
            candidate: candidate_name.clone(),
        }
        .data(),
    };

    let tx2 = Transaction::new_signed_with_payer(
        &[cand_ix],
        Some(&payer.pubkey()),
        &[&payer],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx2).await.unwrap();

    let cand_account = ctx.banks_client.get_account(cand_pda).await.unwrap().unwrap();
    let cand: CandidateAccount =
        CandidateAccount::try_deserialize(&mut cand_account.data.as_ref()).unwrap();

    assert_eq!(cand.candidate_name, candidate_name);
    assert_eq!(cand.candidate_vote, 0);

    let poll_account = ctx.banks_client.get_account(poll_pda).await.unwrap().unwrap();
    let poll: PollAccount =
        PollAccount::try_deserialize(&mut poll_account.data.as_ref()).unwrap();

    assert_eq!(poll.poll_option_index, 1);
}

#[tokio::test]
async fn test_initialize_two_candidates_increments_index() {
    let mut ctx = voting_program_test().start_with_context().await;
    let payer = ctx.payer.insecure_clone();

    let poll_id: u64 = 3;
    let (poll_pda, _) = poll_pda(poll_id);

    let init_ix = Instruction {
        program_id: voting::ID,
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(poll_pda, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: voting_ix::InitPoll {
            poll_id,
            start: 0,
            end: u64::MAX,
            name: "Poll 3".to_string(),
            description: "desc".to_string(),
        }
        .data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[init_ix],
        Some(&payer.pubkey()),
        &[&payer],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.unwrap();

    for (i, name) in ["Alice", "Bob"].iter().enumerate() {
        ctx.last_blockhash = ctx.banks_client.get_latest_blockhash().await.unwrap();
        let (cand_pda, _) = candidate_pda(poll_id, name);

        let cand_ix = Instruction {
            program_id: voting::ID,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(poll_pda, false),
                AccountMeta::new(cand_pda, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: voting_ix::InitializeCandidate {
                poll_id,
                candidate: name.to_string(),
            }
            .data(),
        };

        let tx = Transaction::new_signed_with_payer(
            &[cand_ix],
            Some(&payer.pubkey()),
            &[&payer],
            ctx.last_blockhash,
        );
        ctx.banks_client.process_transaction(tx).await.unwrap();

        let poll_account = ctx.banks_client.get_account(poll_pda).await.unwrap().unwrap();
        let poll: PollAccount =
            PollAccount::try_deserialize(&mut poll_account.data.as_ref()).unwrap();
        assert_eq!(poll.poll_option_index, (i + 1) as u64);
    }
}

// --- vote tests ---

async fn setup_poll_with_candidates(
    ctx: &mut ProgramTestContext,
    poll_id: u64,
    start: u64,
    end: u64,
    candidates: &[&str],
) {
    let payer = ctx.payer.insecure_clone();
    let (poll_pda, _) = poll_pda(poll_id);

    let init_ix = Instruction {
        program_id: voting::ID,
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(poll_pda, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: voting_ix::InitPoll {
            poll_id,
            start,
            end,
            name: format!("Poll {}", poll_id),
            description: "desc".to_string(),
        }
        .data(),
    };

    ctx.last_blockhash = ctx.banks_client.get_latest_blockhash().await.unwrap();
    let tx = Transaction::new_signed_with_payer(
        &[init_ix],
        Some(&payer.pubkey()),
        &[&payer],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.unwrap();

    for name in candidates {
        let (cand_pda, _) = candidate_pda(poll_id, name);
        ctx.last_blockhash = ctx.banks_client.get_latest_blockhash().await.unwrap();

        let cand_ix = Instruction {
            program_id: voting::ID,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(poll_pda, false),
                AccountMeta::new(cand_pda, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: voting_ix::InitializeCandidate {
                poll_id,
                candidate: name.to_string(),
            }
            .data(),
        };

        let tx = Transaction::new_signed_with_payer(
            &[cand_ix],
            Some(&payer.pubkey()),
            &[&payer],
            ctx.last_blockhash,
        );
        ctx.banks_client.process_transaction(tx).await.unwrap();
    }
}

#[tokio::test]
async fn test_vote_increments_vote_count() {
    let mut ctx = voting_program_test().start_with_context().await;
    let payer = ctx.payer.insecure_clone();

    let poll_id: u64 = 10;
    let candidate_name = "Alice";

    setup_poll_with_candidates(&mut ctx, poll_id, 0, u64::MAX, &[candidate_name]).await;

    let (poll_pda, _) = poll_pda(poll_id);
    let (cand_pda, _) = candidate_pda(poll_id, candidate_name);

    ctx.last_blockhash = ctx.banks_client.get_latest_blockhash().await.unwrap();

    let vote_ix = Instruction {
        program_id: voting::ID,
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(poll_pda, false),
            AccountMeta::new(cand_pda, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: voting_ix::Vote {
            poll_id,
            candidate: candidate_name.to_string(),
        }
        .data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[vote_ix],
        Some(&payer.pubkey()),
        &[&payer],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.unwrap();

    let cand_account = ctx.banks_client.get_account(cand_pda).await.unwrap().unwrap();
    let cand: CandidateAccount =
        CandidateAccount::try_deserialize(&mut cand_account.data.as_ref()).unwrap();

    assert_eq!(cand.candidate_vote, 1);
}

#[tokio::test]
async fn test_vote_multiple_times_accumulates() {
    let mut ctx = voting_program_test().start_with_context().await;
    let payer = ctx.payer.insecure_clone();

    let poll_id: u64 = 11;
    let candidate_name = "Bob";

    setup_poll_with_candidates(&mut ctx, poll_id, 0, u64::MAX, &[candidate_name]).await;

    let (poll_pda, _) = poll_pda(poll_id);
    let (cand_pda, _) = candidate_pda(poll_id, candidate_name);

    for _ in 0..3 {
        ctx.last_blockhash = ctx.banks_client.get_latest_blockhash().await.unwrap();
        let vote_ix = Instruction {
            program_id: voting::ID,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(poll_pda, false),
                AccountMeta::new(cand_pda, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: voting_ix::Vote {
                poll_id,
                candidate: candidate_name.to_string(),
            }
            .data(),
        };

        let tx = Transaction::new_signed_with_payer(
            &[vote_ix],
            Some(&payer.pubkey()),
            &[&payer],
            ctx.last_blockhash,
        );
        ctx.banks_client.process_transaction(tx).await.unwrap();
    }

    let cand_account = ctx.banks_client.get_account(cand_pda).await.unwrap().unwrap();
    let cand: CandidateAccount =
        CandidateAccount::try_deserialize(&mut cand_account.data.as_ref()).unwrap();

    assert_eq!(cand.candidate_vote, 3);
}

#[tokio::test]
async fn test_vote_after_end_fails() {
    let mut ctx = voting_program_test().start_with_context().await;
    let payer = ctx.payer.insecure_clone();

    let poll_id: u64 = 12;
    let candidate_name = "Alice";

    // Set voting end in the past (unix timestamp 1 = way in the past)
    setup_poll_with_candidates(&mut ctx, poll_id, 0, 1, &[candidate_name]).await;

    let (poll_pda, _) = poll_pda(poll_id);
    let (cand_pda, _) = candidate_pda(poll_id, candidate_name);

    ctx.last_blockhash = ctx.banks_client.get_latest_blockhash().await.unwrap();

    let vote_ix = Instruction {
        program_id: voting::ID,
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(poll_pda, false),
            AccountMeta::new(cand_pda, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: voting_ix::Vote {
            poll_id,
            candidate: candidate_name.to_string(),
        }
        .data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[vote_ix],
        Some(&payer.pubkey()),
        &[&payer],
        ctx.last_blockhash,
    );

    let result = ctx.banks_client.process_transaction(tx).await;
    assert!(result.is_err(), "Voting after end time should fail with VotingEnded error");
}

#[tokio::test]
async fn test_vote_before_start_fails() {
    let mut ctx = voting_program_test().start_with_context().await;
    let payer = ctx.payer.insecure_clone();

    let poll_id: u64 = 13;
    let candidate_name = "Alice";

    // Set voting start far in the future
    let far_future: u64 = u64::MAX - 1;
    setup_poll_with_candidates(&mut ctx, poll_id, far_future, u64::MAX, &[candidate_name]).await;

    let (poll_pda, _) = poll_pda(poll_id);
    let (cand_pda, _) = candidate_pda(poll_id, candidate_name);

    ctx.last_blockhash = ctx.banks_client.get_latest_blockhash().await.unwrap();

    let vote_ix = Instruction {
        program_id: voting::ID,
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(poll_pda, false),
            AccountMeta::new(cand_pda, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: voting_ix::Vote {
            poll_id,
            candidate: candidate_name.to_string(),
        }
        .data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[vote_ix],
        Some(&payer.pubkey()),
        &[&payer],
        ctx.last_blockhash,
    );

    let result = ctx.banks_client.process_transaction(tx).await;
    assert!(result.is_err(), "Voting before start time should fail with VotingNotStarted error");
}

#[tokio::test]
async fn test_vote_does_not_change_other_candidates() {
    let mut ctx = voting_program_test().start_with_context().await;
    let payer = ctx.payer.insecure_clone();

    let poll_id: u64 = 14;
    setup_poll_with_candidates(&mut ctx, poll_id, 0, u64::MAX, &["Alice", "Bob"]).await;

    let (poll_pda, _) = poll_pda(poll_id);
    let (alice_pda, _) = candidate_pda(poll_id, "Alice");
    let (bob_pda, _) = candidate_pda(poll_id, "Bob");

    // Vote for Alice only
    ctx.last_blockhash = ctx.banks_client.get_latest_blockhash().await.unwrap();

    let vote_ix = Instruction {
        program_id: voting::ID,
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(poll_pda, false),
            AccountMeta::new(alice_pda, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: voting_ix::Vote {
            poll_id,
            candidate: "Alice".to_string(),
        }
        .data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[vote_ix],
        Some(&payer.pubkey()),
        &[&payer],
        ctx.last_blockhash,
    );
    ctx.banks_client.process_transaction(tx).await.unwrap();

    let alice_account = ctx.banks_client.get_account(alice_pda).await.unwrap().unwrap();
    let alice: CandidateAccount =
        CandidateAccount::try_deserialize(&mut alice_account.data.as_ref()).unwrap();

    let bob_account = ctx.banks_client.get_account(bob_pda).await.unwrap().unwrap();
    let bob: CandidateAccount =
        CandidateAccount::try_deserialize(&mut bob_account.data.as_ref()).unwrap();

    assert_eq!(alice.candidate_vote, 1);
    assert_eq!(bob.candidate_vote, 0);
}