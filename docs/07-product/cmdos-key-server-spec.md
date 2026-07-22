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
