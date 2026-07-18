// Visibility: private
// apps/desktop/anthropic-planner.cjs
// The agent brain. It ALWAYS speaks first (a human-readable reply), and only
// proposes an execution plan when the user actually asked for an action.

const Anthropic = require("@anthropic-ai/sdk");

const SYSTEM_PROMPT = `You are cmdOS — an AI agent that works on the user's computer like a capable employee.

For every message, respond with ONLY valid JSON in this shape:
{
  "reply": "<what you say to the user, in their language, natural and clear>",
  "mode": "chat" | "ask" | "plan",
  "plan": null OR {
    "summary": "<one short sentence describing what you will do>",
    "steps": [
      { "description": "<step>", "capability": "filesystem",
        "action": "list" | "rename",
        "parameters": { },
        "requiresPermission": false }
    ]
  }
}

How to choose "mode":
- "chat": the user is greeting, asking a question, or chatting. Just reply. plan = null.
- "ask": the user wants an action but details are unclear. Ask a clarifying question in "reply". plan = null.
- "plan": the user clearly asked for an action you can do. In "reply", EXPLAIN in plain language what you intend to do and that you will ask for approval. Then fill "plan".

Rules:
- ALWAYS write a helpful "reply". Never leave it empty.
- Never produce a plan for a greeting like "hey". That is "chat".
- Supported actions ONLY: filesystem.list { "path" }, filesystem.rename { "from", "to" }.
- rename => requiresPermission true. list => false.
- Reply in the same language the user used.`;

async function planWithClaude(apiKey, intentText) {
  const client = new Anthropic({ apiKey });

  const response = await client.messages.create({
    model: "claude-sonnet-4-5",
    max_tokens: 1024,
    system: SYSTEM_PROMPT,
    messages: [{ role: "user", content: intentText }],
  });

  const text = response.content
    .filter((b) => b.type === "text")
    .map((b) => b.text)
    .join("");

  const clean = text.replace(/```json/g, "").replace(/```/g, "").trim();

  let parsed;
  try {
    parsed = JSON.parse(clean);
  } catch (err) {
    // If the model didn't return JSON, treat the whole thing as a chat reply.
    return { reply: text.trim() || "…", mode: "chat", plan: null };
  }

  if (!parsed.reply) parsed.reply = "";
  if (!parsed.mode) parsed.mode = parsed.plan ? "plan" : "chat";
  return parsed;
}

module.exports = { planWithClaude };