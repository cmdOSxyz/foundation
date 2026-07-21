// Visibility: private
// kernel/mock-planner.ts
// A Planner that produces a fixed plan without any AI. Used to prove the
// Intent -> Planner -> Executor flow before wiring a real model. Swappable with an
// AI-backed planner later, because it satisfies the same Planner contract.

import type { Planner, Intent, ExecutionPlan } from "../../schemas/index.js";

/** Generate a short random id. */
function id(prefix: string): string {
  return prefix + "-" + Math.random().toString(36).slice(2, 8);
}

/**
 * A stand-in planner. It ignores the fine detail of the intent and returns a
 * simple two-step filesystem plan, so the rest of the pipeline can run today.
 */
export class MockPlanner implements Planner {
  async plan(intent: Intent): Promise<ExecutionPlan> {
    const planId = id("plan");

    return {
      id: planId,
      intentId: intent.id,
      createdAt: new Date().toISOString(),
      status: "draft",
      summary: "Mock plan for: " + intent.rawText,
      steps: [
        {
          id: id("step"),
          description: "List files in the target folder",
          capability: "filesystem",
          action: "list",
          parameters: { path: "C:/demo" },
          dependsOn: [],
          requiresPermission: false,
          status: "pending",
        },
        {
          id: id("step"),
          description: "Rename the first file",
          capability: "filesystem",
          action: "rename",
          parameters: { from: "old.txt", to: "new.txt" },
          dependsOn: [],
          requiresPermission: true,
          status: "pending",
        },
      ],
    };
  }
}