//! # claude-http — a real [`ClaudeTransport`]
//!
//! Implements the `ClaudeTransport` trait from `alios` with a real HTTPS call to
//! the Anthropic Messages API. This is the one piece that needs an API key and
//! network, so it is isolated in its own crate: if its HTTP dependency ever
//! causes trouble, the agent core stays unaffected.
//!
//! The planner logic that consumes this (parsing, safe fallback) is already
//! fully tested in `alios` with a fake transport. This crate only adds the wire
//! call, kept as small as possible.
//!
//! Usage:
//! ```no_run
//! use alios::{Agent, ClaudePlanner};
//! use claude_http::HttpTransport;
//!
//! let key = std::env::var("ANTHROPIC_API_KEY").unwrap();
//! let planner = ClaudePlanner::new(HttpTransport::new(key));
//! let agent = Agent::new("Nova", planner);
//! let (_intent, plan) = agent.plan_for("clean up my downloads folder");
//! ```

use alios::{ClaudeTransport, PlanError};

/// A [`ClaudeTransport`] backed by a blocking reqwest client.
pub struct HttpTransport {
    api_key: String,
    model: String,
    max_tokens: u32,
}

impl HttpTransport {
    /// Build a transport with an API key. Model defaults to a current Sonnet.
    pub fn new(api_key: impl Into<String>) -> Self {
        HttpTransport {
            api_key: api_key.into(),
            model: "claude-sonnet-4-5".into(),
            max_tokens: 1500,
        }
    }

    /// Override the model string (e.g. a Haiku for cheaper planning).
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }
}

impl ClaudeTransport for HttpTransport {
    fn send(&self, system: &str, user_text: &str) -> Result<String, PlanError> {
        let body = serde_json::json!({
            "model": self.model,
            "max_tokens": self.max_tokens,
            "system": system,
            "messages": [ { "role": "user", "content": user_text } ],
        });

        let client = reqwest::blocking::Client::new();
        let resp = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| PlanError::Transport(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().unwrap_or_default();
            return Err(PlanError::Transport(format!("API {status}: {text}")));
        }

        let json: serde_json::Value = resp
            .json()
            .map_err(|e| PlanError::Transport(e.to_string()))?;

        // Concatenate all text blocks from the response content array.
        let text = json
            .get("content")
            .and_then(|c| c.as_array())
            .map(|blocks| {
                blocks
                    .iter()
                    .filter_map(|b| b.get("text").and_then(|t| t.as_str()))
                    .collect::<Vec<_>>()
                    .join("")
            })
            .unwrap_or_default();

        if text.is_empty() {
            return Err(PlanError::BadResponse("empty model response".into()));
        }
        Ok(text)
    }
}
