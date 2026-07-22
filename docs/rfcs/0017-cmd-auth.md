# RFC-0017: cmd-auth — Access Control (Credentials + Access Key)

Version: 1.0
Status: Accepted
Category: Services (Machine / auth)
Author: Lead Architect
Depends on: RFC-0004
Implemented by: `services/cmd-auth`

---

# 1. Summary

The entry gate to cmdOS. Two checks guard access:
1. **Credentials** — username + password. The preview defaults a user types are
   `admin` / `cmdOS`. Passwords are hashed, never stored in plaintext.
2. **Access key** — a `CMDOS-XXXX-XXXX-XXXX` key issued by the cmdOS website.
   Not everyone has one; this gates who may run cmdOS at all.

# 2. Model

- `AccessKey::parse` validates the `CMDOS-XXXX-XXXX-XXXX` format (case-insensitive,
  trims whitespace). `generate_key(seed)` mints well-formed keys (the website's
  minting primitive).
- `KeyVerifier` trait decides validity. `LocalVerifier` checks a known set
  (offline, testable, for dev). A remote verifier that calls the cmdOS server
  (revocable, online) implements the same trait later — the login flow does not
  change.
- `Credentials::new` hashes the password. `login(credentials, password, key,
  verifier)` requires BOTH the password and a valid key, returning a `Session`
  (opaque token + username) or a specific `AuthError`
  (BadPassword / MalformedKey / KeyNotRecognized).

# 3. Online vs Offline (design intent)

The key model is built for an **online, revocable** future: the website mints
keys and the server holds the authoritative list, so a key can be revoked at any
time. `LocalVerifier` is the offline stand-in that makes the whole flow run and
test today; swapping in a `RemoteVerifier` is the only change needed to go live.

# 4. Security Note (honest)

Passwords are hashed with SHA-256 as a placeholder to avoid a heavy dependency
now. A production store must use a slow, salted KDF (argon2/bcrypt). This is a
known upgrade, tracked here — not a claim of production-grade password security.

# 5. Testing

9 tests, all green, no warnings: key parse (valid/case-insensitive), malformed
rejection, generated-key format, local verifier accept/reject, password hashing,
login success, and the three failure paths (bad password, unknown key, malformed
key).

# 5b. RemoteVerifier (online, revocable) — added

`RemoteVerifier<T: KeyCheckTransport>` defers to the cmdOS server: the transport
returns `KeyStatus` (Valid / Unknown / Revoked), mapped to Ok / KeyNotRecognized /
KeyRevoked. The HTTP call is behind `KeyCheckTransport`, so it is tested with a
fake server; a real HTTP transport plugs in unchanged. This is the production
path — the server holds the authoritative, revocable list.

The app↔server protocol is specified in `docs/07-product/cmdos-key-server-spec.md`.

# 5c. Plans & expiry (added)

Keys carry a `Plan` (Free 7d / Month $15 / SixMonths $75 / Year $125 / Custom).
`KeyRecord { plan, issued_day }` computes `expiry_day`, `is_valid_on`,
`days_remaining`, and `is_expiring_soon` (renewal nudges), all deterministic
(integer day counts, no wall clock). Expired keys report `KeyStatus::Expired` →
`AuthError::KeyExpired`. Free-tier abuse prevention is a website responsibility,
documented in the key-server spec.

# 6. Next

Wire the login gate into the `cmdos` CLI (prompt username → password → key before
the agent runs); then a `RemoteVerifier` + the cmdOS key website; then the rest of
the Machine (VM, streaming).
