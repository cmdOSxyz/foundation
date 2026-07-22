//! Routed transport: bridge between the [`KeyRouter`](cmd_router::KeyRouter) and
//! the model call.
//!
//! On each planning request it asks the router for the next usable key
//! (round-robin, honoring per-key limits), then delegates the actual HTTPS call
//! to an inner transport built for that key. When the router reports every key is
//! exhausted, it returns a clear error so the agent can warn the user to add or
//! replace a key — it never silently stops.
//!
//! The per-key transport is produced by a factory closure, so the HTTP client
//! (claude-http) stays out of this crate and the whole routing path is testable
//! with a fake.

use crate::claude_planner::{ClaudeTransport, PlanError};
use cmd_router::{KeyRouter, Selection};
use std::cell::RefCell;

/// Builds a [`ClaudeTransport`] for a specific API key secret. In production this
/// returns an HTTP transport keyed to that secret; in tests, a fake.
pub type TransportFactory<'a> = dyn Fn(&str) -> Box<dyn ClaudeTransport> + 'a;

/// A transport that routes each call through a [`KeyRouter`]. Holds the router
/// mutably (via `RefCell`) because selecting a key meters usage, while
/// `ClaudeTransport::send` takes `&self`.
pub struct RoutedTransport<'a> {
    router: RefCell<KeyRouter>,
    make_transport: Box<TransportFactory<'a>>,
}

impl<'a> RoutedTransport<'a> {
    /// Build a routed transport over `router`, using `make_transport` to create a
    /// per-key transport for the chosen key's secret.
    pub fn new(
        router: KeyRouter,
        make_transport: impl Fn(&str) -> Box<dyn ClaudeTransport> + 'a,
    ) -> Self {
        RoutedTransport {
            router: RefCell::new(router),
            make_transport: Box::new(make_transport),
        }
    }

    /// Snapshot of the pool's key stats, for the app's dashboard.
    pub fn stats(&self) -> Vec<cmd_router::KeyStat> {
        self.router.borrow().stats()
    }

    /// Total remaining requests across all keys.
    pub fn total_remaining(&self) -> u64 {
        self.router.borrow().total_remaining()
    }
}

impl ClaudeTransport for RoutedTransport<'_> {
    fn send(&self, system: &str, user_text: &str) -> Result<String, PlanError> {
        // Pick the next usable key (meters one request against it).
        let (secret, _key_id) = {
            let mut router = self.router.borrow_mut();
            match router.next_key() {
                Selection::Use(id) => {
                    let secret = router
                        .secret_of(id)
                        .ok_or_else(|| PlanError::Transport("selected key vanished".into()))?
                        .to_string();
                    (secret, id)
                }
                Selection::AllExhausted => {
                    return Err(PlanError::Transport(
                        "all API keys are exhausted — add or replace a key to continue".into(),
                    ))
                }
                Selection::Empty => {
                    return Err(PlanError::Transport(
                        "no API keys added — add one of your keys to continue".into(),
                    ))
                }
            }
        };

        // Build a transport for that key and delegate the real call.
        let inner = (self.make_transport)(&secret);
        inner.send(system, user_text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cmd_router::{ApiKey, KeyRouter};

    /// A fake per-key transport that echoes which key secret it was built with,
    /// so tests can assert routing behavior without a network.
    struct FakeInner {
        secret: String,
    }
    impl ClaudeTransport for FakeInner {
        fn send(&self, _system: &str, _user: &str) -> Result<String, PlanError> {
            // Return a JSON reply naming the key used (via its length as a proxy).
            Ok(format!(
                "{{ \"reply\": \"used {}\", \"plan\": null }}",
                self.secret
            ))
        }
    }

    fn router_with(keys: &[(&str, u64)]) -> KeyRouter {
        let mut r = KeyRouter::new();
        for (secret, limit) in keys {
            r.add(ApiKey::new(*secret, *limit));
        }
        r
    }

    #[test]
    fn routes_a_call_to_a_key_and_meters_it() {
        let router = router_with(&[("sk-ant-a", 5)]);
        let routed = RoutedTransport::new(router, |secret| {
            Box::new(FakeInner {
                secret: secret.to_string(),
            })
        });

        let out = routed.send("sys", "hi").unwrap();
        assert!(out.contains("used sk-ant-a"));
        // One request metered; four remain.
        assert_eq!(routed.total_remaining(), 4);
    }

    #[test]
    fn rotates_across_keys_round_robin() {
        let router = router_with(&[("sk-ant-a", 5), ("sk-b", 5)]);
        let routed = RoutedTransport::new(router, |secret| {
            Box::new(FakeInner {
                secret: secret.to_string(),
            })
        });

        let first = routed.send("s", "u").unwrap();
        let second = routed.send("s", "u").unwrap();
        // Two different keys served the two calls.
        assert_ne!(first, second);
        assert_eq!(routed.total_remaining(), 8);
    }

    #[test]
    fn errors_clearly_when_all_keys_exhausted() {
        let router = router_with(&[("sk-ant-a", 1)]);
        let routed = RoutedTransport::new(router, |secret| {
            Box::new(FakeInner {
                secret: secret.to_string(),
            })
        });

        // First call uses the only key's only request.
        assert!(routed.send("s", "u").is_ok());
        // Second call: exhausted → clear error mentioning add/replace.
        let err = routed.send("s", "u").unwrap_err();
        match err {
            PlanError::Transport(m) => assert!(m.contains("exhausted")),
            other => panic!("expected transport error, got {other:?}"),
        }
    }

    #[test]
    fn errors_clearly_when_no_keys_added() {
        let routed = RoutedTransport::new(KeyRouter::new(), |secret| {
            Box::new(FakeInner {
                secret: secret.to_string(),
            })
        });
        let err = routed.send("s", "u").unwrap_err();
        match err {
            PlanError::Transport(m) => assert!(m.contains("no API keys")),
            other => panic!("expected transport error, got {other:?}"),
        }
    }
}
