//! # cap-browser — the browser capability
//!
//! Lets an agent act on the web: navigate, read the page, fill fields, and submit
//! forms. This is where most real work lives, and where higher-risk actions
//! appear — so risk classification (R0–R3) matters most here:
//!
//! - `navigate`, `read` → **R0** (read-only; observe, change nothing external)
//! - `fill` a field → **R1** (fully reversible: clearing restores prior state)
//! - `submit` / `click_buy` → **R3** (irreversible external effect; human-gated)
//!
//! Driving a real browser (Chrome automation) is impure and can't run in CI, so
//! it sits behind a [`BrowserBackend`] trait. This crate's logic — action
//! parsing, risk classification, the reversible lifecycle — is fully tested with
//! a fake backend. A real backend (headless Chrome) plugs in unchanged.
//!
//! Defined by RFC-0020.

use cmd_transaction::{Resource, ResourceError, Snapshot};
use cmd_types::{PlanStep, RiskClass};

/// The risk class of a browser action. Central to safe web automation.
pub fn risk_of(action: &str) -> RiskClass {
    match action {
        "navigate" | "read" | "screenshot" => RiskClass::R0ReadOnly,
        // Filling a field is fully reversible: clearing it restores the prior
        // (empty) state, so it is R1, and the engine can auto-undo it.
        "fill" | "clear" => RiskClass::R1Reversible,
        // Anything that commits to the outside world is irreversible → human-gated.
        "submit" | "click_buy" | "confirm" | "pay" => RiskClass::R3Irreversible,
        _ => RiskClass::R2Compensable,
    }
}

/// The impure surface: actually driving a browser. A real implementation wraps a
/// headless Chrome; the fake in tests records calls.
pub trait BrowserBackend {
    /// Navigate to a URL. Returns the resulting page title (or a summary).
    fn navigate(&mut self, url: &str) -> Result<String, ResourceError>;
    /// Read text content matching a selector (or the whole page if empty).
    fn read(&self, selector: &str) -> Result<String, ResourceError>;
    /// Fill a form field identified by `selector` with `value`.
    fn fill(&mut self, selector: &str, value: &str) -> Result<(), ResourceError>;
    /// Clear a form field (the undo for `fill`).
    fn clear(&mut self, selector: &str) -> Result<(), ResourceError>;
    /// Submit a form / click a committing control.
    fn submit(&mut self, selector: &str) -> Result<(), ResourceError>;
    /// The current page's URL, used for verification.
    fn current_url(&self) -> String;
}

/// Snapshot for reversible browser steps. For a `fill` we remember the field so
/// undo can clear it. R0 reads take no snapshot; R3 submits are not reversible.
#[derive(Clone, Debug)]
pub enum BrowserSnapshot {
    /// A field was filled; undo clears it.
    Filled { selector: String },
    /// Nothing to undo.
    None,
}
impl Snapshot for BrowserSnapshot {}

/// The browser capability over a backend `B`.
pub struct Browser<B: BrowserBackend> {
    backend: B,
}

impl<B: BrowserBackend> Browser<B> {
    /// Build the capability over a backend.
    pub fn new(backend: B) -> Self {
        Browser { backend }
    }

    /// Borrow the backend (e.g. to read results after a run).
    pub fn backend(&self) -> &B {
        &self.backend
    }
}

fn param(step: &PlanStep, key: &str) -> String {
    step.parameters
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string()
}

impl<B: BrowserBackend> Resource for Browser<B> {
    type Snap = BrowserSnapshot;

    fn simulate(&self, step: &PlanStep) -> Result<String, ResourceError> {
        let a = step.action.as_str();
        let summary = match a {
            "navigate" => format!("Would navigate to {}", param(step, "url")),
            "read" => format!("Would read {} (read-only)", param(step, "selector")),
            "fill" => format!(
                "Would fill {} = {}",
                param(step, "selector"),
                param(step, "value")
            ),
            "submit" | "click_buy" | "confirm" | "pay" => format!(
                "Would {} {} — IRREVERSIBLE, needs approval",
                a,
                param(step, "selector")
            ),
            other => format!("Would {other}"),
        };
        Ok(summary)
    }

    fn snapshot(&self, step: &PlanStep) -> Result<Option<Self::Snap>, ResourceError> {
        match step.action.as_str() {
            "navigate" | "read" | "screenshot" => Ok(None), // R0: nothing to undo
            "fill" => Ok(Some(BrowserSnapshot::Filled {
                selector: param(step, "selector"),
            })),
            // R3 actions are not reversible; the engine will not auto-undo them,
            // and policy requires human approval before they run at all.
            _ => Ok(Some(BrowserSnapshot::None)),
        }
    }

    fn execute(&mut self, step: &PlanStep) -> Result<(), ResourceError> {
        match step.action.as_str() {
            "navigate" => {
                self.backend.navigate(&param(step, "url"))?;
                Ok(())
            }
            "read" => {
                self.backend.read(&param(step, "selector"))?;
                Ok(())
            }
            "fill" => {
                self.backend
                    .fill(&param(step, "selector"), &param(step, "value"))?;
                Ok(())
            }
            "clear" => {
                self.backend.clear(&param(step, "selector"))?;
                Ok(())
            }
            "submit" | "click_buy" | "confirm" | "pay" => {
                self.backend.submit(&param(step, "selector"))?;
                Ok(())
            }
            other => Err(ResourceError::Failed(format!("unknown action '{other}'"))),
        }
    }

    fn verify(&self, step: &PlanStep) -> Result<bool, ResourceError> {
        match step.action.as_str() {
            // Read-only actions verify trivially.
            "read" | "screenshot" => Ok(true),
            // After navigation, the current URL should match the requested one.
            "navigate" => {
                let want = param(step, "url");
                Ok(self.backend.current_url() == want)
            }
            // For fill/submit we trust the backend's success (a real backend can
            // check the field value / resulting page); default to true here.
            _ => Ok(true),
        }
    }

    fn restore(&mut self, snapshot: Self::Snap) -> Result<(), ResourceError> {
        match snapshot {
            BrowserSnapshot::Filled { selector } => {
                self.backend.clear(&selector)?;
                Ok(())
            }
            BrowserSnapshot::None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cmd_types::{Id, StepStatus};
    use std::collections::BTreeMap;

    /// A fake browser: records navigation and field state in memory.
    #[derive(Default)]
    struct FakeBrowser {
        url: String,
        fields: BTreeMap<String, String>,
        submitted: Vec<String>,
    }
    impl BrowserBackend for FakeBrowser {
        fn navigate(&mut self, url: &str) -> Result<String, ResourceError> {
            self.url = url.to_string();
            Ok(format!("Page: {url}"))
        }
        fn read(&self, _selector: &str) -> Result<String, ResourceError> {
            Ok("page text".into())
        }
        fn fill(&mut self, selector: &str, value: &str) -> Result<(), ResourceError> {
            self.fields.insert(selector.to_string(), value.to_string());
            Ok(())
        }
        fn clear(&mut self, selector: &str) -> Result<(), ResourceError> {
            self.fields.remove(selector);
            Ok(())
        }
        fn submit(&mut self, selector: &str) -> Result<(), ResourceError> {
            self.submitted.push(selector.to_string());
            Ok(())
        }
        fn current_url(&self) -> String {
            self.url.clone()
        }
    }

    fn step(action: &str, params: &[(&str, &str)]) -> PlanStep {
        let mut p = BTreeMap::new();
        for (k, v) in params {
            p.insert((*k).to_string(), serde_json::json!(v));
        }
        PlanStep {
            id: Id::new(),
            description: action.into(),
            capability: "browser".into(),
            action: action.into(),
            parameters: p,
            depends_on: vec![],
            requires_permission: false,
            status: StepStatus::Pending,
            error: None,
        }
    }

    #[test]
    fn risk_classification_matches_web_semantics() {
        assert_eq!(risk_of("navigate"), RiskClass::R0ReadOnly);
        assert_eq!(risk_of("read"), RiskClass::R0ReadOnly);
        assert_eq!(risk_of("fill"), RiskClass::R1Reversible);
        // The load-bearing one: buying / submitting is irreversible → human-gated.
        assert_eq!(risk_of("submit"), RiskClass::R3Irreversible);
        assert_eq!(risk_of("click_buy"), RiskClass::R3Irreversible);
        assert_eq!(risk_of("pay"), RiskClass::R3Irreversible);
        assert!(!risk_of("submit").may_be_autonomous());
    }

    #[test]
    fn navigate_executes_and_verifies_by_url() {
        let mut b = Browser::new(FakeBrowser::default());
        let s = step("navigate", &[("url", "https://example.com")]);
        assert!(b.snapshot(&s).unwrap().is_none()); // R0: no snapshot
        b.execute(&s).unwrap();
        assert!(b.verify(&s).unwrap());
        assert_eq!(b.backend().current_url(), "https://example.com");
    }

    #[test]
    fn fill_is_reversible_by_clearing() {
        let mut b = Browser::new(FakeBrowser::default());
        let s = step("fill", &[("selector", "#email"), ("value", "me@x.com")]);

        let snap = b.snapshot(&s).unwrap().unwrap();
        b.execute(&s).unwrap();
        assert_eq!(b.backend().fields.get("#email").unwrap(), "me@x.com");

        // Undo clears the field.
        b.restore(snap).unwrap();
        assert!(b.backend().fields.get("#email").is_none());
    }

    #[test]
    fn read_is_read_only() {
        let b = Browser::new(FakeBrowser::default());
        let s = step("read", &[("selector", "h1")]);
        assert!(b.snapshot(&s).unwrap().is_none());
        assert!(b.verify(&s).unwrap());
    }

    #[test]
    fn submit_executes_but_is_r3() {
        // The capability performs the submit when told to; but risk_of marks it
        // R3, so upstream policy requires human approval before it ever runs.
        let mut b = Browser::new(FakeBrowser::default());
        let s = step("submit", &[("selector", "#checkout")]);
        assert_eq!(risk_of("submit"), RiskClass::R3Irreversible);
        b.execute(&s).unwrap();
        assert_eq!(b.backend().submitted, vec!["#checkout".to_string()]);
    }

    #[test]
    fn unknown_action_fails() {
        let mut b = Browser::new(FakeBrowser::default());
        let s = step("teleport", &[]);
        assert!(b.execute(&s).is_err());
    }

    #[test]
    fn end_to_end_through_the_engine_fill_then_undo() {
        use cmd_ledger::Ledger;
        use cmd_transaction::TransactionEngine;

        let mut b = Browser::new(FakeBrowser::default());
        let s = step("fill", &[("selector", "#name"), ("value", "Nova")]);

        let snap = b.snapshot(&s).unwrap().unwrap();
        let mut ledger = Ledger::new();
        let out = {
            let mut engine = TransactionEngine::new(&mut ledger);
            engine.run(&mut b, &s, RiskClass::R1Reversible).unwrap()
        };
        assert_eq!(b.backend().fields.get("#name").unwrap(), "Nova");
        assert!(ledger.verify().is_ok());

        // Undo via the engine clears the field.
        let mut engine = TransactionEngine::new(&mut ledger);
        engine.rollback(&mut b, out.transaction, snap, &s).unwrap();
        assert!(b.backend().fields.get("#name").is_none());
    }
}
