# Voting-SOL

A decentralized on-chain voting program built on **Solana** using the **Anchor** framework. The program allows anyone to create polls, register candidates, and cast votes — all enforced by smart contract logic with time-gated voting windows.

---

## 🗳️ Features

- **Create Polls** — Initialize a poll with a name, description, and a start/end timestamp window.
- **Register Candidates** — Add candidates to any existing poll.
- **Cast Votes** — Vote for a candidate; the program enforces that voting only happens within the allowed time window.
- **PDA-based Accounts** — Poll and candidate accounts are derived using Program Derived Addresses (PDAs) for deterministic and secure on-chain storage.
- **Error Handling** — Custom error codes for voting-window violations.

---

## 🏗️ Program Architecture

### Program ID
```
89MNRWbDmSCUTdLPBXyEs5hA8RYZ9Y2NncpU6CgyenoW
```

### Instructions

| Instruction             | Description                                                    |
|-------------------------|----------------------------------------------------------------|
| `init_poll`             | Initializes a new poll with name, description, and time range |
| `initialize_candidate`  | Adds a candidate to an existing poll                          |
| `vote`                  | Casts a vote for a candidate within the voting window         |

---

## 📦 On-Chain Account Structures

### `PollAccount`

| Field                  | Type     | Max Len | Description                       |
|------------------------|----------|---------|-----------------------------------|
| `poll_name`            | `String` | 32      | Name of the poll                  |
| `poll_description`     | `String` | 128     | Description of the poll           |
| `poll_voting_start`    | `u64`    | —       | Unix timestamp for voting start   |
| `poll_voting_end`      | `u64`    | —       | Unix timestamp for voting end     |
| `poll_option_index`    | `u64`    | —       | Counter tracking number of candidates |

### `CandidateAccount`

| Field              | Type     | Max Len | Description                  |
|--------------------|----------|---------|------------------------------|
| `candidate_name`   | `String` | 32      | Name of the candidate        |
| `candidate_vote`   | `u64`    | —       | Number of votes received     |

---

## 🔑 PDA Seeds

| Account             | Seeds                                      |
|---------------------|--------------------------------------------|
| `PollAccount`       | `["poll", poll_id (le_bytes)]`             |
| `CandidateAccount`  | `["poll", poll_id (le_bytes), candidate_name]` |

---

## ⚠️ Error Codes

| Error               | Message                        |
|---------------------|-------------------------------|
| `VotingNotStarted`  | Voting has not started yet     |
| `VotingEnded`       | Voting has ended               |

---

## 🧪 Testing

Tests are written in Rust and use [LiteSVM](https://github.com/LiteSVM/litesvm) — a lightweight, fast Solana VM for unit testing without spinning up a validator.

### Run Tests

```bash
cargo test
```

The test suite (`test_initialize.rs`) covers:
- Deriving poll PDAs correctly
- Sending a versioned transaction to initialize a poll
- Verifying the transaction succeeds on-chain

---

## 🛠️ Tech Stack

| Tool / Library    | Version   | Purpose                          |
|-------------------|-----------|----------------------------------|
| Rust              | 1.89.0    | Core programming language        |
| Anchor            | 1.0.0     | Solana smart contract framework  |
| LiteSVM           | 0.10.0    | Lightweight Solana VM for tests  |
| solana-message    | 3.0.1     | Transaction message construction |
| solana-transaction| 3.0.2     | Versioned transaction support    |
| Yarn              | —         | Package manager                  |

---

## 🚀 Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (`rustup` managed, channel `1.89.0`)
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
# Start local validator
solana-test-validator

# Deploy
anchor deploy
```

### Run Tests

```bash
anchor test
# or for Rust unit tests only:
cargo test
```

---

## 📁 Project Structure

```
Voting-SOL/
├── Anchor.toml                      # Anchor config (cluster, wallet, program IDs)
├── Cargo.toml                       # Workspace Cargo manifest
├── rust-toolchain.toml              # Pinned Rust toolchain (1.89.0)
├── package.json                     # Node dependencies
├── migrations/
│   └── deploy.ts                    # Anchor migration script
└── programs/
    └── voting/
        ├── Cargo.toml               # Program dependencies
        └── src/
            ├── lib.rs               # Program entry point & all instructions
            ├── instructions.rs      # Instruction module declarations
            ├── error.rs             # Custom error codes
            ├── constants.rs         # Program constants
            ├── state.rs             # Account state (extended definitions)
            └── instructions/
                └── initialize.rs    # Initialize handler
        └── tests/
            └── test_initialize.rs   # LiteSVM unit tests
```

---

## 📄 License

This project is open source. See the repository for license details.

---

## 🙋 Author

**Akshat** — [@Akshat0125](https://github.com/Akshat0125)
