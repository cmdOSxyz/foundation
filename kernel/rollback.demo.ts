// Visibility: private
// kernel/rollback.demo.ts
// A plan renames a.txt then b.txt, but step 3 fails on purpose.
// The runtime undoes the two successful renames in reverse order,
// leaving the folder exactly as it started. Demonstrates transaction + recovery.

import { EventLog } from "./event-log.js";
import {
  runFilesystemStep,
  undoForFilesystemStep,
  type UndoFn,
} from "../capabilities/filesystem.js";
import type { PlanStep, ExecutionPlan } from "../schemas/index.js";

const plan: ExecutionPlan = {
  id: "plan-1",
  intentId: "intent-1",
  createdAt: new Date().toISOString(),
  status: "approved",
  summary: "Rename two files, then fail on purpose",
  steps: [
    {
      id: "s1", description: "Rename a.txt", capability: "filesystem", action: "rename",
      parameters: { from: "sandbox/a.txt", to: "a-done.txt" },
      dependsOn: [], requiresPermission: false, status: "pending",
    },
    {
      id: "s2", description: "Rename b.txt", capability: "filesystem", action: "rename",
      parameters: { from: "sandbox/b.txt", to: "b-done.txt" },
      dependsOn: ["s1"], requiresPermission: false, status: "pending",
    },
    {
      id: "s3", description: "Rename a missing file (will fail)", capability: "filesystem", action: "rename",
      parameters: { from: "sandbox/missing.txt", to: "x.txt" },
      dependsOn: ["s2"], requiresPermission: false, status: "pending",
    },
  ],
};

async function main() {
  const log = new EventLog();
  const undos: UndoFn[] = [];

  console.log("Start:", await listSandbox());

  let failed = false;
  for (const step of plan.steps) {
    try {
      const result = await runFilesystemStep(step);
      console.log("  OK   " + result.message);
      const undo = undoForFilesystemStep(step);
      if (undo) undos.push(undo);
    } catch (err) {
      const reason = err instanceof Error ? err.message : String(err);
      console.log("  FAIL " + step.description + " — " + reason);
      failed = true;
      break;
    }
  }

  if (failed) {
    console.log("\nRolling back " + undos.length + " step(s)...");
    // Undo in reverse order.
    for (const undo of undos.reverse()) {
      await undo();
    }
    console.log("Rollback done.");
  }

  console.log("\nEnd:  ", await listSandbox());
  console.log("\nResult:", failed ? "FAILED and ROLLED BACK" : "SUCCESS");
}

// Small helper to show the folder contents.
async function listSandbox(): Promise<unknown> {
  const r = await runFilesystemStep({
    id: "ls", description: "list", capability: "filesystem", action: "list",
    parameters: { path: "sandbox" }, dependsOn: [], requiresPermission: false, status: "pending",
  });
  return r.data;
}

main().catch((err) => console.error("FAILED:", err.message));