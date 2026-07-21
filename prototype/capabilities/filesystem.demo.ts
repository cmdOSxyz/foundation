// Visibility: private
// capabilities/filesystem.demo.ts
// Runs the real filesystem capability on a real folder (./sandbox).
// It lists the folder, renames report.txt -> report-2026.txt, then lists again.

import { runFilesystemStep } from "./filesystem.js";
import type { PlanStep } from "../../schemas/index.js";

// Helper to build a minimal PlanStep for the demo.
function step(action: string, parameters: Record<string, unknown>): PlanStep {
  return {
    id: "step-" + action,
    description: "demo " + action,
    capability: "filesystem",
    action,
    parameters,
    dependsOn: [],
    requiresPermission: action !== "list",
    status: "pending",
  };
}

async function main() {
  console.log("Before:");
  let r = await runFilesystemStep(step("list", { path: "sandbox" }));
  console.log("  " + r.message, r.data);

  console.log("\nRenaming...");
  r = await runFilesystemStep(
    step("rename", { from: "sandbox/report.txt", to: "report-2026.txt" }),
  );
  console.log("  " + r.message);

  console.log("\nAfter:");
  r = await runFilesystemStep(step("list", { path: "sandbox" }));
  console.log("  " + r.message, r.data);
}

main().catch((err) => {
  console.error("FAILED:", err.message);
});