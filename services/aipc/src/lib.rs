//! # aipc — Agent IPC: the capability tool surface
//!
//! AIPC is how a user agent reaches capabilities. Every capability publishes its
//! actions as **MCP-style tools**; the agent never calls a capability directly.
//! Instead it asks AIPC to invoke a tool, and **every call is routed through the
//! policy gate** (Alios) before it reaches the capability. This is the OS ABI for
//! tools: MCP is the shape, the permission gate is the enforcement.
//!
//! This crate is the in-process spine of AIPC — the registry and the router.
//! The transport that speaks the wire MCP protocol to *external* MCP servers
//! (stdio / JSON-RPC) plugs into this same interface later, without changing the
//! router or the gate.
//!
//! Defined by RFC-0012.

use cmd_policy::{Decision, PolicyEngine, ProposedAction};
use cmd_types::{Budget, Mandate, RiskClass};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A tool a capability exposes — the MCP-style descriptor an agent sees.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tool {
    /// Capability this tool belongs to, e.g. `"filesystem"`.
    pub capability: String,
    /// Action name, e.g. `"rename"`. `capability::action` is the tool's full name.
    pub name: String,
    pub description: String,
    /// Parameter names the tool accepts.
    pub parameters: Vec<String>,
    /// Risk class of invoking this tool (drives the policy decision).
    pub risk: RiskClass,
}

impl Tool {
    /// The fully-qualified tool name, `capability.action`.
    pub fn qualified_name(&self) -> String {
        format!("{}.{}", self.capability, self.name)
    }
}

/// A request to invoke a tool, as an agent issues it.
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub capability: String,
    pub action: String,
    pub arguments: BTreeMap<String, serde_json::Value>,
    /// Money the call would spend (0 if none).
    pub spend: u64,
}

/// The result of routing a tool call.
#[derive(Debug, Clone, PartialEq)]
pub enum RouteResult {
    /// Policy allowed the call; it should be executed by the runtime. Carries the
    /// resolved tool so the caller knows exactly what was authorized.
    Authorized { tool: Tool },
    /// Policy requires human approval before this call may run.
    NeedsApproval { reason: String },
    /// Policy blocked the call.
    Blocked { reason: String },
    /// No such tool is registered.
    UnknownTool { requested: String },
}

/// The AIPC registry + router. Capabilities register their tools; agents route
/// calls through it; every call passes the policy gate.
#[derive(Default)]
pub struct Aipc {
    tools: BTreeMap<String, Tool>,
}

impl Aipc {
    /// A new, empty registry.
    pub fn new() -> Self {
        Aipc {
            tools: BTreeMap::new(),
        }
    }

    /// Register a tool. Keyed by qualified name; re-registering replaces.
    pub fn register(&mut self, tool: Tool) {
        self.tools.insert(tool.qualified_name(), tool);
    }

    /// All registered tools, sorted by qualified name — what an agent may call.
    pub fn list_tools(&self) -> Vec<Tool> {
        self.tools.values().cloned().collect()
    }

    /// Look up a tool by capability + action.
    pub fn tool(&self, capability: &str, action: &str) -> Option<&Tool> {
        self.tools.get(&format!("{capability}.{action}"))
    }

    /// Route a tool call through the policy gate. Returns what the runtime should
    /// do: execute (Authorized), ask the human (NeedsApproval), refuse (Blocked),
    /// or report an unknown tool.
    ///
    /// AIPC never executes the tool itself — it authorizes. Execution goes
    /// through the kernel + transaction engine, so it stays reversible and
    /// recorded.
    pub fn route(
        &self,
        call: &ToolCall,
        mandate: Option<&Mandate>,
        budget: Option<&Budget>,
    ) -> RouteResult {
        let tool = match self.tool(&call.capability, &call.action) {
            Some(t) => t.clone(),
            None => {
                return RouteResult::UnknownTool {
                    requested: format!("{}.{}", call.capability, call.action),
                }
            }
        };

        let action = ProposedAction {
            capability: call.capability.clone(),
            risk: tool.risk,
            spend: call.spend,
        };

        match PolicyEngine::now().evaluate(&action, mandate, budget) {
            Decision::Allow { .. } => RouteResult::Authorized { tool },
            Decision::NeedsApproval { reason } => RouteResult::NeedsApproval { reason },
            Decision::Block { reason } => RouteResult::Blocked { reason },
        }
    }
}

/// Tool declarations for the first-party capabilities (filesystem, browser).
///
/// These describe each capability's actions as MCP-style tools — name,
/// parameters, and risk — so the agent sees a populated registry. Risk classes
/// here mirror the capabilities' own `risk_of` semantics (e.g. cap-browser's
/// submit/buy = R3). Declaring the tools here keeps capabilities free of any
/// dependency on AIPC (no cycle); the catalog is the single place that binds
/// capability actions into the tool surface.
pub mod catalog {
    use super::{RiskClass, Tool};

    fn t(cap: &str, name: &str, desc: &str, params: &[&str], risk: RiskClass) -> Tool {
        Tool {
            capability: cap.into(),
            name: name.into(),
            description: desc.into(),
            parameters: params.iter().map(|p| p.to_string()).collect(),
            risk,
        }
    }

    /// Tools exposed by the filesystem capability (cap-files).
    pub fn filesystem_tools() -> Vec<Tool> {
        vec![
            t(
                "filesystem",
                "list",
                "List entries in a folder",
                &["path"],
                RiskClass::R0ReadOnly,
            ),
            t(
                "filesystem",
                "read",
                "Read a file's contents",
                &["path"],
                RiskClass::R0ReadOnly,
            ),
            t(
                "filesystem",
                "rename",
                "Rename a file",
                &["from", "to"],
                RiskClass::R1Reversible,
            ),
            t(
                "filesystem",
                "move",
                "Move a file",
                &["from", "to"],
                RiskClass::R1Reversible,
            ),
            t(
                "filesystem",
                "delete",
                "Delete a file",
                &["path"],
                RiskClass::R1Reversible,
            ),
        ]
    }

    /// Tools exposed by the browser capability (cap-browser). Mirrors its risk
    /// model: read-only browsing is R0, filling is R1, and submitting / buying /
    /// paying is R3 (human-gated).
    pub fn browser_tools() -> Vec<Tool> {
        vec![
            t(
                "browser",
                "navigate",
                "Go to a URL",
                &["url"],
                RiskClass::R0ReadOnly,
            ),
            t(
                "browser",
                "read",
                "Read page content",
                &["selector"],
                RiskClass::R0ReadOnly,
            ),
            t(
                "browser",
                "screenshot",
                "Capture the page",
                &[],
                RiskClass::R0ReadOnly,
            ),
            t(
                "browser",
                "fill",
                "Fill a form field",
                &["selector", "value"],
                RiskClass::R1Reversible,
            ),
            t(
                "browser",
                "submit",
                "Submit a form",
                &["selector"],
                RiskClass::R3Irreversible,
            ),
            t(
                "browser",
                "click_buy",
                "Complete a purchase",
                &["selector"],
                RiskClass::R3Irreversible,
            ),
            t(
                "browser",
                "pay",
                "Authorize a payment",
                &["selector"],
                RiskClass::R3Irreversible,
            ),
        ]
    }

    /// All first-party tools.
    pub fn all() -> Vec<Tool> {
        let mut v = filesystem_tools();
        v.extend(browser_tools());
        v
    }
}

impl Aipc {
    /// Register all first-party capability tools (filesystem + browser). This is
    /// how the agent gets a populated tool surface at startup.
    pub fn with_first_party(mut self) -> Self {
        for tool in catalog::all() {
            self.register(tool);
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cmd_types::{now, Id, RiskClass};

    fn tool(cap: &str, action: &str, risk: RiskClass) -> Tool {
        Tool {
            capability: cap.into(),
            name: action.into(),
            description: format!("{cap} {action}"),
            parameters: vec![],
            risk,
        }
    }

    fn call(cap: &str, action: &str, spend: u64) -> ToolCall {
        ToolCall {
            capability: cap.into(),
            action: action.into(),
            arguments: BTreeMap::new(),
            spend,
        }
    }

    fn mandate(caps: &[&str], max: RiskClass) -> Mandate {
        Mandate {
            id: Id::new(),
            agent_id: Id::new(),
            scope: "test".into(),
            capabilities: caps.iter().map(|c| c.to_string()).collect(),
            max_autonomous_risk: max,
            budget_id: None,
            granted_at: now(),
            expires_at: None,
            revoked_at: None,
        }
    }

    #[test]
    fn registers_and_lists_tools() {
        let mut aipc = Aipc::new();
        aipc.register(tool("filesystem", "list", RiskClass::R0ReadOnly));
        aipc.register(tool("filesystem", "rename", RiskClass::R1Reversible));
        let tools = aipc.list_tools();
        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0].qualified_name(), "filesystem.list");
    }

    #[test]
    fn routes_read_only_as_authorized_without_authority() {
        let mut aipc = Aipc::new();
        aipc.register(tool("filesystem", "list", RiskClass::R0ReadOnly));
        let r = aipc.route(&call("filesystem", "list", 0), None, None);
        assert!(matches!(r, RouteResult::Authorized { .. }));
    }

    #[test]
    fn routes_unknown_tool() {
        let aipc = Aipc::new();
        let r = aipc.route(&call("filesystem", "nope", 0), None, None);
        assert!(matches!(r, RouteResult::UnknownTool { .. }));
    }

    #[test]
    fn routes_r1_within_mandate_as_authorized() {
        let mut aipc = Aipc::new();
        aipc.register(tool("filesystem", "rename", RiskClass::R1Reversible));
        let m = mandate(&["filesystem"], RiskClass::R1Reversible);
        let r = aipc.route(&call("filesystem", "rename", 0), Some(&m), None);
        assert!(matches!(r, RouteResult::Authorized { .. }));
    }

    #[test]
    fn blocks_call_outside_mandate() {
        let mut aipc = Aipc::new();
        aipc.register(tool("filesystem", "rename", RiskClass::R1Reversible));
        let m = mandate(&["browser"], RiskClass::R1Reversible);
        let r = aipc.route(&call("filesystem", "rename", 0), Some(&m), None);
        assert!(matches!(r, RouteResult::Blocked { .. }));
    }

    #[test]
    fn r3_tool_needs_approval() {
        let mut aipc = Aipc::new();
        aipc.register(tool("cmdpay", "pay", RiskClass::R3Irreversible));
        let m = mandate(&["cmdpay"], RiskClass::R3Irreversible);
        let r = aipc.route(&call("cmdpay", "pay", 0), Some(&m), None);
        assert!(matches!(r, RouteResult::NeedsApproval { .. }));
    }

    #[test]
    fn every_call_passes_the_gate_even_when_tool_exists() {
        // A registered spend tool with no budget must still be blocked — AIPC
        // does not bypass policy just because the tool is known.
        let mut aipc = Aipc::new();
        aipc.register(tool("cmdpay", "buy", RiskClass::R1Reversible));
        let m = mandate(&["cmdpay"], RiskClass::R1Reversible);
        let r = aipc.route(&call("cmdpay", "buy", 50), Some(&m), None);
        assert!(matches!(r, RouteResult::Blocked { .. }));
    }

    // ---- First-party capability catalog ------------------------------------

    #[test]
    fn first_party_registry_has_filesystem_and_browser_tools() {
        let aipc = Aipc::new().with_first_party();
        let tools = aipc.list_tools();
        // Both capabilities present.
        assert!(tools.iter().any(|t| t.capability == "filesystem"));
        assert!(tools.iter().any(|t| t.capability == "browser"));
        // A known tool resolves.
        assert!(aipc.tool("filesystem", "rename").is_some());
        assert!(aipc.tool("browser", "navigate").is_some());
    }

    #[test]
    fn browser_buy_is_registered_as_r3() {
        let aipc = Aipc::new().with_first_party();
        let buy = aipc.tool("browser", "click_buy").expect("click_buy tool");
        assert_eq!(buy.risk, RiskClass::R3Irreversible);
    }

    #[test]
    fn routing_a_catalog_tool_respects_risk() {
        // Navigating (R0) is authorized without a mandate; buying (R3) needs
        // approval — proving the catalog's risk flows through the policy gate.
        let aipc = Aipc::new().with_first_party();

        let nav = aipc.route(&call("browser", "navigate", 0), None, None);
        assert!(matches!(nav, RouteResult::Authorized { .. }));

        let m = mandate(&["browser"], RiskClass::R3Irreversible);
        let buy = aipc.route(&call("browser", "click_buy", 0), Some(&m), None);
        assert!(matches!(buy, RouteResult::NeedsApproval { .. }));
    }
}
