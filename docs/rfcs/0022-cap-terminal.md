# RFC-0022: cap-terminal — The Shell Capability

Version: 1.0
Status: Accepted
Category: Capability
Author: Lead Architect
Depends on: RFC-0004, RFC-0008 (Transaction Engine), RFC-0012 (AIPC)
Implemented by: `capabilities/terminal`

---

# 1. Summary

Lets an agent run shell commands. This is the most dangerous thing cmdOS can hand
an agent — a shell command can do anything — so almost all of this capability is
about deciding *how* dangerous a command is, not about running it.

# 2. Risk by allowlist, defaulting to the strictest reading

- Known read-only commands (`ls`, `pwd`, `cat`, `git status`, `npm outdated`) → **R0**.
- `mkdir` → **R1**: the one shell command that can be cleanly undone.
- Known changing commands (`cp`, `mv`, `git commit`, `cargo build`) → **R2**.
- Deletion, elevation, formatting, networking, scheduling → **R3**, always.
- **Everything else → R3.** Unknown means irreversible. An agent cannot walk past
  the supervisor by invoking a binary the table has never heard of.

Subcommands are read separately: `git log` looks, `git push` does not.

# 3. Shell metacharacters escalate

`echo hi && rm -rf /` starts with a harmless word. So any text containing
`| && || ; > >> < $( \` &` or a newline is classified R3 without further
analysis. Parsing a shell to find out what it will really do is a losing game;
declining to guess is the safe move.

# 4. Reversibility, honestly

Most shell commands cannot be undone, and the snapshot says so rather than
pretending. `mkdir` is the exception, and its undo removes the directory **only
if it is still empty** — a folder that now holds output belongs to whoever put it
there.

A command that exits non-zero is an error, not a completed step.

# 5. Design

`TerminalBackend` is the impure surface (spawning a process, platform-specific),
so classification — the part that matters — is tested exhaustively against a
fake. `Terminal<B>` implements `Resource`, so shell steps run through the same
engine, gate and ledger as everything else.

In the AIPC catalog the shell is a single tool registered at R3, because the
catalog cannot know which command will be passed; the capability narrows the
class per command at run time.

# 6. Testing

13 tests in cap-terminal plus one in aipc, all green, no warnings: read-only
commands; subcommand-decides-risk; deletion always gated; elevation and network
gated; **a command cannot hide behind a safe first word**; unknown commands
gated; empty command refused; no snapshot for reads; output recorded and
verified; non-zero exit is a failure; mkdir undone; undo declines a directory
that now has contents; simulate states the consequence plainly.

# 7. Next

A real backend spawning the platform shell (cmd on Windows, sh elsewhere), with
a working directory confined to the agent's machine. Then the Terminal app can
run commands rather than only display the agent's narration.
