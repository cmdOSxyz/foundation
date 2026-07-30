# cmdCapital — System Specification and Build Standard

> Source: `cmdCapital - Bản Đặc Tả Hệ Thống Và Quy Chuẩn Xây Dựng.pdf` (Google Docs,
> 3 pages, 2026-07-30), translated from Vietnamese. The binary original was replaced by
> this document so the content lives in git rather than as a loose file outside the repo.
>
> This is a specification, **not an RFC**. Under the RFC-first rule, cmdCapital needs an
> Accepted RFC in `docs/rfcs/` before any component code is written.

Project: **cmdCapital — Omnichain Ecosystem**

## 1. Vision and Mission

Build a decentralized onchain verification layer and marketplace dedicated to autonomous
AI trading agents.

The objective is to remove two risks completely:

- **Fake track record** — a screenshot of a PnL curve costs nothing to produce and nothing
  to fake.
- **Counterparty risk** — following a strategy usually means handing someone the ability
  to move your funds.

The approach combines cryptography, TEE hardware, and smart contracts to make trading
performance transparent (Proof of PnL).

### Packaging and Brand Positioning

The flagship agent ships under the exclusive commercial name **AuraCore**, with the
message **"Your autonomous nexus"**.

Brand identity is standardized around premium commercial packaging, in the manner of
BANDAI-standard boxes for 1/7 scale hyper-realistic figurines.

For visual communication, the agent's imagery is grounded in Web3 culture: 3D modeling in
ZBrush combined with pixel NFT art. Promotional frames place the character at the center,
surrounded by icons familiar to crypto-native users (Azuki, Penguin, CryptoPunks). Primary
graphics are optimized to a wide horizontal ratio (1500x500 px) for the official banner.

## 2. Product Suite

- **Super App** — Telegram Mini-App and web dashboard. Natural language processing,
  real-time asset state reporting, market movement analysis, and strategy activation
  commands.
- **Marketplace** — lists reviewed AI trading agents. Each bot carries a transparent
  onchain leaderboard showing ROI, win rate, and maximum drawdown, attested by TEE
  hardware.
- **Smart Wallet** — personal custody wallet on the Account Abstraction standard
  (ERC-4337). Sign in with a social account or email, no seed phrase. Strictly
  non-custodial: the right to withdraw belongs 100% to the owner.
- **Vaults** — for passive investors. Automatically allocates capital across the
  top-performing agents on the marketplace and rebalances on a fixed cycle.

## 3. User Journey

A "1-Click Copy Trade" model:

1. **Onboarding and funding** — the system creates a personalized smart wallet. The user
   deposits USDC or USDT from an external wallet or a centralized exchange.
2. **Screening and selection** — the investor studies the strategy list on the
   marketplace, reads cryptographically attested PnL figures, and decides how much to
   allocate.
3. **Safe delegation and circuit breaker** — the user grants a session key with limited
   privileges: the bot **may only trade and can never withdraw the principal**. A circuit
   breaker automatically suspends the bot when assets fall to the configured risk
   threshold (for example -15%).
4. **Live monitoring** — the Super App streams order movements, cash flow state, and ROI
   back in real time.
5. **Settlement and profit split** — the investor can stop the strategy at any time. The
   smart contract deducts a 20% performance fee **on realised profit only** before
   returning the remainder.

## 4. Architecture Standard — Four Core Backend Layers

- **NexusKernel** (execution and hardware) — the AI agent runs inside a TEE (Intel SGX or
  AMD SEV). The private key is generated in full isolation inside the enclave and cannot
  be extracted. Every order the AI produces is signed and carries a remote attestation
  proof.
- **Metric Engine** (audit and data) — a decentralized indexer tracks trades on DEX and
  perp DEX venues (Jupiter, Uniswap, Hyperliquid) and derives ROI, maximum drawdown
  (MDD), Sharpe ratio, and win rate. All data is recorded permanently onchain.
- **NexusShield** (user custody) — Account Abstraction (ERC-4337) smart wallet management.
  The bot receives only a session key limited to trading, with withdrawal forbidden.
- **NexusEscrow** (settlement and reconciliation) — a smart contract automatically
  distributes the performance fee split at the end of each agent operating cycle.

## 5. Omnichain Technical Loop

1. **Signal detection** — inside the TEE, the agent continuously scans market-wide data
   across EVM, SVM, and MoveVM networks along with social channels. When it finds an
   opportunity (for example a pool on Sui offering 28% annualized), the agent decides.
2. **Order dispatch and synchronization** — the TEE signs the raw transactions to withdraw
   capital, bridges through Wormhole or LayerZero, and deposits into the new pool. A
   relayer uses the session key to attach signatures and executes across thousands of
   copy-trading wallets within milliseconds so followers do not absorb the slippage of
   going last.
3. **Yield sharing** — performance fee revenue splits **85%** directly to the AI
   developer's wallet to sustain the system and **15%** into the cmdCapital protocol
   treasury, part of which funds a **buy-back and burn** of the governance token.

## Policy Parameters

| Key | Value |
| --- | --- |
| `agent_permissions` | trade only, withdrawal denied |
| `circuit_breaker` | agent suspended at the drawdown the owner sets (for example -15%) |
| `performance_fee` | 20% of realised profit, charged on profit only |
| `fee_split` | 85% AI developer, 15% protocol treasury |
| `settlement_gate` | R3, human approval required |

These are **settings, not results**. This document contains no performance figure.

## Status

No cmdCapital code exists in this repository. The dependency order is mandatory: attested
execution and derived metrics come before any marketplace surface, and settlement is built
last.

The leaderboard may show a number only once three conditions hold: the TEE attestation
pipeline (NexusKernel), the venue indexer and metric derivation (Metric Engine), and at
least one reviewed agent with a settled cycle.
