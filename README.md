# 🎙️ PodcastFund — Recurring Micropayments on Stellar

> **Fund the voices you love — automatically, on-chain, no middlemen.**

PodcastFund is a [Soroban](https://soroban.stellar.org/) smart contract on the **Stellar network** that lets listeners directly support podcasters through recurring micropayments. No platform cuts, no subscription services sitting in the middle — just a transparent, trustless stream of support flowing straight to creators every period.

---

## 📖 Project Description

The podcast economy is broken. Creators depend on ad networks and platform algorithms while listeners have no direct way to fund the content they value. PodcastFund changes that by putting subscriptions on-chain.

A podcaster registers their show with a price and a billing period. Listeners subscribe and approve recurring charges. A keeper bot (or anyone) calls `collect_payment` each period — the contract verifies timing, transfers tokens directly from the listener's wallet to the podcaster's wallet, and emits an event. No escrow, no wrapping, no custody — the tokens never touch the contract.

Built on **Soroban** (Stellar's WASM smart contract platform), PodcastFund benefits from Stellar's ~5-second finality, sub-cent transaction fees, and native token standard (SAC), making true micropayments practical for the first time.

---

## ✨ What It Does

| Step | Who | Action |
|------|-----|--------|
| 1 | **Podcaster** | Calls `register_podcast` — sets name, price per period, billing interval, and accepted token |
| 2 | **Listener** | Calls `subscribe` — first payment is charged immediately; wallet pre-authorizes future pulls |
| 3 | **Keeper / bot** | Calls `collect_payment` each period — contract checks elapsed time, transfers tokens, emits event |
| 4 | **Listener** | Calls `unsubscribe` anytime — no refunds for the current period, no future charges |
| 5 | **Podcaster** | Calls `deactivate_podcast` to stop accepting new subscribers |

All payment flows are **peer-to-peer**: tokens go directly from listener → podcaster. The contract never holds funds.

---

## 🚀 Features

### 💸 True Micropayments
Stellar's near-zero fees make per-episode or daily billing economically viable — something impossible on Ethereum. Set prices as low as a fraction of a cent.

### 🔁 Recurring Billing, On-Chain
Subscriptions are stored in persistent contract storage. A keeper (cron job, bot, or any caller) triggers `collect_payment` — the contract enforces the time lock so no double-charges are possible.

### 🔒 Non-Custodial
The contract **never holds listener funds**. Every `transfer` goes directly from the listener's Stellar account to the podcaster's account using the SAC token standard. Zero smart-contract custody risk.

### 📢 Event Emission
Every key action — registration, subscribe, payment, unsubscribe — emits a Soroban event. Index these with Horizon or a custom indexer to build dashboards, analytics, or push notifications.

### 🪙 Any SAC Token
Accepts any Stellar Asset Contract (SAC) token: native XLM, USDC, or a custom creator token. Each podcast picks its own currency.

### 📋 Rich Read Interface
Query helpers (`get_podcast`, `get_subscription`, `get_listeners`, `is_subscribed`) make it trivial to build frontends or bots without extra indexing infrastructure.

### 🛑 Graceful Cancellation
Listeners cancel with `unsubscribe` at any time. Podcasters can `deactivate_podcast` to freeze new sign-ups without breaking existing subscribers.

### ⚡ Soroban-Native
Written in `#![no_std]` Rust, compiled to WASM, and deployed on Stellar — benefiting from ~5s finality, deterministic gas, and the full Soroban SDK (`contracttype`, persistent storage, token interface).

---

## 🗂️ Project Structure

```
podcast-fund/
├── Cargo.toml          # Rust workspace & Soroban SDK dependency
└── src/
    └── lib.rs          # Contract: data types, storage keys, all entry points
```

---

## 🛠️ Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) + `wasm32-unknown-unknown` target
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/cli/stellar-cli)

```bash
rustup target add wasm32-unknown-unknown
cargo install --locked stellar-cli --features opt
```

### Build

```bash
cargo build --target wasm32-unknown-unknown --release
```

### Run Tests

```bash
cargo test
```

### Deploy to Testnet

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/podcast_fund.wasm \
  --source <YOUR_SECRET_KEY> \
  --network testnet
```

---

## 📡 Contract Interface (Summary)

```rust
// Podcaster
register_podcast(owner, name, price_per_period, period_seconds, token) → PodcastInfo
deactivate_podcast(owner)

// Listener
subscribe(listener, podcaster) → SubscriptionInfo
unsubscribe(listener, podcaster)

// Keeper / anyone
collect_payment(listener, podcaster) → i128   // returns amount charged

// Read-only
get_podcast(podcaster)             → PodcastInfo
get_subscription(listener, podcaster) → SubscriptionInfo
get_listeners(podcaster)           → Vec<Address>
is_subscribed(listener, podcaster) → bool
```

---

## 🗺️ Roadmap Ideas

- [ ] Trial periods (first N periods free)
- [ ] Tiered subscription levels per podcast
- [ ] On-chain fan badges / NFT receipts via SAC
- [ ] Multi-sig podcaster payouts (split revenue between co-hosts)
- [ ] Frontend dApp with Freighter wallet integration

---

## 📄 License

MIT — free to use, fork, and build upon.

---

> Built with ❤️ on [Stellar Soroban](https://soroban.stellar.org/)

wallet address - GAQ24HMN5BBQG74TQIBBYEC3NMKBF6SZCAWK35PXLB3P6374RRJK7UGV

Contract address - CBWXP75DJLP25HF2BJBMHOZNE42M7JJLTSZKOB4STLUBVNZKTZ6L63QR

https://stellar.expert/explorer/testnet/contract/CBWXP75DJLP25HF2BJBMHOZNE42M7JJLTSZKOB4STLUBVNZKTZ6L63QR

<img width="1920" height="1080" alt="image" src="https://github.com/user-attachments/assets/e578edb1-186a-4874-bb18-706f69ed8391" />
