// Visibility: private
// kernel/verify-loop.demo.ts
// Runs a plan where each step is verified after it runs. If verification fails,
// the step is treated as failed and the plan stops safely.
// Demonstrates: Runtime -> Verification part of the canonical loop.

import { EventLog } from "./event-log.js";
import { Executor } from "./executor.js";
import { runFilesystemStep, verifyFilesystemStep } from "../capabilities/filesystem.js";
import type { PlanStep, ExecutionPlan } from "../schemas/index.js";

// Build a plan. Set BREAK_IT to true to make step 2 target a missing file,
// so verification catches the failure.
const BREAK_IT = false;

const plan: ExecutionPlan = {
  id: "plan-1",
  intentId: "intent-1",
  createdAt: new Date().toISOString(),
  status: "approved",
  summary: "Rename with verification",
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
      parameters: {
        from: BREAK_IT ? "sandbox/does-not-exist.txt" : "sandbox/report.txt",
        to: "report-2026.txt",
      },
      dependsOn: ["s1"],
      requiresPermission: false,
      status: "pending",
    },
  ],
};

async function main() {
  const log = new EventLog();
  const executor = new Executor(log);

  // Step runner: run the capability, THEN verify. Fail if verification fails.
  const runStep = async (step: PlanStep): Promise<void> => {
    const result = await runFilesystemStep(step);
    console.log("    " + result.message);

    const check = await verifyFilesystemStep(step);
    console.log("    " + check.message);
    if (!check.ok) {
      throw new Error(check.message);
    }
  };

  console.log("Running plan (BREAK_IT=" + BREAK_IT + ")...\n");
  const ok = await executor.run(plan, runStep);

  console.log("\nResult: " + (ok ? "SUCCESS" : "FAILED (stopped safely)"));
  console.log("\nEvent trail (" + log.count() + " events):");
  for (const e of log.all()) {
    console.log("  [" + e.type + "] " + e.message);
  }
}

main().catch((err) => {
  console.error("FAILED:", err.message);
});