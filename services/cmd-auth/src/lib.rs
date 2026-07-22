//! # cmd-auth — cmdOS access control
//!
//! Two gates guard entry to cmdOS:
//! 1. **Credentials** — a username and password (the preview defaults users type
//!    are `admin` / `cmdOS`). Passwords are never stored in plaintext.
//! 2. **Access key** — a `CMDOS-XXXX-XXXX-XXXX` key issued by the cmdOS website.
//!    Not everyone has one; this is what gates who may run cmdOS at all.
//!
//! Key verification sits behind a [`KeyVerifier`] trait: a `LocalVerifier`
//! (checks against a known set — offline, testable, for dev) ships now, and a
//! remote verifier that calls the cmdOS server (revocable, online) plugs into the
//! same interface later without changing the login flow.
//!
//! Defined by RFC-0017.

use sha2::{Digest, Sha256};
use std::collections::HashSet;

/// A parsed cmdOS access key of the form `CMDOS-XXXX-XXXX-XXXX`, where each `X`
/// is an uppercase letter or digit.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AccessKey(String);

impl AccessKey {
    /// Parse and validate a key string. Accepts surrounding whitespace and
    /// lowercase input (normalized to uppercase).
    pub fn parse(raw: &str) -> Result<AccessKey, AuthError> {
        let s = raw.trim().to_uppercase();
        let parts: Vec<&str> = s.split('-').collect();
        // Expect: CMDOS + three 4-char groups.
        if parts.len() != 4 || parts[0] != "CMDOS" {
            return Err(AuthError::MalformedKey);
        }
        for group in &parts[1..] {
            if group.len() != 4 || !group.chars().all(|c| c.is_ascii_alphanumeric()) {
                return Err(AuthError::MalformedKey);
            }
        }
        Ok(AccessKey(s))
    }

    /// The canonical string form.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for AccessKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Generate a random access key from a seed (deterministic given the seed, so it
/// is testable). The website uses this to mint keys. Not cryptographically
/// strong on its own — the server keeps the authoritative list.
pub fn generate_key(seed: u64) -> AccessKey {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut state = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let mut next = || {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ALPHABET[((state >> 33) as usize) % ALPHABET.len()] as char
    };
    let group = |next: &mut dyn FnMut() -> char| -> String { (0..4).map(|_| next()).collect() };
    let key = format!(
        "CMDOS-{}-{}-{}",
        group(&mut next),
        group(&mut next),
        group(&mut next)
    );
    AccessKey(key)
}

/// Decides whether an access key is currently valid. Implementations may check a
/// local set (offline) or call the cmdOS server (online, revocable).
pub trait KeyVerifier {
    /// Returns Ok(()) if the key is valid and active, or an [`AuthError`].
    fn verify(&self, key: &AccessKey) -> Result<(), AuthError>;
}

/// An offline verifier backed by a known set of valid keys. Suitable for dev and
/// tests; the production verifier calls the server instead.
#[derive(Default)]
pub struct LocalVerifier {
    valid: HashSet<String>,
}

impl LocalVerifier {
    /// A verifier with no valid keys.
    pub fn new() -> Self {
        LocalVerifier {
            valid: HashSet::new(),
        }
    }

    /// Add a key string to the valid set (parsed/normalized). Ignores malformed.
    pub fn allow(&mut self, key: &str) {
        if let Ok(k) = AccessKey::parse(key) {
            self.valid.insert(k.0);
        }
    }

    /// Number of valid keys registered.
    pub fn len(&self) -> usize {
        self.valid.len()
    }

    /// Whether the set is empty.
    pub fn is_empty(&self) -> bool {
        self.valid.is_empty()
    }
}

impl KeyVerifier for LocalVerifier {
    fn verify(&self, key: &AccessKey) -> Result<(), AuthError> {
        if self.valid.contains(&key.0) {
            Ok(())
        } else {
            Err(AuthError::KeyNotRecognized)
        }
    }
}

/// The verdict the cmdOS key server returns for a key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyStatus {
    /// Valid and active.
    Valid,
    /// Not found in the server's list.
    Unknown,
    /// Was issued but has been revoked.
    Revoked,
}

/// The one impure operation for online verification: ask the cmdOS server about
/// a key. Behind a trait so `RemoteVerifier` is testable with a fake, and a real
/// HTTP transport (in the app / a service crate) plugs in unchanged.
pub trait KeyCheckTransport {
    /// Ask the server for the status of `key`. Errors are transport failures.
    fn check(&self, key: &AccessKey) -> Result<KeyStatus, AuthError>;
}

/// An online verifier: defers the decision to the cmdOS server via a
/// [`KeyCheckTransport`]. This is the revocable, production path — the server
/// holds the authoritative list, so a key can be disabled at any time.
pub struct RemoteVerifier<T: KeyCheckTransport> {
    transport: T,
}

impl<T: KeyCheckTransport> RemoteVerifier<T> {
    /// Build a remote verifier over the given transport.
    pub fn new(transport: T) -> Self {
        RemoteVerifier { transport }
    }
}

impl<T: KeyCheckTransport> KeyVerifier for RemoteVerifier<T> {
    fn verify(&self, key: &AccessKey) -> Result<(), AuthError> {
        match self.transport.check(key)? {
            KeyStatus::Valid => Ok(()),
            KeyStatus::Unknown => Err(AuthError::KeyNotRecognized),
            KeyStatus::Revoked => Err(AuthError::KeyRevoked),
        }
    }
}

/// A username + password pair. The password is hashed immediately; the plaintext
/// is never retained.
#[derive(Debug, Clone)]
pub struct Credentials {
    pub username: String,
    password_hash: String,
}

impl Credentials {
    /// Build credentials, hashing the password. The preview defaults users type
    /// are `admin` / `cmdOS`.
    pub fn new(username: impl Into<String>, password: &str) -> Self {
        Credentials {
            username: username.into(),
            password_hash: hash(password),
        }
    }

    /// Whether a supplied password matches.
    pub fn password_matches(&self, password: &str) -> bool {
        // Constant-ish comparison on fixed-length hex hashes.
        self.password_hash == hash(password)
    }
}

/// A successful login session. Carries an opaque token and the username.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub token: String,
    pub username: String,
}

/// Attempt to log in: check the password, then the access key via the verifier.
/// Both must pass. Returns a [`Session`] or a specific [`AuthError`].
pub fn login(
    credentials: &Credentials,
    password_attempt: &str,
    key_input: &str,
    verifier: &dyn KeyVerifier,
) -> Result<Session, AuthError> {
    if !credentials.password_matches(password_attempt) {
        return Err(AuthError::BadPassword);
    }
    let key = AccessKey::parse(key_input)?;
    verifier.verify(&key)?;
    Ok(Session {
        // Token binds username + key; opaque to callers.
        token: hash(&format!("{}:{}", credentials.username, key.as_str())),
        username: credentials.username.clone(),
    })
}

/// SHA-256 hex of the input. Note: for a production password store this should be
/// a slow, salted KDF (argon2/bcrypt); SHA-256 is a placeholder to avoid pulling
/// a heavy dependency now. Tracked for upgrade in RFC-0017.
fn hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let digest = hasher.finalize();
    let mut s = String::with_capacity(digest.len() * 2);
    use std::fmt::Write;
    for b in digest {
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// Everything that can go wrong during auth.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthError {
    /// The key is not in the `CMDOS-XXXX-XXXX-XXXX` format.
    MalformedKey,
    /// The key is well-formed but not recognized (unknown key).
    KeyNotRecognized,
    /// The key was valid but has been revoked by cmdOS.
    KeyRevoked,
    /// The password did not match.
    BadPassword,
    /// Could not reach or understand the verification server.
    Transport(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let m = match self {
            AuthError::MalformedKey => "access key is not in CMDOS-XXXX-XXXX-XXXX format",
            AuthError::KeyNotRecognized => "access key is not recognized",
            AuthError::KeyRevoked => "access key has been revoked",
            AuthError::BadPassword => "incorrect password",
            AuthError::Transport(m) => return write!(f, "verification server error: {m}"),
        };
        write!(f, "{m}")
    }
}

impl std::error::Error for AuthError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_valid_key_case_insensitively() {
        let k = AccessKey::parse("  cmdos-ab12-cd34-ef56  ").unwrap();
        assert_eq!(k.as_str(), "CMDOS-AB12-CD34-EF56");
    }

    #[test]
    fn rejects_malformed_keys() {
        assert!(AccessKey::parse("CMDOS-ABC-DEF-GHI").is_err()); // wrong length
        assert!(AccessKey::parse("NOPE-AB12-CD34-EF56").is_err()); // wrong prefix
        assert!(AccessKey::parse("CMDOS-AB12-CD34").is_err()); // too few groups
        assert!(AccessKey::parse("CMDOS-AB1!-CD34-EF56").is_err()); // bad char
    }

    #[test]
    fn generated_keys_are_well_formed() {
        let k = generate_key(42);
        // Round-trips through the parser => correct format.
        assert!(AccessKey::parse(k.as_str()).is_ok());
        assert!(k.as_str().starts_with("CMDOS-"));
    }

    #[test]
    fn local_verifier_accepts_known_and_rejects_unknown() {
        let mut v = LocalVerifier::new();
        v.allow("CMDOS-AB12-CD34-EF56");
        assert_eq!(v.len(), 1);

        let good = AccessKey::parse("CMDOS-AB12-CD34-EF56").unwrap();
        let bad = AccessKey::parse("CMDOS-0000-0000-0000").unwrap();
        assert!(v.verify(&good).is_ok());
        assert_eq!(v.verify(&bad), Err(AuthError::KeyNotRecognized));
    }

    #[test]
    fn password_is_hashed_not_stored_plaintext() {
        let c = Credentials::new("admin", "cmdOS");
        assert!(c.password_matches("cmdOS"));
        assert!(!c.password_matches("wrong"));
    }

    #[test]
    fn login_succeeds_with_right_password_and_valid_key() {
        let c = Credentials::new("admin", "cmdOS");
        let mut v = LocalVerifier::new();
        v.allow("CMDOS-AB12-CD34-EF56");

        let session = login(&c, "cmdOS", "CMDOS-AB12-CD34-EF56", &v).unwrap();
        assert_eq!(session.username, "admin");
        assert!(!session.token.is_empty());
    }

    #[test]
    fn login_fails_on_bad_password() {
        let c = Credentials::new("admin", "cmdOS");
        let mut v = LocalVerifier::new();
        v.allow("CMDOS-AB12-CD34-EF56");
        let r = login(&c, "wrong", "CMDOS-AB12-CD34-EF56", &v);
        assert_eq!(r.unwrap_err(), AuthError::BadPassword);
    }

    #[test]
    fn login_fails_on_unknown_key_even_with_right_password() {
        let c = Credentials::new("admin", "cmdOS");
        let v = LocalVerifier::new(); // no keys allowed
        let r = login(&c, "cmdOS", "CMDOS-AB12-CD34-EF56", &v);
        assert_eq!(r.unwrap_err(), AuthError::KeyNotRecognized);
    }

    #[test]
    fn login_fails_on_malformed_key() {
        let c = Credentials::new("admin", "cmdOS");
        let v = LocalVerifier::new();
        let r = login(&c, "cmdOS", "not-a-key", &v);
        assert_eq!(r.unwrap_err(), AuthError::MalformedKey);
    }

    // ---- RemoteVerifier (online path) via a fake transport -----------------

    struct FakeServer(KeyStatus);
    impl KeyCheckTransport for FakeServer {
        fn check(&self, _key: &AccessKey) -> Result<KeyStatus, AuthError> {
            Ok(self.0)
        }
    }

    struct DownServer;
    impl KeyCheckTransport for DownServer {
        fn check(&self, _key: &AccessKey) -> Result<KeyStatus, AuthError> {
            Err(AuthError::Transport("connection refused".into()))
        }
    }

    #[test]
    fn remote_verifier_accepts_valid_key() {
        let v = RemoteVerifier::new(FakeServer(KeyStatus::Valid));
        let k = AccessKey::parse("CMDOS-AB12-CD34-EF56").unwrap();
        assert!(v.verify(&k).is_ok());
    }

    #[test]
    fn remote_verifier_rejects_unknown_and_revoked() {
        let unknown = RemoteVerifier::new(FakeServer(KeyStatus::Unknown));
        let revoked = RemoteVerifier::new(FakeServer(KeyStatus::Revoked));
        let k = AccessKey::parse("CMDOS-AB12-CD34-EF56").unwrap();
        assert_eq!(unknown.verify(&k), Err(AuthError::KeyNotRecognized));
        assert_eq!(revoked.verify(&k), Err(AuthError::KeyRevoked));
    }

    #[test]
    fn remote_verifier_surfaces_transport_errors() {
        let v = RemoteVerifier::new(DownServer);
        let k = AccessKey::parse("CMDOS-AB12-CD34-EF56").unwrap();
        assert!(matches!(v.verify(&k), Err(AuthError::Transport(_))));
    }

    #[test]
    fn login_works_through_a_remote_verifier() {
        let c = Credentials::new("admin", "cmdOS");
        let v = RemoteVerifier::new(FakeServer(KeyStatus::Valid));
        let session = login(&c, "cmdOS", "CMDOS-AB12-CD34-EF56", &v).unwrap();
        assert_eq!(session.username, "admin");
    }
}
