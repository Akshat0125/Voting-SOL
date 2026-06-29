use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::instruction::Instruction,
        InstructionData,
        ToAccountMetas,
    },
    litesvm::LiteSVM,
    solana_clock::Clock,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

// ─── setup ────────────────────────────────────────────────────────────────
fn setup() -> (LiteSVM, Keypair) {
    let program_id = voting::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/voting.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();
    (svm, payer)
}

// ─── send helpers ─────────────────────────────────────────────────────────
fn send_ix(
    svm: &mut LiteSVM,
    payer: &Keypair,
    instruction: Instruction,
) -> litesvm::types::TransactionMetadata {
    svm.expire_blockhash();
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(
        &[instruction],
        Some(&payer.pubkey()),
        &blockhash,
    );
    let tx = VersionedTransaction::try_new(
        VersionedMessage::Legacy(msg),
        &[payer],
    ).unwrap();
    svm.send_transaction(tx).unwrap()
}

fn send_ix_expect_fail(
    svm: &mut LiteSVM,
    payer: &Keypair,
    instruction: Instruction,
) -> litesvm::types::FailedTransactionMetadata {
    svm.expire_blockhash();
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(
        &[instruction],
        Some(&payer.pubkey()),
        &blockhash,
    );
    let tx = VersionedTransaction::try_new(
        VersionedMessage::Legacy(msg),
        &[payer],
    ).unwrap();
    match svm.send_transaction(tx) {
        Ok(_) => panic!("Expected failure but transaction succeeded"),
        Err(e) => e,
    }
}

// ─── PDA finders ──────────────────────────────────────────────────────────
fn find_poll_pda(poll_id: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"poll",
            poll_id.to_le_bytes().as_ref(), // must match poll_id.to_le_bytes()
        ],
        &voting::id(),
    )
}

fn find_candidate_pda(poll_id: u64, candidate: &str) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"poll",
            poll_id.to_le_bytes().as_ref(),
            candidate.as_bytes(), // must match candidate.as_ref()
        ],
        &voting::id(),
    )
}

// ─── instruction builders ─────────────────────────────────────────────────
fn make_init_poll_ix(
    poll_id: u64,
    start: u64,
    end: u64,
    name: String,
    description: String,
    poll_pda: Pubkey,
    signer: Pubkey,
) -> Instruction {
    Instruction::new_with_bytes(
        voting::id(),
        &voting::instruction::InitPoll {
            _poll_id: poll_id,
            start,
            end,
            name,
            description,
        }.data(),
        voting::accounts::InitPoll {
            signer,
            poll_account: poll_pda,
            system_program: anchor_lang::solana_program::system_program::id(),
        }.to_account_metas(None),
    )
}

fn make_init_candidate_ix(
    poll_id: u64,
    candidate: String,
    poll_pda: Pubkey,
    candidate_pda: Pubkey,
    signer: Pubkey,
) -> Instruction {
    Instruction::new_with_bytes(
        voting::id(),
        &voting::instruction::InitializeCandidate {
            _poll_id: poll_id,
            candidate,
        }.data(),
        voting::accounts::InitializeCandidate {
            signer,
            poll_account: poll_pda,
            candidate_account: candidate_pda,
            system_program: anchor_lang::solana_program::system_program::id(),
        }.to_account_metas(None),
    )
}

fn make_vote_ix(
    poll_id: u64,
    candidate: String,
    poll_pda: Pubkey,
    candidate_pda: Pubkey,
    signer: Pubkey,
) -> Instruction {
    Instruction::new_with_bytes(
        voting::id(),
        &voting::instruction::Vote {
            _poll_id: poll_id,      
            _candidate: candidate,
        }.data(),
        voting::accounts::Vote {
            signer,
            poll_account: poll_pda,
            candidate_account: candidate_pda,
            system_program: anchor_lang::solana_program::system_program::id(),
        }.to_account_metas(None),
    )
}

// ─── set clock helper ─────────────────────────────────────────────────────
fn set_clock(svm: &mut LiteSVM, unix_timestamp: i64) {
    svm.set_sysvar(&Clock {
        unix_timestamp,
        slot: 100,
        epoch: 1,
        leader_schedule_epoch: 2,
        epoch_start_timestamp: unix_timestamp - 100,
    });
}

// ─── Test 1: initialize poll ──────────────────────────────────────────────
#[test]
fn test_init_poll() {
    let (mut svm, payer) = setup();
    let poll_id: u64 = 1;
    let (poll_pda, _) = find_poll_pda(poll_id);

    let ix = make_init_poll_ix(
        poll_id,
        1000,                           // start time
        2000,                           // end time
        "Best Language".to_string(),
        "Vote for your favourite language".to_string(),
        poll_pda,
        payer.pubkey(),
    );

    let res = send_ix(&mut svm, &payer, ix);

    println!("\n--- INIT POLL LOGS ---");
    for log in &res.logs { println!("{}", log); }
    println!("----------------------\n");

    // poll has no msg!() so just verify transaction succeeded
    let logs = res.logs.join(" ");
    assert!(
        logs.contains("Instruction: InitPoll"),
        "InitPoll instruction not found: {}",
        logs
    );
    println!("Poll initialized successfully");
}

// ─── Test 2: initialize candidate ────────────────────────────────────────
#[test]
fn test_initialize_candidate() {
    let (mut svm, payer) = setup();
    let poll_id: u64 = 1;
    let candidate_name = "Rust";
    let (poll_pda, _) = find_poll_pda(poll_id);
    let (candidate_pda, _) = find_candidate_pda(poll_id, candidate_name);

    // init poll first
    send_ix(&mut svm, &payer, make_init_poll_ix(
        poll_id,
        1000,
        2000,
        "Best Language".to_string(),
        "Vote for your favourite language".to_string(),
        poll_pda,
        payer.pubkey(),
    ));

    // init candidate
    let ix = make_init_candidate_ix(
        poll_id,
        candidate_name.to_string(),
        poll_pda,
        candidate_pda,
        payer.pubkey(),
    );

    let res = send_ix(&mut svm, &payer, ix);

    println!("\n--- INIT CANDIDATE LOGS ---");
    for log in &res.logs { println!("{}", log); }
    println!("---------------------------\n");

    let logs = res.logs.join(" ");
    assert!(
        logs.contains("Instruction: InitializeCandidate"),
        "InitializeCandidate not found: {}",
        logs
    );
    println!("Candidate Rust initialized successfully");
}

// ─── Test 3: vote during valid window → success ───────────────────────────
#[test]
fn test_vote_valid() {
    let (mut svm, payer) = setup();
    let poll_id: u64 = 1;
    let candidate_name = "Rust";
    let (poll_pda, _) = find_poll_pda(poll_id);
    let (candidate_pda, _) = find_candidate_pda(poll_id, candidate_name);

    // set clock to voting window
    let start: u64 = 1000;
    let end: u64 = 5000;
    set_clock(&mut svm, 2000); // inside window

    // init poll
    send_ix(&mut svm, &payer, make_init_poll_ix(
        poll_id,
        start,
        end,
        "Best Language".to_string(),
        "Vote for your favourite language".to_string(),
        poll_pda,
        payer.pubkey(),
    ));

    // init candidate
    send_ix(&mut svm, &payer, make_init_candidate_ix(
        poll_id,
        candidate_name.to_string(),
        poll_pda,
        candidate_pda,
        payer.pubkey(),
    ));

    // vote
    let ix = make_vote_ix(
        poll_id,
        candidate_name.to_string(),
        poll_pda,
        candidate_pda,
        payer.pubkey(),
    );

    let res = send_ix(&mut svm, &payer, ix);

    println!("\n--- VOTE LOGS ---");
    for log in &res.logs { println!("{}", log); }
    println!("-----------------\n");

    let logs = res.logs.join(" ");
    assert!(
        logs.contains("Instruction: Vote"),
        "Vote instruction not found: {}",
        logs
    );
    println!("Vote cast successfully for Rust");
}

// ─── Test 4: vote before start → VotingNotStarted ────────────────────────
#[test]
fn test_vote_not_started() {
    let (mut svm, payer) = setup();
    let poll_id: u64 = 1;
    let candidate_name = "Rust";
    let (poll_pda, _) = find_poll_pda(poll_id);
    let (candidate_pda, _) = find_candidate_pda(poll_id, candidate_name);

    // set clock BEFORE voting starts
    set_clock(&mut svm, 500); // before start of 1000

    // init poll
    send_ix(&mut svm, &payer, make_init_poll_ix(
        poll_id,
        1000,                           // start
        5000,                           // end
        "Best Language".to_string(),
        "Vote for your favourite language".to_string(),
        poll_pda,
        payer.pubkey(),
    ));

    // init candidate
    send_ix(&mut svm, &payer, make_init_candidate_ix(
        poll_id,
        candidate_name.to_string(),
        poll_pda,
        candidate_pda,
        payer.pubkey(),
    ));

    // vote before start — should fail
    let ix = make_vote_ix(
        poll_id,
        candidate_name.to_string(),
        poll_pda,
        candidate_pda,
        payer.pubkey(),
    );

    let e = send_ix_expect_fail(&mut svm, &payer, ix);
    let err_str = format!("{:?}", e);
    assert!(
        err_str.contains("VotingNotStarted"),
        "Expected VotingNotStarted but got: {}",
        err_str
    );
    println!("Correctly rejected vote before start");
}

// ─── Test 5: vote after end → VotingEnded ────────────────────────────────
#[test]
fn test_vote_ended() {
    let (mut svm, payer) = setup();
    let poll_id: u64 = 1;
    let candidate_name = "Rust";
    let (poll_pda, _) = find_poll_pda(poll_id);
    let (candidate_pda, _) = find_candidate_pda(poll_id, candidate_name);

    // set clock AFTER voting ends
    set_clock(&mut svm, 6000); // after end of 5000

    // init poll
    send_ix(&mut svm, &payer, make_init_poll_ix(
        poll_id,
        1000,
        5000,
        "Best Language".to_string(),
        "Vote for your favourite language".to_string(),
        poll_pda,
        payer.pubkey(),
    ));

    // init candidate
    send_ix(&mut svm, &payer, make_init_candidate_ix(
        poll_id,
        candidate_name.to_string(),
        poll_pda,
        candidate_pda,
        payer.pubkey(),
    ));

    // vote after end — should fail
    let ix = make_vote_ix(
        poll_id,
        candidate_name.to_string(),
        poll_pda,
        candidate_pda,
        payer.pubkey(),
    );

    let e = send_ix_expect_fail(&mut svm, &payer, ix);
    let err_str = format!("{:?}", e);
    assert!(
        err_str.contains("VotingEnded"),
        "Expected VotingEnded but got: {}",
        err_str
    );
    println!("Correctly rejected vote after end");
}

// ─── Test 6: multiple candidates + multiple votes ─────────────────────────
#[test]
fn test_multiple_candidates() {
    let (mut svm, payer) = setup();
    let poll_id: u64 = 1;
    let (poll_pda, _) = find_poll_pda(poll_id);

    set_clock(&mut svm, 2000);

    // init poll
    send_ix(&mut svm, &payer, make_init_poll_ix(
        poll_id,
        1000,
        5000,
        "Best Language".to_string(),
        "Vote for your favourite language".to_string(),
        poll_pda,
        payer.pubkey(),
    ));

    // add Rust candidate
    let rust_pda = find_candidate_pda(poll_id, "Rust").0;
    send_ix(&mut svm, &payer, make_init_candidate_ix(
        poll_id,
        "Rust".to_string(),
        poll_pda,
        rust_pda,
        payer.pubkey(),
    ));

    // add Python candidate
    let python_pda = find_candidate_pda(poll_id, "Python").0;
    send_ix(&mut svm, &payer, make_init_candidate_ix(
        poll_id,
        "Python".to_string(),
        poll_pda,
        python_pda,
        payer.pubkey(),
    ));

    // vote for Rust
    let res = send_ix(&mut svm, &payer, make_vote_ix(
        poll_id,
        "Rust".to_string(),
        poll_pda,
        rust_pda,
        payer.pubkey(),
    ));

    println!("\n--- VOTE FOR RUST LOGS ---");
    for log in &res.logs { println!("{}", log); }
    println!("--------------------------\n");

    let logs = res.logs.join(" ");
    assert!(
        logs.contains("Instruction: Vote"),
        "Vote for Rust not found: {}",
        logs
    );

    // vote for Python
    let res2 = send_ix(&mut svm, &payer, make_vote_ix(
        poll_id,
        "Python".to_string(),
        poll_pda,
        python_pda,
        payer.pubkey(),
    ));

    println!("\n--- VOTE FOR PYTHON LOGS ---");
    for log in &res2.logs { println!("{}", log); }
    println!("----------------------------\n");

    let logs2 = res2.logs.join(" ");
    assert!(
        logs2.contains("Instruction: Vote"),
        "Vote for Python not found: {}",
        logs2
    );

    println!("Multiple candidates voted successfully");
}