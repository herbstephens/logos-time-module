# Architecture: TIME Protocol as a Logos Core Module

## System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        LOGOS NODE                               │
│                                                                 │
│  ┌──────────┐   ┌──────────┐   ┌───────────────────────────┐   │
│  │ Messaging│   │  Storage │   │       Blockchain           │   │
│  │  (Waku)  │   │ (Codex)  │   │       (Nomos/LSSA)         │   │
│  └────┬─────┘   └────┬─────┘   └──────────┬────────────────┘   │
│       │              │                    │                     │
│  ─────┴──────────────┴────────────────────┴──────  Logos Core   │
│                    Plugin Runtime                               │
│                         │                                      │
│              ┌──────────▼───────────┐                          │
│              │   logos-time module  │  ◄── This repository     │
│              │                      │                          │
│              │  WakuListener        │                          │
│              │  MintTrigger         │                          │
│              │  BirthrightClock     │                          │
│              └──────────────────────┘                          │
└─────────────────────────────────────────────────────────────────┘
```

## Component Breakdown

### 1. WakuListener
Subscribes to `/time/1/work-agreements/proto` on the Waku messaging layer.
- Receives countersigned WorkAgreements from payers
- Validates both worker and payer signatures (EIP-712)
- Forwards valid agreements to MintTrigger via internal broadcast channel

### 2. MintTrigger
Watches the Logos blockchain for payment confirmation.
- Holds pending agreements keyed by agreementHash
- Queries the Logos chain for `MintEvent` logs from TimeProtocol.sol
- On confirmation: indexes the mint, uploads Work NFT metadata to Logos Storage
- Updates module health metrics

### 3. BirthrightClock
Runs on a 24-hour cadence aligned to UTC midnight.
- Opens each new day's birthright claim window
- Optionally notifies clients via Waku `/time/1/birthright/proto`
- Does NOT push tokens automatically — users self-claim with World ID proof

## The Mint Event: Detailed Flow

```
  WORKER                  PAYER               WAKU                 LOGOS CHAIN
    │                       │                   │                       │
    │  createWorkAgreement  │                   │                       │
    │─────────────────────► │                   │                       │
    │                       │                   │                       │
    │ ◄─ workerSign ──────  │                   │                       │
    │                       │                   │                       │
    │ ──── signed ────────► │                   │                       │
    │                       │  payerSign        │                       │
    │                       │─────────────────► │                       │
    │                       │  publish to topic │                       │
    │                       │                   │ ◄── WakuListener      │
    │                       │                   │     receives          │
    │                       │                   │     agreement         │
    │                       │                   │                       │
    │                       │ payForWork() tx ──┼──────────────────────►│
    │                       │                   │                       │ mint TIME
    │                       │                   │                       │ mint WorkNFT
    │ ◄── TIME tokens ──────┼───────────────────┼───────────────────────│
    │ ◄── WorkNFT ──────────┼───────────────────┼───────────────────────│
    │                       │                   │                       │
    │                       │                   │     MintTrigger ──────│
    │                       │                   │     indexes event     │
    │                       │                   │     → Logos Storage   │
```

## Privacy Model

The module is designed around LSSA's dual-state architecture:

```
Public state  (on-chain, visible):
  - Payment confirmation (required for mint verification)
  - World ID nullifier (required for birthright sybil check)
  - MintEvent log (worker address, TIME amount, NFT ID)

Private state (ZK-protected, owner-only):
  - TIME token balance
  - WorkNFT collection
  - Work NFT metadata (stored encrypted in Logos Storage)

Selectively disclosed (owner controls):
  - WorkNFT content (via ZK proof to chosen verifier)
  - Contribution history (worker chooses what to share)
```

## Logos Stack Integration Points

| Logos Component | TIME Protocol Usage |
|---|---|
| **Messaging (Waku)** | Work agreement coordination topic |
| **Storage (Codex)** | Encrypted Work NFT metadata |
| **Blockchain (LSSA)** | TimeToken + WorkNFT smart contracts |
| **Blend Network** | Proposer privacy for mint transactions |
| **AnonComms** | Future: gasless birthright claim relay |
| **Logos Core** | Plugin registration and lifecycle |

## Node Configuration Reference

```toml
# In your logos-node/config.toml

[modules.logos-time]
enabled                  = true
logos_rpc_url            = "ws://logos-testnet-rpc:8546"
lssa_contract_address    = "0x<TimeProtocol.sol address>"
world_id_verifier        = "0x<WorldID verifier address>"
waku_content_topic       = "/time/1/work-agreements/proto"
waku_node_url            = "ws://localhost:8547"
birthright_interval_secs = 86400
logos_storage_url        = "http://localhost:8090"
signer_key_env           = "TIME_MODULE_SIGNER_KEY"
```
