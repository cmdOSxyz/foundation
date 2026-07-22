//! Model-backed planning: turn real natural language into an [`ExecutionPlan`]
//! by asking Claude, exactly the role the prototype's `anthropic-planner` plays.
//!
//! Design for testability: the crate splits into two halves.
//! - **`parse_plan_response`** turns Claude's JSON reply into a plan. This is
//!   pure and fully unit-tested — no network needed.
//! - **`ClaudePlanner::plan_via_api`** does the one impure thing: the HTTP call.
//!   It cannot run in CI (needs an API key and network), so it is exercised on a
//!   developer machine. It is kept as small as possible around the tested parser.
//!
//! `ClaudePlanner` implements the same [`Planner`](crate::Planner) trait as the
//! rule-based planner, so it drops into the agent unchanged.

use crate::Planner;
use cmd_types::{now, ExecutionPlan, Id, Intent, PlanStatus, PlanStep, StepStatus};
use serde::Deserialize;
use std::collections::BTreeMap;

/// Claude's expected JSON reply shape (a trimmed version of the prototype's
/// pass-2 schema: a friendly reply plus an optional plan).
#[derive(Debug, Deserialize)]
struct ClaudeReply {
    #[serde(default)]
    reply: String,
    #[serde(default)]
    plan: Option<ClaudePlan>,
}

#[derive(Debug, Deserialize)]
struct ClaudePlan {
    #[serde(default)]
    summary: String,
    #[serde(default)]
    steps: Vec<ClaudeStep>,
}

#[derive(Debug, Deserialize)]
struct ClaudeStep {
    #[serde(default)]
    description: String,
    #[serde(default)]
    capability: String,
    #[serde(default)]
    action: String,
    #[serde(default)]
    parameters: BTreeMap<String, serde_json::Value>,
}

/// Errors the model planner can produce.
#[derive(Debug)]
pub enum PlanError {
    /// The model reply was not valid JSON in the expected shape.
    BadResponse(String),
    /// The HTTP call failed.
    Transport(String),
}

impl std::fmt::Display for PlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanError::BadResponse(m) => write!(f, "bad model response: {m}"),
            PlanError::Transport(m) => write!(f, "transport error: {m}"),
        }
    }
}

impl std::error::Error for PlanError {}

/// The system prompt: instruct Claude to return a plan in the exact JSON shape
/// this crate parses. Kept close to the prototype's pass-2 contract.
const SYSTEM_PROMPT: &str = r#"You are Alios, the planning agent inside cmdOS. Turn the user's request into a filesystem plan.
Respond with ONLY valid JSON, no markdown fences:
{
  "reply": "<short friendly reply in the user's language>",
  "plan": null OR {
    "summary": "<one sentence>",
    "steps": [
      { "description": "", "capability": "filesystem",
        "action": "list"|"read"|"rename"|"move"|"delete",
        "parameters": { } }
    ]
  }
}
Rules: only use the filesystem capability. For anything you cannot resolve to a
concrete safe action, return plan: null. Never invent file paths."#;

/// Parse Claude's raw text reply into an [`ExecutionPlan`] for `intent`.
///
/// Tolerates ```json fences. If the reply carries no plan, returns a safe,
/// read-only fallback plan (a `list` of ".") — the agent never invents a
/// destructive action from an unparseable reply.
pub fn parse_plan_response(raw: &str, intent: &Intent) -> Result<ExecutionPlan, PlanError> {
    let cleaned = raw.replace("```json", "").replace("```", "");
    let cleaned = cleaned.trim();

    let reply: ClaudeReply =
        serde_json::from_str(cleaned).map_err(|e| PlanError::BadResponse(e.to_string()))?;

    let plan_summary = reply.plan.as_ref().map(|p| p.summary.clone());

    let steps = match reply.plan {
        Some(plan) if !plan.steps.is_empty() => plan
            .steps
            .into_iter()
            .map(|s| PlanStep {
                id: Id::new(),
                description: s.description,
                capability: if s.capability.is_empty() {
                    "filesystem".into()
                } else {
                    s.capability
                },
                action: s.action,
                parameters: s.parameters,
                depends_on: vec![],
                requires_permission: false,
                status: StepStatus::Pending,
                error: None,
            })
            .collect(),
        _ => vec![fallback_step()],
    };

    // Prefer the model's friendly reply; fall back to its plan summary; then to
    // a generic line.
    let summary = if !reply.reply.is_empty() {
        reply.reply
    } else if let Some(s) = plan_summary.filter(|s| !s.is_empty()) {
        s
    } else {
        format!("Plan for: {}", intent.raw_text)
    };

    Ok(ExecutionPlan {
        id: Id::new(),
        intent_id: intent.id,
        created_at: now(),
        status: PlanStatus::Draft,
        summary,
        steps,
    })
}

/// The safe fallback: a read-only list of the current folder.
fn fallback_step() -> PlanStep {
    let mut parameters = BTreeMap::new();
    parameters.insert("path".into(), serde_json::Value::String(".".into()));
    PlanStep {
        id: Id::new(),
        description: "Inspect the current folder (fallback)".into(),
        capability: "filesystem".into(),
        action: "list".into(),
        parameters,
        depends_on: vec![],
        requires_permission: false,
        status: StepStatus::Pending,
        error: None,
    }
}

/// How a request reaches the model. The one impure operation — the network call
/// — lives behind this trait, so the planner logic is fully testable with a fake
/// transport, and a real HTTP transport (in the desktop app or a dedicated
/// service crate) plugs in without changing anything here.
pub trait ClaudeTransport {
    /// Send the system prompt + user text to the model and return its raw text
    /// reply (the concatenated text blocks). Errors are transport failures.
    fn send(&self, system: &str, user_text: &str) -> Result<String, PlanError>;
}

/// A planner that asks Claude through a [`ClaudeTransport`]. The transport is
/// injected, so this planner is testable with a fake and production-ready with a
/// real HTTP transport.
pub struct ClaudePlanner<T: ClaudeTransport> {
    transport: T,
}

impl<T: ClaudeTransport> ClaudePlanner<T> {
    /// Build a planner over the given transport.
    pub fn new(transport: T) -> Self {
        ClaudePlanner { transport }
    }

    /// Ask the model and parse the reply into a plan.
    pub fn plan_via_model(&self, intent: &Intent) -> Result<ExecutionPlan, PlanError> {
        let raw = self.transport.send(SYSTEM_PROMPT, &intent.raw_text)?;
        parse_plan_response(&raw, intent)
    }
}

/// `ClaudePlanner` satisfies the `Planner` trait. A transport/parse failure
/// degrades to the safe read-only fallback plan — the agent stays safe even when
/// the model is unreachable.
impl<T: ClaudeTransport> Planner for ClaudePlanner<T> {
    fn plan(&self, intent: &Intent) -> ExecutionPlan {
        match self.plan_via_model(intent) {
            Ok(plan) => plan,
            Err(_) => ExecutionPlan {
                id: Id::new(),
                intent_id: intent.id,
                created_at: now(),
                status: PlanStatus::Draft,
                summary: format!("Fallback plan for: {}", intent.raw_text),
                steps: vec![fallback_step()],
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cmd_types::IntentSource;

    fn intent() -> Intent {
        Intent::received("rename my file", IntentSource::UserCommand)
    }

    #[test]
    fn parses_a_well_formed_plan() {
        let raw = r#"{
            "reply": "Sure, renaming it now.",
            "plan": {
                "summary": "Rename report",
                "steps": [
                    { "description": "rename", "capability": "filesystem",
                      "action": "rename",
                      "parameters": { "from": "a.txt", "to": "b.txt" } }
                ]
            }
        }"#;
        let plan = parse_plan_response(raw, &intent()).unwrap();
        assert_eq!(plan.steps.len(), 1);
        assert_eq!(plan.steps[0].action, "rename");
        assert_eq!(plan.summary, "Sure, renaming it now.");
    }

    #[test]
    fn tolerates_json_fences() {
        let raw = "```json\n{ \"reply\": \"hi\", \"plan\": null }\n```";
        let plan = parse_plan_response(raw, &intent()).unwrap();
        // plan: null → safe fallback list step.
        assert_eq!(plan.steps.len(), 1);
        assert_eq!(plan.steps[0].action, "list");
    }

    #[test]
    fn null_plan_falls_back_to_safe_read_only() {
        let raw = r#"{ "reply": "I'm not sure what to do.", "plan": null }"#;
        let plan = parse_plan_response(raw, &intent()).unwrap();
        assert_eq!(plan.steps[0].action, "list");
        assert_eq!(plan.steps[0].capability, "filesystem");
    }

    #[test]
    fn empty_steps_falls_back_to_safe_read_only() {
        let raw = r#"{ "reply": "ok", "plan": { "summary": "", "steps": [] } }"#;
        let plan = parse_plan_response(raw, &intent()).unwrap();
        assert_eq!(plan.steps.len(), 1);
        assert_eq!(plan.steps[0].action, "list");
    }

    #[test]
    fn garbage_response_is_an_error_not_a_guess() {
        let raw = "this is not json at all";
        assert!(parse_plan_response(raw, &intent()).is_err());
    }

    #[test]
    fn plan_links_back_to_the_intent() {
        let raw = r#"{ "reply": "ok", "plan": null }"#;
        let i = intent();
        let plan = parse_plan_response(raw, &i).unwrap();
        assert_eq!(plan.intent_id, i.id);
    }

    #[test]
    fn missing_capability_defaults_to_filesystem() {
        let raw = r#"{ "reply": "ok",
            "plan": { "summary": "s", "steps": [
                { "description": "d", "capability": "", "action": "list",
                  "parameters": {} } ] } }"#;
        let plan = parse_plan_response(raw, &intent()).unwrap();
        assert_eq!(plan.steps[0].capability, "filesystem");
    }

    // A fake transport lets us test the whole ClaudePlanner path — prompt in,
    // canned model reply out, parsed to a plan — with no network.
    struct FakeTransport {
        reply: String,
    }
    impl ClaudeTransport for FakeTransport {
        fn send(&self, _system: &str, _user: &str) -> Result<String, PlanError> {
            Ok(self.reply.clone())
        }
    }

    struct FailingTransport;
    impl ClaudeTransport for FailingTransport {
        fn send(&self, _system: &str, _user: &str) -> Result<String, PlanError> {
            Err(PlanError::Transport("network down".into()))
        }
    }

    #[test]
    fn claude_planner_parses_model_reply_into_a_plan() {
        let t = FakeTransport {
            reply: r#"{ "reply": "renaming", "plan": { "summary": "s",
                "steps": [ { "description": "r", "capability": "filesystem",
                "action": "rename", "parameters": {"from":"a","to":"b"} } ] } }"#
                .into(),
        };
        let planner = ClaudePlanner::new(t);
        let plan = planner.plan_via_model(&intent()).unwrap();
        assert_eq!(plan.steps[0].action, "rename");
    }

    #[test]
    fn claude_planner_degrades_to_safe_fallback_when_transport_fails() {
        use crate::Planner;
        let planner = ClaudePlanner::new(FailingTransport);
        // The Planner trait impl must never fail — it falls back safely.
        let plan = planner.plan(&intent());
        assert_eq!(plan.steps.len(), 1);
        assert_eq!(plan.steps[0].action, "list");
    }
}
