//! # cmd-router — the API key router
//!
//! cmdOS is a **bring-your-own-key** tool: the user adds their own API keys, and
//! this router manages them. It detects each key's provider, meters usage against
//! a per-key limit, rotates round-robin across keys that still have headroom, and
//! moves to the next key automatically when one is exhausted. When every key is
//! spent, it stops and signals the user to add or replace a key.
//!
//! Positioning: cmdOS neither owns, provides, nor sells keys. It forwards the
//! user's requests to the keys the user supplied, like any API client. The user
//! is responsible for using their keys within their providers' terms. This crate
//! is a management and transparency layer over keys the user already holds — not
//! a means of evading provider limits.
//!
//! This crate is pure key-management logic (selection, metering, rotation), fully
//! tested. Actually sending a request with a chosen key is the transport's job
//! (e.g. claude-http), which lives behind the planner traits.
//!
//! Defined by RFC-0018.

use cmd_types::Id;

/// The provider a key belongs to, detected from its format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    Anthropic,
    OpenAI,
    Google,
    /// Recognized shape but unknown vendor, or a custom endpoint.
    Other,
}

impl Provider {
    /// Human label for display in the app.
    pub fn label(self) -> &'static str {
        match self {
            Provider::Anthropic => "Anthropic",
            Provider::OpenAI => "OpenAI",
            Provider::Google => "Google",
            Provider::Other => "Other",
        }
    }

    /// Detect the provider from a key string's format. Best-effort, based on
    /// well-known prefixes; unknown shapes map to `Other`.
    pub fn detect(key: &str) -> Provider {
        let k = key.trim();
        if k.starts_with("sk-ant-") {
            Provider::Anthropic
        } else if k.starts_with("sk-") {
            Provider::OpenAI
        } else if k.starts_with("AIza") {
            Provider::Google
        } else {
            Provider::Other
        }
    }
}

/// One API key the user added, with its meter. The raw secret is stored so the
/// transport can use it, but `masked()` is what the app displays.
#[derive(Debug, Clone)]
pub struct ApiKey {
    pub id: Id,
    secret: String,
    pub provider: Provider,
    /// A user-facing label, e.g. "Anthropic free tier #1".
    pub label: String,
    /// Max requests this key may serve (the user's understanding of their tier).
    pub request_limit: u64,
    /// Requests already served by this key.
    pub requests_used: u64,
}

impl ApiKey {
    /// Add a key: detect its provider, set a request limit, start the meter at 0.
    pub fn new(secret: impl Into<String>, request_limit: u64) -> Self {
        let secret = secret.into();
        let provider = Provider::detect(&secret);
        ApiKey {
            id: Id::new(),
            provider,
            label: format!("{} key", provider.label()),
            request_limit,
            requests_used: 0,
            secret,
        }
    }

    /// Set a custom display label.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// The raw secret, for the transport. Not shown in the UI.
    pub fn secret(&self) -> &str {
        &self.secret
    }

    /// A masked form for display, e.g. `sk-ant-…a1b2`.
    pub fn masked(&self) -> String {
        let s = &self.secret;
        if s.len() <= 10 {
            return "…".to_string();
        }
        let head: String = s.chars().take(7).collect();
        let tail: String = s
            .chars()
            .rev()
            .take(4)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        format!("{head}…{tail}")
    }

    /// Remaining requests before this key is exhausted.
    pub fn remaining(&self) -> u64 {
        self.request_limit.saturating_sub(self.requests_used)
    }

    /// Whether this key still has headroom.
    pub fn has_headroom(&self) -> bool {
        self.requests_used < self.request_limit
    }
}

/// A read-only snapshot of a key's meter, for display in the app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyStat {
    pub id: Id,
    pub provider: Provider,
    pub label: String,
    pub masked: String,
    pub used: u64,
    pub limit: u64,
    pub remaining: u64,
    pub active: bool,
}

/// The result of asking the router for a key to use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selection {
    /// Use the key with this id (the router has metered one request against it).
    Use(Id),
    /// Every key is exhausted — warn the user to add or replace a key.
    AllExhausted,
    /// No keys have been added yet.
    Empty,
}

/// The router: a pool of the user's keys, rotated round-robin with per-key limits.
#[derive(Default)]
pub struct KeyRouter {
    keys: Vec<ApiKey>,
    /// Rotation cursor for round-robin.
    cursor: usize,
}

impl KeyRouter {
    /// A new, empty router.
    pub fn new() -> Self {
        KeyRouter {
            keys: Vec::new(),
            cursor: 0,
        }
    }

    /// Add a key to the pool.
    pub fn add(&mut self, key: ApiKey) {
        self.keys.push(key);
    }

    /// Number of keys in the pool.
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Whether the pool is empty.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Total remaining requests across all keys.
    pub fn total_remaining(&self) -> u64 {
        self.keys.iter().map(|k| k.remaining()).sum()
    }

    /// Pick the next usable key round-robin and meter one request against it.
    ///
    /// Scans from the rotation cursor for the next key with headroom, advancing
    /// the cursor so load spreads evenly. Returns `AllExhausted` when no key has
    /// headroom (the caller should warn the user), or `Empty` if none were added.
    pub fn next_key(&mut self) -> Selection {
        if self.keys.is_empty() {
            return Selection::Empty;
        }
        let n = self.keys.len();
        for offset in 0..n {
            let idx = (self.cursor + offset) % n;
            if self.keys[idx].has_headroom() {
                self.keys[idx].requests_used += 1;
                // Advance the cursor past the chosen key for even spread.
                self.cursor = (idx + 1) % n;
                return Selection::Use(self.keys[idx].id);
            }
        }
        Selection::AllExhausted
    }

    /// The raw secret for a key id (for the transport). `None` if not found.
    pub fn secret_of(&self, id: Id) -> Option<&str> {
        self.keys.iter().find(|k| k.id == id).map(|k| k.secret())
    }

    /// Provider for a key id.
    pub fn provider_of(&self, id: Id) -> Option<Provider> {
        self.keys.iter().find(|k| k.id == id).map(|k| k.provider)
    }

    /// Full per-key stats for display in the app.
    pub fn stats(&self) -> Vec<KeyStat> {
        self.keys
            .iter()
            .map(|k| KeyStat {
                id: k.id,
                provider: k.provider,
                label: k.label.clone(),
                masked: k.masked(),
                used: k.requests_used,
                limit: k.request_limit,
                remaining: k.remaining(),
                active: k.has_headroom(),
            })
            .collect()
    }

    /// Remove a key by id (user deletes it). Returns true if removed.
    pub fn remove(&mut self, id: Id) -> bool {
        let before = self.keys.len();
        self.keys.retain(|k| k.id != id);
        if self.cursor >= self.keys.len() {
            self.cursor = 0;
        }
        self.keys.len() != before
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_providers_from_key_format() {
        assert_eq!(Provider::detect("sk-ant-abc123"), Provider::Anthropic);
        assert_eq!(Provider::detect("sk-proj-xyz"), Provider::OpenAI);
        assert_eq!(Provider::detect("AIzaSyABC"), Provider::Google);
        assert_eq!(Provider::detect("whatever"), Provider::Other);
    }

    #[test]
    fn masks_the_secret_for_display() {
        let k = ApiKey::new("sk-ant-secret1234", 100);
        let m = k.masked();
        assert!(m.starts_with("sk-ant-"));
        assert!(m.contains('…'));
        assert!(!m.contains("secret"));
    }

    #[test]
    fn meters_and_reports_remaining() {
        let mut r = KeyRouter::new();
        r.add(ApiKey::new("sk-ant-a", 3));
        let id = match r.next_key() {
            Selection::Use(id) => id,
            other => panic!("expected Use, got {other:?}"),
        };
        assert_eq!(r.stats()[0].used, 1);
        assert_eq!(r.stats()[0].remaining, 2);
        assert_eq!(r.provider_of(id), Some(Provider::Anthropic));
    }

    #[test]
    fn round_robin_spreads_across_keys() {
        let mut r = KeyRouter::new();
        r.add(ApiKey::new("sk-ant-a", 10));
        r.add(ApiKey::new("sk-b", 10));
        // Two calls should hit two different keys (round-robin).
        let first = r.next_key();
        let second = r.next_key();
        assert_ne!(first, second);
        // Each key metered once.
        assert!(r.stats().iter().all(|s| s.used == 1));
    }

    #[test]
    fn moves_to_next_key_when_one_is_exhausted() {
        let mut r = KeyRouter::new();
        r.add(ApiKey::new("sk-ant-a", 1)); // tiny limit
        r.add(ApiKey::new("sk-b", 5));
        // Exhaust: 3 calls total; key A can serve only 1, B serves the rest.
        let mut used = Vec::new();
        for _ in 0..3 {
            if let Selection::Use(id) = r.next_key() {
                used.push(id);
            }
        }
        assert_eq!(used.len(), 3);
        // Key A (limit 1) is now exhausted; B carried the overflow.
        let a = &r.stats()[0];
        assert!(!a.active, "exhausted key is inactive");
        assert_eq!(a.used, 1);
    }

    #[test]
    fn warns_when_all_keys_exhausted() {
        let mut r = KeyRouter::new();
        r.add(ApiKey::new("sk-ant-a", 1));
        r.add(ApiKey::new("sk-b", 1));
        // Consume both.
        assert!(matches!(r.next_key(), Selection::Use(_)));
        assert!(matches!(r.next_key(), Selection::Use(_)));
        // Now everything is spent → warn the user.
        assert_eq!(r.next_key(), Selection::AllExhausted);
        assert_eq!(r.total_remaining(), 0);
    }

    #[test]
    fn empty_router_reports_empty() {
        let mut r = KeyRouter::new();
        assert_eq!(r.next_key(), Selection::Empty);
        assert!(r.is_empty());
    }

    #[test]
    fn user_can_add_and_remove_keys() {
        let mut r = KeyRouter::new();
        let k = ApiKey::new("sk-ant-a", 5).with_label("my free tier");
        let id = k.id;
        r.add(k);
        assert_eq!(r.len(), 1);
        assert_eq!(r.stats()[0].label, "my free tier");
        assert!(r.remove(id));
        assert!(r.is_empty());
    }
}
