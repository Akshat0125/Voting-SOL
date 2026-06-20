# Voting-SOL

A fully on-chain decentralized voting program built on **Solana** using the **Anchor** framework. Supports creating time-gated polls, registering candidates via PDAs, and casting votes — all enforced by smart contract logic with custom error handling.

---

## 🗳️ Features

- **Create Polls** — Initialize a poll with a name, description, and a Unix timestamp voting window.
- **Register Candidates** — Add multiple candidates to any existing poll; each stored in its own PDA.
- **Cast Votes** — Vote for a candidate with on-chain time enforcement (rejects votes outside the window).
- **PDA-based Storage** — Poll and candidate accounts are derived deterministically using PDAs.
- **Custom Errors** — `VotingNotStarted` and `VotingEnded` errors for clean failure handling.
- **Comprehensive Tests** — 8 async integration tests using `solana-program-test` covering all instructions and edge cases.

---

## 🏗️ Program Architecture

### Program ID
```
89MNRWbDmSCUTdLPBXyEs5hA8RYZ9Y2NncpU6CgyenoW
```

### Instructions

| Instruction              | Parameters                                        | Description                                           |
|--------------------------|---------------------------------------------------|-------------------------------------------------------|
| `init_poll`              | `poll_id, start, end, name, description`          | Creates a new poll with a time-gated voting window    |
| `initialize_candidate`   | `poll_id, candidate`                              | Adds a candidate to an existing poll                  |
| `vote`                   | `poll_id, candidate`                              | Casts a vote; enforces start/end time on-chain        |

---

## 📦 On-Chain Account Structures

### `PollAccount`

| Field                 | Type     | Max Len | Description                            |
|-----------------------|----------|---------|----------------------------------------|
| `poll_name`           | `String` | 32      | Name of the poll                       |
| `poll_description`    | `String` | 128     | Description of the poll                |
| `poll_voting_start`   | `u64`    | —       | Unix timestamp when voting opens       |
| `poll_voting_end`     | `u64`    | —       | Unix timestamp when voting closes      |
| `poll_option_index`   | `u64`    | —       | Auto-incrementing count of candidates  |

### `CandidateAccount`

| Field              | Type     | Max Len | Description                     |
|--------------------|----------|---------|---------------------------------|
| `candidate_name`   | `String` | 32      | Name of the candidate           |
| `candidate_vote`   | `u64`    | —       | Total votes received            |

> Both accounts use Anchor's `#[derive(InitSpace)]` for automatic space calculation.

---

## 🔑 PDA Seeds

| Account             | Seeds                                                   |
|---------------------|---------------------------------------------------------|
| `PollAccount`       | `[b"poll", poll_id.to_le_bytes()]`                      |
| `CandidateAccount`  | `[b"poll", poll_id.to_le_bytes(), candidate_name]`      |

---

## ⚠️ Error Codes

| Error              | Message                     | Trigger                                          |
|--------------------|-----------------------------|--------------------------------------------------|
| `VotingNotStarted` | Voting has not started yet  | `current_time < poll_voting_start`               |
| `VotingEnded`      | Voting has ended            | `current_time > poll_voting_end`                 |

---

## 🧪 Testing

Tests are written in Rust using [`solana-program-test`](https://docs.rs/solana-program-test) — a full async Solana BPF test environment with `banks_client` for submitting transactions and reading on-chain state.

### Test Coverage

| Test                                      | What it verifies                                              |
|-------------------------------------------|---------------------------------------------------------------|
| `test_init_poll_success`                  | Poll initializes with correct fields and zero option index    |
| `test_init_poll_duplicate_fails`          | Re-initializing the same PDA fails                           |
| `test_initialize_candidate_success`       | Candidate created with zero votes; poll index increments to 1 |
| `test_initialize_two_candidates_increments_index` | Adding 2 candidates increments `poll_option_index` to 2 |
| `test_vote_increments_vote_count`         | Voting for a candidate increments their `candidate_vote` to 1 |
| `test_vote_multiple_times_accumulates`    | Voting 3 times accumulates to `candidate_vote = 3`           |
| `test_vote_after_end_fails`               | Voting with `end = 1` (past) returns `VotingEnded` error     |
| `test_vote_before_start_fails`            | Voting with `start = u64::MAX - 1` returns `VotingNotStarted`|
| `test_vote_does_not_change_other_candidates` | Voting for Alice keeps Bob's count at 0                   |

### Run Tests

```bash
cargo test-sbf
```

> Tests require the `test-sbf` feature flag as indicated by `#![cfg(feature = "test-sbf")]`.

---

## 🛠️ Tech Stack

| Tool / Library          | Version   | Purpose                                    |
|-------------------------|-----------|--------------------------------------------|
| Rust                    | 1.89.0    | Core programming language                  |
| Anchor                  | 1.0.0     | Solana smart contract framework            |
| solana-program-test     | —         | Async BPF integration test environment     |
| solana-sdk              | —         | Transaction, Keypair, Instruction building |
| Yarn                    | —         | Node package manager                       |

---

## 🚀 Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) — toolchain `1.89.0` is pinned via `rust-toolchain.toml`
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools)
- [Anchor CLI](https://www.anchor-lang.com/docs/installation)
- [Yarn](https://yarnpkg.com/)

### Installation

```bash
git clone https://github.com/Akshat0125/Voting-SOL.git
cd Voting-SOL
yarn install
```

### Build

```bash
anchor build
```

### Deploy (Localnet)

```bash
# Start a local Solana validator
solana-test-validator

# Deploy the program
anchor deploy
```

### Run Tests

```bash
# Full BPF integration tests
cargo test-sbf

# Anchor test suite
anchor test
```

---

## 📁 Project Structure

```
Voting-SOL/
├── Anchor.toml                        # Cluster, wallet, and program ID config
├── Cargo.toml                         # Workspace manifest
├── rust-toolchain.toml                # Pinned Rust toolchain (1.89.0)
├── package.json                       # Node/Yarn dependencies
├── migrations/
│   └── deploy.ts                      # Anchor deploy migration script
└── programs/
    └── voting/
        ├── Cargo.toml                 # Program crate dependencies
        └── src/
            ├── lib.rs                 # Program entry point: all 3 instructions + account structs + errors
            ├── instructions.rs        # Instruction module re-exports
            ├── instructions/
            │   └── initialize.rs      # Stub initialize handler
            ├── error.rs               # Custom error codes (VotingNotStarted, VotingEnded)
            ├── constants.rs           # Program constants (SEED)
            └── state.rs               # Extended state definitions (reserved)
        └── tests/
            └── test_initialize.rs     # 9 async integration tests (BPF)
```

---

## 📄 License

This project is open source. See the repository for license details.

---

## 🙋 Author

**Akshat** — [@Akshat0125](https://github.com/Akshat0125)
