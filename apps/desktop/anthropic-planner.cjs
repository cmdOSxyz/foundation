// Visibility: private
// apps/desktop/anthropic-planner.cjs
// Uses the user's Claude key to turn a natural-language intent into an ExecutionPlan.
// Claude only PROPOSES a plan (JSON). cmdOS validates, asks permission, and executes.

const Anthropic = require("@anthropic-ai/sdk");

const SYSTEM_PROMPT = `You are the planning engine of cmdOS, an AI execution operating system.
Convert the user's request into a JSON execution plan. Respond with ONLY valid JSON, no prose.

The plan format:
{
  "summary": "<one short sentence>",
  "steps": [
    { "description": "<what this step does>",
      "capability": "filesystem",
      "action": "list" | "rename",
      "parameters": { },
      "requiresPermission": false }
  ]
}

Supported actions ONLY:
- filesystem.list   params: { "path": "<folder>" }
- filesystem.rename params: { "from": "<path>", "to": "<new name>" }

Rules:
- Use ONLY the actions above. If the request needs something else, return a plan with an empty steps array and explain in summary.
- rename always has requiresPermission true. list always false.
- Keep plans minimal.`;

async function planWithClaude(apiKey, intentText) {
  const client = new Anthropic({ apiKey });

  const response = await client.messages.create({
    model: "claude-sonnet-4-5",
    max_tokens: 1024,
    system: SYSTEM_PROMPT,
    messages: [{ role: "user", content: intentText }],
  });

  const text = response.content
    .filter((block) => block.type === "text")
    .map((block) => block.text)
    .join("");

  const clean = text.replace(/```json/g, "").replace(/```/g, "").trim();

  let parsed;
  try {
    parsed = JSON.parse(clean);
  } catch (err) {
    throw new Error("Claude did not return valid JSON: " + text.slice(0, 200));
  }

  if (!parsed.steps || !Array.isArray(parsed.steps)) {
    throw new Error("Plan has no steps array");
  }
  return parsed;
}

module.exports = { planWithClaude };