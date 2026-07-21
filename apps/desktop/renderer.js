// Visibility: private
// apps/desktop/renderer.js
// The cmdOS command UI. Talks to the agent (main process) via window.cmdos.

const workspace = document.getElementById("workspace");
const input = document.getElementById("cmdInput");

// Conversation memory for this session (sent to the agent each turn).
const conversation = [];

// Inject a tiny CSS animation for the loading dots and cursor.
const style = document.createElement("style");
style.textContent =
  "@keyframes blink { 0%,100%{opacity:.2} 50%{opacity:1} }" +
  ".loading span { animation: blink 1.2s infinite; }" +
  ".loading span:nth-child(2){ animation-delay:.2s }" +
  ".loading span:nth-child(3){ animation-delay:.4s }" +
  ".typing::after { content:'▋'; animation: blink 1s infinite; color: var(--accent); }";
document.head.appendChild(style);

// Typewriter: reveal text gradually inside an element.
function typeText(el, text, speed) {
  return new Promise((resolve) => {
    el.classList.add("typing");
    let i = 0;
    const tick = () => {
      if (i <= text.length) {
        el.textContent = text.slice(0, i);
        i++;
        workspace.scrollTop = workspace.scrollHeight;
        setTimeout(tick, speed || 12);
      } else {
        el.classList.remove("typing");
        resolve();
      }
    };
    tick();
  });
}

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

// Show an approval box WITH a dry-run preview; resolve true/false.
async function askApproval(container, step) {
  // Get the preview first.
  let previewHtml = '<div style="color:var(--muted); font-size:12px;">Loading preview…</div>';
  const box = document.createElement("div");
  box.className = "approval";
  box.innerHTML =
    '<div class="label">CMDOS WANTS YOUR APPROVAL</div>' +
    '<div class="cmd">' + escapeHtml(step.capability) + "." + escapeHtml(step.action) + "</div>" +
    '<div id="previewArea" style="margin-bottom:10px;">' + previewHtml + '</div>' +
    '<button class="btn approve">Approve</button>' +
    '<button class="btn deny">Deny</button>';
  container.appendChild(box);
  workspace.scrollTop = workspace.scrollHeight;

  // Fetch and render the dry-run preview.
  const res = await window.cmdos.dryRun(step);
  const area = box.querySelector("#previewArea");
  if (res.ok) {
    const p = res.preview;
    const revColor = p.reversible ? "var(--accent)" : "#f87171";
    const revText = p.reversible ? "Reversible ✓" : "Cannot be undone ✗";
    let html =
      '<div style="font-size:12px; color:var(--text); background:var(--panel-2); ' +
      'border:1px solid var(--border); border-radius:8px; padding:10px; margin-bottom:8px;">' +
      '<div style="color:var(--dim); font-size:10px; letter-spacing:1px; margin-bottom:6px;">PREVIEW — WHAT WILL HAPPEN</div>' +
      '<div style="word-break:break-all;">' + escapeHtml(p.summary) + "</div>" +
      '<div style="color:' + revColor + '; margin-top:6px;">' + revText + "</div>";
    if (p.warnings && p.warnings.length) {
      html += '<div style="color:#facc15; margin-top:6px;">⚠ ' +
        p.warnings.map(escapeHtml).join("<br>⚠ ") + "</div>";
    }
    html += "</div>";
    area.innerHTML = html;
  } else {
    area.innerHTML = '<div style="color:#f87171; font-size:12px;">Preview failed: ' + escapeHtml(res.message) + "</div>";
  }
  workspace.scrollTop = workspace.scrollHeight;

  return new Promise((resolve) => {
    box.querySelector(".approve").onclick = () => { box.remove(); resolve(true); };
    box.querySelector(".deny").onclick = () => { box.remove(); resolve(false); };
  });
}

// Handle one user message end to end.
async function runIntent(text) {
  add('<div class="who">You</div><div>' + escapeHtml(text) + "</div>", "msg you");

 const cm = add('<div class="who">cmdOS</div>', "msg");
  // Animated loading indicator while the agent thinks.
  const loader = document.createElement("div");
  loader.className = "comment loading";
  loader.innerHTML = "// thinking<span>.</span><span>.</span><span>.</span>";
  cm.appendChild(loader);

  const result = await window.cmdos.plan(text, conversation);
  loader.remove(); // stop the loading animation
  if (!result.ok) {
    cm.innerHTML += '<div class="trace" style="color:#f87171;">&#10007; ' + escapeHtml(result.message) + "</div>";
    return;
  }

  const agent = result.plan; // { reply, mode, plan }

  // 1) Always show the agent's spoken reply — typed out gradually.
  if (agent.reply) {
    const replyEl = document.createElement("div");
    replyEl.style.margin = "6px 0";
    cm.appendChild(replyEl);
    await typeText(replyEl, agent.reply, 30);
  }

  // Remember this turn so the agent keeps context next time.
  conversation.push({ role: "user", content: text });
  conversation.push({ role: "assistant", content: agent.reply || "" });
  // Keep only the last ~20 messages to stay light.
  if (conversation.length > 20) conversation.splice(0, conversation.length - 20);

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

    // Offer an Undo button for reversible actions.
    if (res.canUndo) {
      const undoWrap = document.createElement("div");
      undoWrap.style.margin = "6px 0";
      const undoBtn = document.createElement("button");
      undoBtn.className = "btn deny";
      undoBtn.textContent = "↶ Undo";
      undoWrap.appendChild(undoBtn);
      cm.appendChild(undoWrap);

      undoBtn.addEventListener("click", async () => {
        undoBtn.disabled = true;
        undoBtn.textContent = "Undoing…";
        const u = await window.cmdos.undo();
        const result = document.createElement("div");
        result.className = "trace";
        if (!u.ok) result.style.color = "#f87171";
        result.textContent = (u.ok ? "✓ " : "✗ ") + u.message;
        undoWrap.replaceWith(result);
        workspace.scrollTop = workspace.scrollHeight;
      });
      workspace.scrollTop = workspace.scrollHeight;
    }
  }
const doneEl = document.createElement("div");
  doneEl.className = "trace";
  doneEl.textContent = "✓ completed · audit trail written";
  cm.appendChild(doneEl);
  loadReceipts(); // refresh the audit trail
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

// --- Workspace setup ---
async function setupWorkspaceUI() {
  const ws = await window.cmdos.getWorkspace();
  const bar = document.createElement("div");
  bar.className = "panel";
  bar.style.margin = "0 14px 14px";

  function render(path) {
    if (path) {
      bar.innerHTML =
        '<h4>WORKSPACE</h4><div style="color:var(--accent); font-size:12px; word-break:break-all;">📁 ' +
        escapeHtml(path) + '</div>' +
        '<button class="btn deny" id="changeWs" style="margin-top:8px;">Change folder</button>';
    } else {
      bar.innerHTML =
        '<h4>WORKSPACE</h4><div style="color:var(--muted); font-size:12px;">No workspace selected.</div>' +
        '<button class="btn approve" id="pickWs" style="margin-top:8px;">Select workspace</button>';
    }
    const pick = bar.querySelector("#pickWs") || bar.querySelector("#changeWs");
    pick.onclick = async () => {
      const res = await window.cmdos.pickWorkspace();
      if (res.ok) render(res.path);
    };
  }
  render(ws.path);
  workspace.prepend(bar);
}
setupWorkspaceUI();

// --- Audit trail: load real execution receipts into the right column ---
async function loadReceipts() {
  const res = await window.cmdos.getReceipts();
  if (!res.ok) return;

  const historyCol = document.getElementById("historyCol");
  if (!historyCol) return;

  const receipts = res.receipts || [];
  let html =
    '<div style="display:flex; align-items:center; justify-content:space-between; margin-bottom:10px;">' +
    '<h4 style="color:var(--dim); font-size:10px; letter-spacing:1px; margin:0;">AUDIT TRAIL</h4>' +
    '<span style="color:var(--dim); font-size:10px;">' + receipts.length + ' logged</span>' +
    "</div>";

  if (receipts.length === 0) {
    html += '<div style="color:var(--muted); font-size:12px; padding:8px 0;">No actions yet. Everything Alios does will be recorded here with proof.</div>';
    historyCol.innerHTML = html;
    return;
  }

  receipts.slice(0, 30).forEach((r, i) => {
    const ok = r.result === "success";
    const statusColor = ok ? "var(--accent)" : "#f87171";
    const icon = ok ? "&#10003;" : "&#10007;";
    const time = new Date(r.timestamp).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" });
    const date = new Date(r.timestamp).toLocaleDateString();

    html +=
      '<div class="receipt" data-i="' + i + '" ' +
      'style="border:1px solid var(--border); border-left:2px solid ' + statusColor + '; ' +
      'border-radius:8px; padding:10px; margin-bottom:8px; cursor:pointer; background:var(--panel);">' +
      // top row: action + status icon
      '<div style="display:flex; align-items:center; justify-content:space-between;">' +
      '<span style="color:var(--text); font-size:12px; font-weight:700;">' +
      escapeHtml(r.capability) + '.' + escapeHtml(r.action) + '</span>' +
      '<span style="color:' + statusColor + '; font-size:12px;">' + icon + '</span>' +
      '</div>' +
      // meta row
      '<div style="color:var(--muted); font-size:10px; margin-top:4px;">' +
      time +
      (r.approved ? ' &middot; approved' : '') +
      (r.reversible ? ' &middot; undoable' : '') +
      '</div>' +
      // hidden detail (revealed on click)
      '<div class="receipt-detail" style="display:none; margin-top:8px; padding-top:8px; border-top:1px solid var(--border);">' +
      '<div style="color:var(--dim); font-size:10px;">' + escapeHtml(r.description || '') + '</div>' +
      '<div style="color:var(--text); font-size:11px; margin-top:4px; word-break:break-all;">' + escapeHtml(r.message || '') + '</div>' +
      '<div style="color:var(--dim); font-size:9px; margin-top:6px;">' + date + ' &middot; ' + escapeHtml(r.id) + '</div>' +
      '<div style="color:var(--accent); font-size:9px; margin-top:2px;">hash: ' + escapeHtml(r.hash) + '</div>' +
      '</div>' +
      '</div>';
  });

  historyCol.innerHTML = html;

  // Click a receipt to expand/collapse its detail.
  historyCol.querySelectorAll(".receipt").forEach((el) => {
    el.addEventListener("click", () => {
      const detail = el.querySelector(".receipt-detail");
      detail.style.display = detail.style.display === "none" ? "block" : "none";
    });
  });
}

loadReceipts();