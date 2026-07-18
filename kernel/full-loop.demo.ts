// Visibility: private
// kernel/full-loop.demo.ts
// The first end-to-end run with a real permission gate and real files.
// Intent -> Planner -> (per step) PermissionGate -> Filesystem -> Event trail.

import { createInterface } from "node:readline/promises";
import { stdin, stdout } from "node:process";

import { EventLog } from "./event-log.js";
import { Executor } from "./executor.js";
import { PermissionGate } from "./permission-gate.js";
import { runFilesystemStep } from "../capabilities/filesystem.js";
import type { Intent, PlanStep, PermissionRequest, ExecutionPlan } from "../schemas/index.js";

// Console approver: prints the request and reads y/n from the user.
const rl = createInterface({ input: stdin, output: stdout });
const askUser = async (req: PermissionRequest): Promise<boolean> => {
  console.log("\n  [PERMISSION] " + req.summary);
  console.log("  details: " + JSON.stringify(req.details));
  const answer = await rl.question("  Approve? (y/n) ");
  return answer.trim().toLowerCase() === "y";
};

// A hand-built plan targeting the real sandbox folder.
const intent: Intent = {
  id: "intent-1",
  rawText: "Rename report.txt in my sandbox",
  source: "user_command",
  createdAt: new Date().toISOString(),
  status: "understood",
};

const plan: ExecutionPlan = {
  id: "plan-1",
  intentId: intent.id,
  createdAt: new Date().toISOString(),
  status: "approved",
  summary: "Rename report.txt to report-2026.txt in sandbox",
  steps: [
    {
      id: "s1",
      description: "List the sandbox folder",
      capability: "filesystem",
      action: "list",
      parameters: { path: "sandbox" },
      dependsOn: [],
      requiresPermission: false,
      status: "pending",
    },
    {
      id: "s2",
      description: "Rename report.txt to report-2026.txt",
      capability: "filesystem",
      action: "rename",
      parameters: { from: "sandbox/report.txt", to: "report-2026.txt" },
      dependsOn: ["s1"],
      requiresPermission: true,
      status: "pending",
    },
  ],
};

async function main() {
  const log = new EventLog();
  const gate = new PermissionGate(askUser);
  const executor = new Executor(log);

  // The step runner checks permission first, then runs the real capability.
  const runStep = async (step: PlanStep): Promise<void> => {
    const allowed = await gate.check(plan.id, step);
    if (!allowed) {
      log.append({
        id: "evt-denied-" + step.id,
        type: "permission_denied",
        severity: "warning",
        timestamp: new Date().toISOString(),
        subject: { planId: plan.id, stepId: step.id },
        message: "User denied: " + step.description,
      });
      throw new Error("permission denied by user");
    }
    const result = await runFilesystemStep(step);
    console.log("    " + result.message);
  };

  console.log("Intent: " + intent.rawText);
  const ok = await executor.run(plan, runStep);

  console.log("\nResult: " + (ok ? "SUCCESS" : "FAILED (stopped safely)"));
  console.log("\nEvent trail (" + log.count() + " events):");
  for (const e of log.all()) {
    console.log("  [" + e.type + "] " + e.message);
  }

  rl.close();
}

main().catch((err) => {
  console.error("FAILED:", err.message);
  rl.close();
});