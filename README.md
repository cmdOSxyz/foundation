# cmdOS — foundation

> **The direction this repository was built for was retired on 2026-08-04, and the
> brand name changes with whatever replaces it.** What that is has not been decided.
> The reasoning is recorded in [`docs/archive/README.md`](docs/archive/README.md).
>
> There is no product claim here on purpose. This README describes what the code
> actually is, and nothing else.

---

## What is here

A Rust workspace of seven crates: infrastructure for making an unreliable actor's
irreversible actions safe and reviewable.

| Crate | What it does |
| --- | --- |
| `cmd-types` | Object model the rest depend on |
| `cmd-transaction` | simulate → snapshot → execute → verify → commit/rollback |
| `cmd-policy` | R0–R3 risk classes, budgets, mandate checks |
| `cmd-ledger` | Append-only hash-chained audit record |
| `cmd-approval` | Binds an approval to an exact plan digest and pre-state |
| `cmd-proof` | Proof Bundle v0 — evidence for one execution, verifiable without the original data |
| `cmd-shadow` | Copy-on-write forks: run alternatives to completion, promote one or discard |

Alongside it: `schemas/` (TypeScript mirror of the object model), `prototype/`
(a runnable TypeScript reference implementation whose tests are the behavior
contracts the Rust must satisfy), and `docs/`.

## Build

```bash
cargo build --workspace
cargo test  --workspace          # 29 tests
bash tools/ci/check-docs.sh      # documentation invariants
npm install && npm test          # prototype behavior contracts
```

## What was removed

On 2026-08-04 the components that existed only to serve the agent-OS
direction were removed: the intent scheduler, all services, the Alios agent, the
capability servers, the CLI and the Tauri shell — 5,847 lines. The architecture
documentation that described them moved to `docs/archive/`, and the fourteen RFCs
that specified them are marked `Status: Superseded` rather than deleted.

Nothing is lost. Every removed path is recoverable:

```bash
git checkout pre-cmdcapital-trim -- <path>
```

## Documentation

- [`docs/archive/README.md`](docs/archive/README.md) — why the direction was retired, and what was kept
- [`docs/rfcs/`](docs/rfcs/) — the RFC record, including superseded ones
- [`docs/02-philosophy/`](docs/02-philosophy/) and [`docs/03-design-principles/`](docs/03-design-principles/) — engineering principles, not tied to a positioning
- [`SECURITY.md`](SECURITY.md) — vulnerability reporting

## Status

Pre-product. No users, no release, no roadmap until a direction is chosen.

## License

Apache-2.0. See [`LICENSE`](LICENSE).
