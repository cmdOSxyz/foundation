# RFC-0005: The Shadow World Engine

Version: 1.0
Status: Accepted (filesystem scope)
Category: Architecture (flagship)
Author: Lead Architect
Depends on: RFC-0004
Implemented by: `kernel/cmd-shadow`

---

# 1. Summary

Before the agent touches the real machine, fork it. The agent finishes the whole
plan inside the fork; the user is shown a *finished outcome* and chooses whether
it becomes reality. Discarding costs nothing, because nothing real ever changed.

Reversibility, dry-run and undo are special cases of this: a dry-run is a fork you
never promote, an undo is a promotion you decline.

# 2. Copy-on-write

A fork starts empty and copies nothing. Reads fall through to reality; the first
*write* to a path puts a copy in the fork and the write lands on the copy. So
forking a large folder is instant, and cost is proportional to what changed, not
to what exists.

Several forks can hang off one root at once — the agent finishes the same job
three ways and the user picks. That is the product.

# 3. API

- `ShadowFork::new(root, work, label)` — a fork; nothing copied.
- `read` / `write` / `delete` / `materialize` — work inside the fork.
- `changes() -> Vec<Change>` — Created / Modified / Deleted, relative to reality.
  Writing bytes identical to reality produces **no** change, so the chooser is
  never offered an outcome that does nothing.
- `promote()` — apply every change to the root. `discard()` — free, and the
  reason the agent can be allowed to try things at all.
- `ShadowWorld::fork / outcomes / choose / discard_all` — several candidate
  futures over one root; `choose(id)` promotes one and discards the rest.

# 4. Containment

A shadow that can reach reality is not a shadow. Every path is validated before
use: absolute paths, `..` traversal and root prefixes are rejected
(`ShadowError::Escapes`). A fork can only ever write beneath its own work
directory, and only promotion moves anything into the root.

# 5. Scope

This implementation is scoped to the filesystem. At VM level the same model spans
the whole computer — files, browser sessions, open apps — which remains this
RFC's eventual target. The API is deliberately shaped so that a VM-backed
implementation can satisfy it without changing callers.

# 6. Testing

11 tests, all green, no warnings: forking copies nothing; reads fall through;
writes leave reality alone; promote applies; discard is free; deletion is staged
until promotion; an identical rewrite is not a change; escape attempts are
refused; nested paths; walking away; and `choose_your_future` — the same job
finished three ways, one chosen, the others gone.

# 6b. Running plans in a fork (added)

`shell-core::Machine::run_in_shadow(text, root, label, scratch)` plans an intent
and executes it **inside a fork**. Step path parameters are rewritten to point at
the fork, materialising originals on the way in, so the capability and the kernel
are untouched — they simply operate on ground that is not real yet. The same
policy gate and the same ledger apply.

It returns a finished `OutcomeInfo`: the plan step by step, and what promoting it
would do to the real folder in plain words. `shadow_outcomes` /
`shadow_choose` / `shadow_discard` are the chooser.

A path that is absolute and outside the root is refused rather than rewritten:
an agent working in a shadow must not be able to reach past it.

# 6c. Path form

A `Change` crosses into the UI and can come back into the engine, so its path is
always canonical: forward slashes, whatever the platform. Input accepts either
separator, and a drive-qualified path (`C:\...`) is refused like any other
absolute path. This was found by the first Windows run — the engine was reporting
the host's separator, which would have made a change produced on one machine
unreadable on another.

# 7. Next

Run whole plans inside a fork through the kernel (rooting `cap-files` at the fork
rather than reality), surface outcomes in the Shadow app, then take the model to
VM level.
