# Archive

Documents that no longer describe what this project is, kept because rule 8 says
superseded docs are archived and never deleted.

---

## 2026-08-04 — the operating-system positioning was retired

### What was retired

The framing in `docs/01-vision/strategy-v2.md`: that cmdOS is an operating system, the
successor category to Windows, macOS and Linux for the AI era, delivered as six
self-developed technologies (cmdKernel, SemFS, NIS, AIPC, cmdShell, cmdPay) across three
horizons ending in OEM devices.

The `cmdCapital` direction in `docs/01-vision/cmdcapital-spec.md` and
[RFC-0024](../rfcs/0024-cmdcapital.md) is on the same hold. RFC-0024 was never accepted
and blocks its own implementation, so no code is affected.

### Why

Four reasons, recorded plainly so they do not have to be re-derived.

**The scope was a decade of work and the team is small.** `strategy-v2.md` says this
itself in its own risk register: "six self-developed technologies is a decade of work."
Each of the six is a company. The document then cites Fuchsia — Google, ten years,
still niche — as a cautionary example, and proceeds anyway.

**There was no single answer to "what does cmdOS sell."** Three incompatible product
descriptions were live at the same time: the six-technology OS in `strategy-v2.md`, the
agent-with-a-personal-cloud-computer in `ROADMAP.md`, and a six-product suite
(cmdFirewall, cmdShadow, cmdProof, cmdMandate, cmdSettle, cmdCapital) in the root
`CLAUDE.md` whose names appear nowhere in the strategy document.

**Twenty-six RFCs, zero users.** "Lock the wedge persona" was still an open item in the
week 0-2 section of the roadmap while the architecture had reached RFC-0025. The process
was producing specifications faster than it was producing contact with anyone who would
use the result.

**The moat argument ran the wrong way.** The claim was six technologies "no incumbent can
retrofit." But Microsoft owns the operating system on a billion machines and is adding
agents to it, while cmdOS had agents and was trying to build an operating system around
them. Adding agents to an OS is easier than adding an OS to an agent. This risk was not
in the risk register.

### What survives, and is not archived

The engineering is sound and is deliberately left in place:

- `kernel/cmd-transaction` — simulate → snapshot → execute → verify → commit/rollback.
  Correct for any system where an autonomous actor causes irreversible effects.
- `kernel/cmd-policy` — risk classification and budget enforcement.
- `kernel/cmd-ledger` — append-only signed audit record.
- `kernel/cmd-types` — the object model the above depend on.
- `kernel/cmd-shadow` — fork, run to completion, choose the outcome. The clearest product
  idea the project produced.
- `prototype/tests` — the behavior contracts. These encode what "correct" means and do
  not depend on any positioning.
- The RFC process itself, and [RFC-0024](../rfcs/0024-cmdcapital.md), which is retained
  as a piece of analysis: it argues against its own design, and its findings on
  attestation survivorship bias and relayer front-running remain true.

These are kept for optionality, not because they port directly to whatever comes next.
The concepts transfer; the code is built around Intent / Agent / Capability / Mandate and
would be a rewrite rather than a port in a different domain.

### What was then removed

Once the positioning was retired, the components that existed only to serve it were
removed from the tree — 5,847 lines: `kernel/cmd-kernel` (the intent scheduler), all of
`services/`, `agent/alios`, all of `capabilities/`, the `cli`, and the `shell/` shim.

The architecture documentation that described them moved here as
`docs/archive/05-architecture/` — 119 files, each carrying an ARCHIVED header.

Nothing is lost. Every removed path is recoverable:

```bash
git checkout pre-cmdcapital-trim -- <path>
```

The tag is pushed to origin, so it survives this machine.

### What was kept, and one thing deliberately not removed

`cmd-types`, `cmd-transaction`, `cmd-policy`, `cmd-ledger`, `cmd-approval`, `cmd-proof`,
`cmd-shadow`, the schemas and the prototype behavior contracts.

`cmd-transaction` and `cmd-shadow` would not be used by a trading product — trading has
no rollback and a live market cannot be forked. They are kept anyway because removing
them is the highest-regret move available and no decision yet forces it.

### What was not done

No RFC was withdrawn. RFC-0024 (cmdCapital) remains a Draft that blocks its own
implementation. The kernel crates that were kept were not modified.
