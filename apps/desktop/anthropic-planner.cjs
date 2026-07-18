// Visibility: private
// apps/desktop/anthropic-planner.cjs
// Two-pass agent: pass 1 decides which paths to inspect; cmdOS gathers real facts;
// pass 2 produces a detailed reply + plan grounded in those facts (no guessing).

const Anthropic = require("@anthropic-ai/sdk");

const PASS1_PROMPT = `You are cmdOS. The user sent a message. Decide if you need to inspect any files/folders on their computer before answering.
Respond with ONLY JSON:
{ "inspect": ["<path1>", "<path2>"] }
- List the paths (relative like "sandbox/report.txt" or "sandbox") that are relevant to the request.
- If nothing needs inspecting (a greeting, a general question), return { "inspect": [] }.`;

const PASS2_PROMPT = `You are cmdOS — an AI agent working on the user's computer like a capable employee.
You are given the user's message and REAL FACTS about relevant paths (from the actual filesystem).
Base everything you say on these facts. Never invent paths, sizes, or existence.

Respond with ONLY valid JSON:
{
  "reply": "<detailed, natural reply in the user's language>",
  "mode": "chat" | "ask" | "plan",
  "plan": null OR { "summary": "<sentence>", "steps": [ { "description":"", "capability":"filesystem", "action":"list"|"rename", "parameters":{}, "requiresPermission": false } ] }
}

When mode is "plan", the "reply" must be specific and grounded in the facts:
- State the file's full path, whether it exists, its size and last-modified time.
- Say clearly what you will change and what it becomes after.
- Note which steps need approval and why.
- Invite the user to approve or adjust.
If a target file does NOT exist, use mode "ask" and tell them it wasn't found (with the full path you checked).

Rules: filesystem.list { "path" }, filesystem.rename { "from","to" }. rename => requiresPermission true. Reply in the user's language.`;

async function callClaude(client, system, userContent) {
  const res = await client.messages.create({
    model: "claude-sonnet-4-5",
    max_tokens: 1024,
    system,
    messages: [{ role: "user", content: userContent }],
  });
  const text = res.content.filter((b) => b.type === "text").map((b) => b.text).join("");
  return text.replace(/```json/g, "").replace(/```/g, "").trim();
}

async function planWithClaude(apiKey, intentText, inspectPath) {
  const client = new Anthropic({ apiKey });

  // Pass 1: which paths to inspect?
  let toInspect = [];
  try {
    const p1 = await callClaude(client, PASS1_PROMPT, intentText);
    const parsed1 = JSON.parse(p1);
    if (Array.isArray(parsed1.inspect)) toInspect = parsed1.inspect.slice(0, 6);
  } catch {
    toInspect = [];
  }

  // Gather real facts.
  const facts = {};
  for (const path of toInspect) {
    try {
      facts[path] = await inspectPath(path);
    } catch (e) {
      facts[path] = { error: String(e) };
    }
  }

  // Pass 2: detailed reply + plan grounded in facts.
  const userContent =
    "User message:\n" + intentText +
    "\n\nReal filesystem facts (JSON):\n" + JSON.stringify(facts, null, 2);

  const p2 = await callClaude(client, PASS2_PROMPT, userContent);
  let parsed;
  try {
    parsed = JSON.parse(p2);
  } catch {
    return { reply: p2 || "…", mode: "chat", plan: null };
  }
  if (!parsed.reply) parsed.reply = "";
  if (!parsed.mode) parsed.mode = parsed.plan ? "plan" : "chat";
  return parsed;
}

module.exports = { planWithClaude };