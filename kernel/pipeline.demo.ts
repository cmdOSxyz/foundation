// Visibility: private
// kernel/pipeline.demo.ts
// End-to-end demo of the first half of the canonical loop:
// Intent -> Planner -> Executor -> Event trail.

import { EventLog } from "./event-log.js";
import { Executor } from "./executor.js";
import { MockPlanner } from "./mock-planner.js";
import type { Intent, PlanStep } from "../schemas/index.js";

// 1. A user intent, as if typed into cmdOS.
const intent: Intent = {
  id: "intent-1",
  rawText: "Tidy up my demo folder",
  source: "user_command",
  createdAt: new Date().toISOString(),
  status: "understood",
};

// 2. A fake step runner (real capabilities plug in here in M3).
const fakeRunStep = async (step: PlanStep): Promise<void> => {
  console.log("    -> running " + step.capability + "." + step.action);
};

async function main() {
  const log = new EventLog();
  const planner = new MockPlanner();
  const executor = new Executor(log);

  console.log("Intent: " + intent.rawText + "\n");

  // 3. Turn the intent into a plan.
  const plan = await planner.plan(intent);
  console.log("Planned " + plan.steps.length + " steps: " + plan.summary + "\n");

  // 4. Execute the plan.
  const ok = await executor.run(plan, fakeRunStep);

  // 5. Show the result and the event trail.
  console.log("\nResult: " + (ok ? "SUCCESS" : "FAILED"));
  console.log("\nEvent trail (" + log.count() + " events):");
  for (const e of log.all()) {
    console.log("  [" + e.type + "] " + e.message);
  }
}

main();