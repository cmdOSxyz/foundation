// Visibility: private
// apps/desktop/renderer.js
// The cmdOS command UI. Talks to the agent (main process) via window.cmdos.

const workspace = document.getElementById("workspace");
const input = document.getElementById("cmdInput");

// Append an element to the workspace and scroll to the bottom.
function add(html, className) {
  const div = document.createElement("div");
  div.className = className || "";
  div.innerHTML = html;
  workspace.appendChild(div);
  workspace.scrollTop = workspace.scrollHeight;
  return div;
}

// Escape text before inserting into HTML.
function escapeHtml(s) {
  return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

// Show an approval box; resolve true (approve) or false (deny).
function askApproval(container, step) {
  return new Promise((resolve) => {
    const box = document.createElement("div");
    box.className = "approval";
    box.innerHTML =
      '<div class="label">CMDOS WANTS YOUR APPROVAL</div>' +
      '<div class="cmd">' + escapeHtml(step.capability) + "." + escapeHtml(step.action) + "</div>" +
      '<div style="color:var(--muted); font-size:12px; margin-bottom:10px;">' + escapeHtml(step.description) + "</div>" +
      '<button class="btn approve">Approve</button>' +
      '<button class="btn deny">Deny</button>';
    container.appendChild(box);
    workspace.scrollTop = workspace.scrollHeight;
    box.querySelector(".approve").onclick = () => { box.remove(); resolve(true); };
    box.querySelector(".deny").onclick = () => { box.remove(); resolve(false); };
  });
}

// Handle one user message end to end.
async function runIntent(text) {
  add('<div class="who">You</div><div>' + escapeHtml(text) + "</div>", "msg you");

  const cm = add('<div class="who">cmdOS</div>', "msg");
  cm.innerHTML += '<div class="comment">// thinking…</div>';

  const result = await window.cmdos.plan(text);
  if (!result.ok) {
    cm.innerHTML += '<div class="trace" style="color:#f87171;">&#10007; ' + escapeHtml(result.message) + "</div>";
    return;
  }

  const agent = result.plan; // { reply, mode, plan }

  // 1) Always show the agent's spoken reply.
  if (agent.reply) {
    cm.innerHTML += '<div style="margin:6px 0;">' + escapeHtml(agent.reply) + "</div>";
  }

  // 2) chat / ask => nothing to execute.
  if (agent.mode !== "plan" || !agent.plan || !agent.plan.steps || agent.plan.steps.length === 0) {
    workspace.scrollTop = workspace.scrollHeight;
    return;
  }

  // 3) plan => show the proposal, then run steps.
  const plan = agent.plan;
  cm.innerHTML += '<div class="comment">// proposal: ' + escapeHtml(plan.summary) +
    " (" + plan.steps.length + " steps)</div>";

  for (const step of plan.steps) {
    if (step.requiresPermission) {
      const approved = await askApproval(cm, step);
      if (!approved) {
        cm.innerHTML += '<div class="trace" style="color:#f87171;">&#10007; denied &middot; stopped safely</div>';
        return;
      }
    }
    const res = await window.cmdos.runStep({
      id: "s-" + Math.random().toString(36).slice(2, 6),
      description: step.description,
      capability: step.capability,
      action: step.action,
      parameters: step.parameters || {},
      dependsOn: [],
      requiresPermission: step.requiresPermission,
      status: "pending",
    });
    const mark = res.ok ? "&#10003;" : "&#10007;";
    const color = res.ok ? "" : ' style="color:#f87171;"';
    cm.innerHTML += '<div class="trace"' + color + ">" + mark + " " + escapeHtml(res.message) + "</div>";
    if (!res.ok) return;
  }
  cm.innerHTML += '<div class="trace">&#10003; completed &middot; audit trail written</div>';
}

// Enter runs the intent.
input.addEventListener("keydown", (e) => {
  if (e.key === "Enter" && input.value.trim()) {
    const text = input.value.trim();
    input.value = "";
    runIntent(text);
  }
});

// --- API key setup (BYOK) ---
async function setupKeyUI() {
  const status = await window.cmdos.hasKey("anthropic");
  if (status.hasKey) return;
  const bar = document.createElement("div");
  bar.className = "approval";
  bar.innerHTML =
    '<div class="label">CONNECT YOUR CLAUDE API KEY (stored locally)</div>' +
    '<input id="keyInput" type="password" placeholder="sk-ant-..." ' +
    'style="width:100%; padding:8px; margin-bottom:8px; background:var(--panel); ' +
    'border:1px solid var(--border); border-radius:6px; color:var(--text); font-family:inherit;" />' +
    '<button class="btn approve" id="saveKey">Save key</button>';
  workspace.prepend(bar);
  document.getElementById("saveKey").onclick = async () => {
    const key = document.getElementById("keyInput").value.trim();
    if (!key) return;
    await window.cmdos.setKey("anthropic", key);
    bar.innerHTML = '<div class="label" style="color:var(--accent);">✓ Claude key saved locally</div>';
  };
}
setupKeyUI();