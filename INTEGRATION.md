# Integration Guide: TIME Protocol for Logos Core

This document is addressed to Logos core contributors and node operators.
It explains how to integrate the TIME Protocol module into the Logos technology
stack and proposes TIME as a reference application for the 2026 testnet.

---

## Why TIME Protocol?

Logos is building the infrastructure for parallel societies: consensus, messaging,
and storage. These are the roads, towers, and vaults of a new civilization.

**TIME Protocol is the economy.**

Without an economic layer that values human work — not capital, not compute,
not validator stake — the Logos stack is infrastructure waiting for inhabitants.
TIME Protocol answers the question every parallel society participant will ask:
*"What is my contribution worth here?"*

### The gap TIME fills

The Logos stack today provides:
- ✅ Private, censorship-resistant communication (Waku)
- ✅ Decentralised, durable storage (Codex)
- ✅ Privacy-preserving L1 with Sovereign Zones (Nomos/LSSA)

What is currently missing:
- ❌ A primitive for valuing human work
- ❌ A verifiable contribution history (reputation layer)
- ❌ A universal economic floor (birthright allocation)
- ❌ A sybil-resistant identity layer for economic participation

TIME Protocol provides all four.

---

## Proposal: TIME as a Logos Testnet Reference Application

### What we are proposing

1. **Module listing**: Include `logos-time` in the official Logos Core community
   plugin library once the plugin registry is live.

2. **Testnet co-deployment**: Deploy TIME Protocol contracts to LSSA on the 2026
   testnet. Co-announce as a reference application demonstrating the full Logos
   stack in a real economic use case.

3. **Specification input**: Add explicit support for ERC-5192 soulbound/
   non-transferable tokens in the LSSA token standard documentation.
   This is a small addition that benefits all reputation and contribution
   protocols built on Logos, not just TIME.

### What this costs Logos

- No changes to core protocols required
- No governance changes required
- Module loads as a Logos Core plugin — opt-in for node operators
- Logos team involvement is optional; TIME can be entirely community-built

### What Logos gains

- A flagship economic application before mainnet
- Validation that the Sovereign Zone + LSSA model supports non-financial use cases
- A compelling demonstration for the Parallel Society community
- Alignment with the *Farewell to Westphalia* thesis at the application layer

---

## Technical Integration Checklist

For Logos core contributors who want to validate this integration:

### Blockchain (LSSA)

- [ ] Confirm ERC-20 `transferFrom` support in LSSA execution environment
- [ ] Confirm ERC-721 `_update` hook availability for soulbound enforcement
- [ ] Confirm `bytes32` and `uint256` ABI encoding compatibility
- [ ] Provide testnet RPC endpoint for TIME contract deployment
- [ ] Confirm `block.timestamp` availability for day-number calculation

### Messaging (Waku)

- [ ] Confirm `/time/1/work-agreements/proto` content topic is valid format
- [ ] Provide Waku testnet node URL for integration testing
- [ ] Confirm Waku message size limits (WorkAgreement JSON ~500 bytes — well within limits)

### Storage (Codex/Logos Storage)

- [ ] Provide Storage node API endpoint for Work NFT metadata upload
- [ ] Confirm content-addressing scheme (CID format) for URI construction
- [ ] Confirm access control: owner-encrypted uploads are supported

### Logos Core

- [ ] Share draft `LogosCoreModule` trait signature for alignment
- [ ] Confirm `cdylib` dynamic loading mechanism
- [ ] Confirm config injection format (JSON via C-ABI or TOML parsed pre-load)

---

## Contact

Democracy Earth Foundation
- GitHub: [Logos TIME Module](https://github.com/herbstephens/logos-time-module)
- Email: herb@democracy.earth
- Logos Forum: post to the Logos Forum thread linking this repository
