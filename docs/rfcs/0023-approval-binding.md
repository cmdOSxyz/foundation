# RFC-0023: Approval Binding

Version: 1.0
Status: Accepted
Category: Kernel (security)
Author: Lead Architect
Depends on: RFC-0004, RFC-0010 (Policy Gate)
Implemented by: `kernel/cmd-approval`

---

# 1. Summary

Before this, an approval was a decision: a person said yes, and the kernel
remembered that a yes had happened. Nothing tied the yes to *what* was approved.
Three attacks followed, none of them detectable:

- **Substitution** — approve "list /home", execute "delete /home". The approval
  did not know which plan it belonged to.
- **Drift** — approve after seeing a folder, then the folder changes before the
  plan runs. The approval was granted against a world that no longer exists.
- **Replay** — approve once, execute twice.

An approval is now bound to two digests, an expiry, and a single-use guard. This
is the contract the README states:

```text
approval = exact plan hash + exact pre-state
state drift = approval invalidation
```

# 2. The plan digest

Covers, per step: capability, action, parameters, dependency **positions**, and
the permission flag. Plus the step count, so a plan cannot be extended or
truncated without changing the digest. Step order is included, because deleting
then writing is not writing then deleting.

Deliberately **excluded**: step ids, descriptions, status.

That exclusion is load-bearing rather than convenient. Ids are freshly generated
on every planning run, so hashing them would mean no approval could ever match
the plan it was granted for. Status changes as a plan executes, which is exactly
when the digest must stay stable. Descriptions are prose for the human and carry
no authority.

Every field is written length-prefixed. Without that,
`capability="file", action="system.list"` and
`capability="filesystem", action="list"` hash identically, and whoever controls
the strings chooses where the boundary falls.

## 2b. The consequence, stated

Two plans differing only in wording produce the same digest, so an approval for
one is valid for the other. That is correct — they authorize identical work — but
it means the description shown to a person must be **derived from the parameters**,
not supplied alongside them, or the screen could say "tidy up some old files"
while authorizing a delete. `ApprovalRequest::describe` builds the summary from
the same fields the digest covers, and there is a test asserting the planner's
prose never reaches the screen.

# 3. The pre-state digest

What goes in is the caller's decision, because only the caller knows which facts
a plan depends on. The contract: it must cover everything the plan reads or
overwrites. A digest that omits a file the plan will delete cannot detect that the
file changed.

Absence is a fact. `observe_absent` exists so that a plan creating `report.pdf`
fails its approval if someone else created it in the meantime.

An **empty** observation is refused outright (`Invalid::NoStateObserved`) rather
than treated as a match. An empty state matches everything, which is worse than
having no check because it looks like one.

# 4. Authorization

`ApprovalGate::authorize` returns a distinct error per failure, because
"approve again" and "that is the wrong plan" call for very different responses.
Checks run in order: already-used, expired, wrong plan, no state, drifted. A
spent approval reports as spent even if the plan also changed, since that is the
actionable half.

Single use is enforced by the gate, not the caller — "did we already spend this"
is the bookkeeping a caller forgets under pressure. `authorize_once` consumes on
success so a crash between authorizing and running cannot leave it reusable.

# 5. Not in this RFC

- **Signatures.** `Approval::approver` is carried for the ledger, not verified.
  Cryptographic approver identity belongs with paired-device approval (Phase 2).
- **Automatic observation.** Capabilities do not yet report what they will touch,
  so callers assemble the observation by hand. Effect Manifest v0 is what makes
  that automatic, and until it lands an incomplete observation is a real risk:
  the gate can only check the facts it was given.

# 6. Testing

19 tests, no warnings. The digest tests cover: stability across fresh ids and
edited prose, sensitivity to parameters, action, step count, step order, and
dependency shape, and resistance to field-boundary shifting. The gate tests cover
substitution, drift in both directions, refusal of an empty observation, expiry,
single use, error ordering, and that the summary is built from hashed fields.

# 7. Next

Wire the gate into `cmd-kernel` so an R3 step cannot execute without a valid
approval, and have `cap-files` and `cap-terminal` report the facts they touch so
the observation is assembled from the capability rather than by hand.
