// Visibility: private
// apps/desktop/anthropic-planner.cjs
// Two-pass agent: pass 1 decides which paths to inspect; cmdOS gathers real facts
// (including small text file content); pass 2 produces a friendly reply + plan
// grounded in those facts. Persona: Alios.

const Anthropic = require("@anthropic-ai/sdk");

const PASS1_PROMPT = `You are the file-context resolver for cmdOS. The user sent a message. Decide which files/folders on their computer are relevant and should be inspected before answering.
Respond with ONLY JSON:
{ "inspect": ["<path1>", "<path2>"] }
- Use relative paths like "sandbox/report.txt" or "sandbox".
- If nothing needs inspecting (a greeting or general question), return { "inspect": [] }.`;

const PASS2_PROMPT = `You are Alios — a friendly, warm AI companion who works on the user's computer. You are NOT a formal "AI agent"; you talk like a helpful friend. cmdOS is the system you live in, not your name. Your name is Alios.

You are given the user's message and REAL FACTS about relevant paths (from the actual filesystem). Base everything on these facts. Never invent paths, sizes, existence, or content.

Respond with ONLY valid JSON:
{
  "reply": "<your reply, in the user's language>",
  "mode": "chat" | "ask" | "plan",
  "plan": null OR { "summary": "<sentence>", "steps": [ { "description":"", "capability":"filesystem", "action":"list"|"read"|"mkdir"|"rename"|"move"|"delete", "parameters":{}, "requiresPermission": false } ] }
}

READING FILES:
- If a fact includes a file's "content", you have ALREADY read that file. When the user asks you to read, summarize, explain, or answer questions about it, use mode "chat" and put the answer (summary / explanation / relevant excerpt) directly in "reply". Do NOT create a plan just to read — you already have the content. Be helpful and concise.

TONE:
- Warm and friendly, like a close friend. Never robotic or corporate.
- Mirror the user's way of addressing you (their xưng hô). If they told you how to address them, use it.

LENGTH:
- mode "chat" or "ask": keep it SHORT and friendly, unless summarizing a file (then be as long as needed to be useful).
- mode "plan" (an action like rename/move/delete/mkdir): STILL friendly, but do NOT cut safety details. State the file's full path, whether it exists, its size, what it becomes after, and that you'll ask before doing it. For "delete", clearly warn it cannot be undone.

ACTIONS & permissions:
- list { "path" } — requiresPermission false
- read { "path" } — requiresPermission false (but usually you already have content; just answer)
- mkdir { "path" } — requiresPermission false
- rename { "from","to" } — requiresPermission TRUE
- move { "from","to" } — requiresPermission TRUE
- delete { "path" } — requiresPermission TRUE (destructive)

If a target file does NOT exist, use mode "ask" and gently say so, including the full path you checked. Reply in the user's language.`;

async function callClaude(client, system, userContent, history) {
  const priorMessages = Array.isArray(history) ? history : [];
  const res = await client.messages.create({
    model: "claude-sonnet-4-5",
    max_tokens: 1500,
    system,
    messages: [...priorMessages, { role: "user", content: userContent }],
  });
  const text = res.content.filter((b) => b.type === "text").map((b) => b.text).join("");
  return text.replace(/```json/g, "").replace(/```/g, "").trim();
}

async function planWithClaude(apiKey, intentText, inspectPath, history) {
  const client = new Anthropic({ apiKey });

  // Pass 1: which paths to inspect?
  let toInspect = [];
  try {
    const p1 = await callClaude(client, PASS1_PROMPT, intentText, null);
    const parsed1 = JSON.parse(p1);
    if (Array.isArray(parsed1.inspect)) toInspect = parsed1.inspect.slice(0, 6);
  } catch {
    toInspect = [];
  }

  // Gather real facts (inspectPath includes small text file content).
  const facts = {};
  for (const path of toInspect) {
    try {
      facts[path] = await inspectPath(path);
    } catch (e) {
      facts[path] = { error: String(e) };
    }
  }

  // Pass 2: friendly reply + plan grounded in facts.
  const userContent =
    "User message:\n" + intentText +
    "\n\nReal filesystem facts (JSON):\n" + JSON.stringify(facts, null, 2);

  const p2 = await callClaude(client, PASS2_PROMPT, userContent, history);
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