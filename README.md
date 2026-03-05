# TIME Protocol — Logos Core Module

> **Payment for work is the mint event.**

A [Logos Core](https://logos.co) plugin that brings the TIME Protocol economic layer
to the Logos technology stack. TIME is the missing economic primitive for parallel
societies: a verifiable, sybil-resistant record of human work that creates money
from contribution rather than capital.

---

## What This Module Does

When a Logos node loads the `logos-time` plugin, it gains:

| Capability | Description |
|---|---|
| **Work Agreement Coordination** | Listens on a Waku content topic for signed work agreements between payer and worker |
| **Mint Trigger** | Fires a TIME token mint + soulbound Work NFT on the Logos Blockchain when payment is confirmed |
| **Birthright Clock** | Issues 1 TIME/day to every verified human address (World ID sybil-resistant) |
| **Work Registry** | Stores Work NFT metadata to Logos decentralised storage |
| **Privacy by Default** | All balances and work records use LSSA private state; only ZK proofs are disclosed |

---

## TIME Protocol Canonical Spec

```
1 TIME = 1 hour of human work (the atomic unit)

Birthright allocation:   1 TIME/day  (≈ 0.0417 TIME/hour)
Max earned allocation:  23 TIME/day  (for 23 hours of verified paid work)
Daily maximum:          24 TIME/day  (birthright + earned)

Mint event:             Payment for work → mints TIME to worker + Work NFT (soulbound)
Work NFT:               ERC-5192 soulbound, non-transferable, permanent record
TIME token:             ERC-20, transferable, the currency of parallel society labour
```

---

## Repository Structure

```
logos-time-module/
├── contracts/                    # Solidity smart contracts (EVM / LSSA)
│   ├── src/
│   │   ├── TimeToken.sol         # ERC-20 TIME token with mint-on-payment logic
│   │   ├── WorkNFT.sol           # ERC-5192 soulbound Work NFT
│   │   ├── BirthrightClock.sol   # Daily 1 TIME/day birthright issuer
│   │   └── TimeProtocol.sol      # Orchestrator — the primary entry point
│   └── foundry.toml
├── logos-core-module/            # Rust Logos Core plugin
│   ├── src/
│   │   ├── lib.rs                # Module trait impl + registration
│   │   ├── service.rs            # Core TIME service (async, Tokio)
│   │   ├── waku_listener.rs      # Subscribes to work agreement topic
│   │   ├── mint_trigger.rs       # Fires mint tx on confirmed payment
│   │   └── birthright.rs        # Daily birthright clock
│   └── Cargo.toml
├── waku-integration/             # TypeScript — off-chain coordination layer
│   ├── src/
│   │   ├── workAgreement.ts      # Sign + publish work agreements to Waku
│   │   ├── paymentWatcher.ts     # Watch for on-chain payment confirmation
│   │   └── index.ts              # Entry point
│   └── package.json
├── docs/
│   ├── ARCHITECTURE.md           # System design and flow diagrams
│   ├── INTEGRATION.md            # How to add logos-time to your Logos node
│   └── TIME_SPEC.md              # Full TIME Protocol specification
└── README.md                     # This file
```

---

## Quick Start

### Prerequisites
- Rust 1.75+
- Node.js 20+
- Foundry (`curl -L https://foundry.paradigm.xyz | bash`)
- A running Logos node (testnet)

### 1. Add the module to your Logos node

```toml
# logos-node/config.toml
[modules]
logos-time = { version = "0.1.0", enabled = true }

[modules.logos-time]
world_id_verifier = "0x..."          # World ID verifier contract address
birthright_interval_secs = 86400     # 24 hours
waku_content_topic = "/time/1/work-agreements/proto"
lssa_contract_address = "0x..."      # TimeProtocol.sol deployment address
```

### 2. Deploy contracts to LSSA testnet

```bash
cd contracts
forge install
forge build
forge script script/Deploy.s.sol --rpc-url $LOGOS_RPC --broadcast
```

### 3. Run the Waku coordination layer

```bash
cd waku-integration
npm install
WAKU_NODE_URL=ws://localhost:8546 npm start
```

---

## How It Works: The Mint Event Flow

```
Worker + Payer agree on work
         │
         ▼
  Both sign WorkAgreement message
         │
         ▼
  Published to Waku topic:
  /time/1/work-agreements/proto
         │
         ▼
  Logos Core module detects message
         │
         ▼
  Payer sends payment tx on Logos chain
         │
         ▼
  TimeProtocol.sol confirms payment
         ├──► Mints TIME tokens to worker
         └──► Mints soulbound WorkNFT to worker
                (stores metadata to Logos Storage)
```

**Payment for work is the mint event.** There is no other way to create TIME.

---

## The Birthright Allocation

Every verified human receives 1 TIME/day — regardless of whether they perform paid work.
This is the economic floor of the parallel society: no human starts from zero.

- Sybil resistance is provided by **World ID** (nullifier-based, one allocation per person)
- The `BirthrightClock.sol` contract issues the daily allocation on a 24-hour cadence
- Birthright TIME is indistinguishable from earned TIME on-chain (privacy by default)

---

## Privacy Model

The module is designed around Logos' LSSA dual-state model:

| Data | State | Visible to |
|---|---|---|
| TIME token balance | **Private** | Owner only (ZK proof on request) |
| Work NFT existence | **Private** | Owner only |
| Work NFT disclosure | **Selective** | Owner chooses what to reveal |
| Payment confirmation | **Public** | Required for mint verification |
| World ID nullifier | **Public** | Required for birthright sybil check |

---

## Connection to the Logos Mission

Logos is building the infrastructure for parallel societies: consensus, messaging,
and storage. TIME Protocol is the **economic activation layer** — the primitive that
answers the question *"what is human contribution worth in this new system?"*

> *"Logos is building the city. TIME Protocol is its economy."*

A parallel society without an economy of work is infrastructure waiting for inhabitants.
TIME Protocol gives the inhabitants a reason to build — and a permanent record that they did.

---

## Contributing

This module is open-source and MIT licensed. Pull requests welcome.

Maintained by [Democracy Earth Foundation](https://democracy.earth) /
[herbstephens](https://github.com/herbstephens).

To propose TIME Protocol as a reference application for the Logos testnet,
see [docs/INTEGRATION.md](docs/INTEGRATION.md).
