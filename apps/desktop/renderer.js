// Visibility: private
// apps/desktop/renderer.js
// Layer 2: interactive UI with MOCK data (no real execution yet).
// Typing an intent runs a fake plan through the on-screen flow.

const workspace = document.getElementById("workspace");
const input = document.getElementById("cmdInput");

// Small helper to append an element to the workspace and scroll down.
function add(html, className) {
  const div = document.createElement("div");
  div.className = className || "";
  div.innerHTML = html;
  workspace.appendChild(div);
  workspace.scrollTop = workspace.scrollHeight;
  return div;
}

// Pause helper so the flow feels alive.
const wait = (ms) => new Promise((r) => setTimeout(r, ms));

// A fake plan builder (stands in for the Planner). Two steps: list + rename.
function mockPlan(intentText) {
  return {
    summary: "Plan for: " + intentText,
    steps: [
      { description: "List the sandbox folder", capability: "filesystem", action: "list",
        parameters: { path: "sandbox" }, requiresPermission: false },
      { description: "Rename report.txt to report-2026.txt", capability: "filesystem", action: "rename",
        parameters: { from: "sandbox/report.txt", to: "report-2026.txt" }, requiresPermission: true },
    ],
  };
}

// Run one intent through the mock flow.
async function runIntent(text) {
  add('<div class="who">You</div><div>' + text + "</div>", "msg you");

  const cm = add('<div class="who">cmdOS</div>', "msg");
  await wait(300);
  cm.innerHTML += '<div class="trace"><span class="diamond">&#9671;</span> understanding &middot; ' + text + "</div>";

  const plan = mockPlan(text);
  await wait(400);
  cm.innerHTML += '<div class="comment">// planned ' + plan.steps.length + " steps</div>";

  // Run each step. Sensitive steps pause for approval.
  for (const step of plan.steps) {
    if (step.requiresPermission) {
      const approved = await askApproval(cm, step);
      if (!approved) {
        cm.innerHTML += '<div class="trace" style="color:#f87171;">&#10007; denied &middot; stopped safely</div>';
        return;
      }
    }
    // Real execution: send the approved step to main.
    const res = await window.cmdos.runStep({
      id: "s-" + Math.random().toString(36).slice(2, 6),
      description: step.description,
      capability: "filesystem",
      action: step.action,
      parameters: step.parameters || {},
      dependsOn: [],
      requiresPermission: step.requiresPermission,
      status: "pending",
    });
    const mark = res.ok ? "&#10003;" : "&#10007;";
    const color = res.ok ? "" : ' style="color:#f87171;"';
    cm.innerHTML += '<div class="trace"' + color + ">" + mark + " " + res.message + "</div>";
    if (!res.ok) return;
  }

  cm.innerHTML += '<div class="trace">&#10003; completed &middot; audit trail written</div>';
}

// Show an approval box inside the given message; resolve true/false on click.
function askApproval(container, step) {
  return new Promise((resolve) => {
    const box = document.createElement("div");
    box.className = "approval";
    box.innerHTML =
      '<div class="label">CMDOS WANTS YOUR APPROVAL</div>' +
      '<div class="cmd">' + step.capability + "." + step.action + '</div>' +
      '<button class="btn approve">Approve</button>' +
      '<button class="btn deny">Deny</button>';
    container.appendChild(box);
    workspace.scrollTop = workspace.scrollHeight;

    box.querySelector(".approve").onclick = () => { box.remove(); resolve(true); };
    box.querySelector(".deny").onclick = () => { box.remove(); resolve(false); };
  });
}

// Wire the input: Enter runs the intent.
input.addEventListener("keydown", (e) => {
  if (e.key === "Enter" && input.value.trim()) {
    const text = input.value.trim();
    input.value = "";
    runIntent(text);
  }
});
// --- 3B: test the bridge ---
async function testBridge() {
  if (!window.cmdos) {
    console.error("Bridge NOT available");
    return;
  }
  const reply = await window.cmdos.ping("hello");
  console.log("Bridge reply:", reply);
}
testBridge();