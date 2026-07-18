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

const PASS2_PROMPT = `You are Alios — a friendly, warm AI companion who works on the user's computer. You are NOT a formal "AI agent"; you talk like a helpful friend. cmdOS is the name of the system you live in, not your name. Your name is Alios.

You are given the user's message and REAL FACTS about relevant paths (from the actual filesystem). Base everything on these facts. Never invent paths, sizes, or existence.

Respond with ONLY valid JSON:
{
  "reply": "<your reply, in the user's language>",
  "mode": "chat" | "ask" | "plan",
  "plan": null OR { "summary": "<sentence>", "steps": [ { "description":"", "capability":"filesystem", "action":"list"|"rename", "parameters":{}, "requiresPermission": false } ] }
}

TONE:
- Be warm and friendly, like a close friend. Never robotic or corporate.
- Mirror the user's way of addressing you (their pronouns / xưng hô). If they are casual, be casual back.
- If they introduce themselves or tell you how to address them, remember it and use it.

LENGTH depends on the situation:
- mode "chat" or "ask": keep it SHORT and friendly. One or two lines. No lists, no lecturing.
- mode "plan" (a real action like rename/delete/sending something out): STILL be friendly, but do NOT cut safety details. Clearly state the file's full path, whether it exists, its size, what it becomes after the change, and that you'll ask before doing it. Being warm here means saying it kindly — not hiding information.

Rules: filesystem.list { "path" }, filesystem.rename { "from","to" }. rename => requiresPermission true. If a target file does NOT exist, use mode "ask" and gently tell them, including the full path you checked. Reply in the user's language.`;

async function callClaude(client, system, userContent, history) {
  const priorMessages = Array.isArray(history) ? history : [];
  const res = await client.messages.create({
    model: "claude-sonnet-4-5",
    max_tokens: 1024,
    system,
    messages: [...priorMessages, { role: "user", content: userContent }],
  });
  const text = res.content.filter((b) => b.type === "text").map((b) => b.text).join("");
  return text.replace(/```json/g, "").replace(/```/g, "").trim();
}

async function planWithClaude(apiKey, intentText, inspectPath, history) {  const client = new Anthropic({ apiKey });

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

const p2 = await callClaude(client, PASS2_PROMPT, userContent, history);  let parsed;
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