// Visibility: private
// kernel/executor.demo.ts
import { EventLog } from "./event-log.js";
import { Executor } from "./executor.js";
import type { ExecutionPlan, PlanStep } from "../schemas/index.js";

const plan: ExecutionPlan = {
  id: "plan-1",
  intentId: "intent-1",
  createdAt: new Date().toISOString(),
  status: "approved",
  summary: "Demo plan: pretend to tidy up a folder",
  steps: [
    {
      id: "step-1",
      description: "List files in the folder",
      capability: "filesystem",
      action: "list",
      parameters: { path: "C:/demo" },
      dependsOn: [],
      requiresPermission: false,
      status: "pending",
    },
    {
      id: "step-2",
      description: "Rename report.txt to report-2026.txt",
      capability: "filesystem",
      action: "rename",
      parameters: { from: "report.txt", to: "report-2026.txt" },
      dependsOn: ["step-1"],
      requiresPermission: true,
      status: "pending",
    },
    {
      id: "step-3",
      description: "Report the result",
      capability: "filesystem",
      action: "summarize",
      parameters: {},
      dependsOn: ["step-2"],
      requiresPermission: false,
      status: "pending",
    },
  ],
};

const fakeRunStep = async (step: PlanStep): Promise<void> => {
  console.log("    -> running " + step.capability + "." + step.action);
};

async function main() {
  const log = new EventLog();
  const executor = new Executor(log);

  console.log("Running plan...");
  const ok = await executor.run(plan, fakeRunStep);

  console.log("Plan result: " + (ok ? "SUCCESS" : "FAILED"));
  console.log("Event trail (" + log.count() + " events):");
  for (const e of log.all()) {
    console.log("  [" + e.type + "] " + e.message);
  }
}

main();