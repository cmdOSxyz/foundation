# cmdOS Key Server — Protocol Spec

The contract between the cmdOS app (`RemoteVerifier`) and the cmdOS key website.
The website mints `CMDOS-XXXX-XXXX-XXXX` keys, holds the authoritative revocable
list, and exposes one verification endpoint the app calls.

## Endpoint

`POST /api/keys/verify`

Request body (JSON):
```json
{ "key": "CMDOS-AB12-CD34-EF56" }
```

Response body (JSON), HTTP 200 in all three cases:
```json
{ "status": "valid" }      // key exists and is active
{ "status": "unknown" }    // key not in the list
{ "status": "revoked" }    // key existed but was disabled
```

The app maps: valid → access granted; unknown → KeyNotRecognized;
revoked → KeyRevoked. Any non-200 / malformed body → transport error (access
denied, with a retry hint).

## Minting (admin side, website only)

- An admin signs into the website and generates keys (format
  `CMDOS-XXXX-XXXX-XXXX`, uppercase A–Z/0–9).
- Each key row stores: the key, an issued-to label, created-at, and a
  `revoked` boolean.
- Revoking = set `revoked = true`; the verify endpoint then returns `revoked`.
- Keys are NOT for everyone: the admin issues them selectively.

## Security notes

- The verify endpoint should rate-limit by IP to resist key guessing.
- Keys are random, not guessable; the server list is authoritative.
- Over HTTPS only.

## Reference implementation target

The website repo (`cmdOSxyz/website`, React + Vite) hosts the admin UI; the verify
endpoint can be a serverless function / small API. The Rust side is done: a
`KeyCheckTransport` that POSTs to this endpoint and parses `status` satisfies
`RemoteVerifier` with no further changes to the login flow.

## Plans & expiry (added)

The website mints keys on one of these plans; each sets the key's lifetime.
Prices are reference (USD):

| Plan | Price | Duration | On expiry |
|---|---|---|---|
| Free | $0 | 7 days | key auto-expires; agent cannot be used |
| 1 month | $15 | 30 days | app shows a renewal prompt; no renew → key locked |
| 6 months | $75 | 180 days | (same renewal behavior) |
| 12 months | $125 | 365 days | (same) |
| Custom | — | admin-set days | (same) |

The verify endpoint returns `expired` once a key is past its duration; the app
maps that to a locked state with a renewal call-to-action. When a key is within a
renewal window (e.g. 7 days of expiry), the app can nudge the user to renew.

### Verify response — expired case
```json
{ "status": "expired" }
```

## Free-tier abuse prevention (website responsibility)

Free 7-day keys are the main abuse target (one person minting many). Because
cmdOS is bring-your-own-key, free keys do NOT cost cmdOS provider tokens — the
user still uses their own model keys — so the pressure is lost revenue, not
runaway cost. Recommended layers, cheapest first; add more only when abuse is
observed:

1. **Email verification** — one free key per verified email; block disposable /
   temp-mail domains.
2. **Device / IP fingerprint** — flag many free keys from one device or IP (a
   signal, not a hard block; VPNs evade it).
3. **Phone OTP** — one free key per verified phone. The strongest cost-effective
   layer; add when email-only abuse appears.
4. **Card verification (no charge)** — near-eliminates abuse but reduces signups;
   only for scale / organized abuse.

Also **limit the VALUE of a free key** (e.g. cap agent runs, or read-only
capabilities during trial) so a minted free key isn't worth farming, while a real
user still gets enough to decide. This is often more effective than chasing
identities.

Do NOT position cmdOS as a way to bypass provider free-tier limits — frame the
router as transparent management of the user's own keys.
