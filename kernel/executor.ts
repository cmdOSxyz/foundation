// Visibility: private
// kernel/executor.ts
// The Executor runs an ExecutionPlan step by step, in dependency order, emitting an
// Event for every transition. It implements the Command -> Runtime part of the loop.
// It does NOT talk to the AI or the real filesystem yet — a StepRunner is injected,
// so the same executor can later drive real capabilities.

import type { ExecutionPlan, PlanStep, Event, EventType } from "../schemas/index.js";
import { EventLog } from "./event-log.js";

/** A function that actually performs one step. Returns nothing on success, throws on failure. */
export type StepRunner = (step: PlanStep) => Promise<void>;

/** Small helper to build an Event with common fields filled in. */
function makeEvent(
  type: EventType,
  message: string,
  subject: Event["subject"],
): Event {
  return {
    id: `evt-${Math.random().toString(36).slice(2, 10)}`,
    type,
    severity: type.endsWith("_failed") ? "error" : "info",
    timestamp: new Date().toISOString(),
    subject,
    message,
  };
}

/**
 * Runs plans. Given a plan and a StepRunner, it executes steps in order,
 * records every transition in the EventLog, and stops on the first failure.
 */
export class Executor {
  constructor(private readonly log: EventLog) {}

  /** Execute a whole plan. Returns true if all steps succeeded. */
  async run(plan: ExecutionPlan, runStep: StepRunner): Promise<boolean> {
    this.log.append(
      makeEvent("plan_created", plan.summary, { planId: plan.id, intentId: plan.intentId }),
    );

    for (const step of plan.steps) {
      this.log.append(
        makeEvent("step_started", step.description, { planId: plan.id, stepId: step.id }),
      );

      try {
        await runStep(step);
        this.log.append(
          makeEvent("step_succeeded", step.description, { planId: plan.id, stepId: step.id }),
        );
      } catch (err) {
        const reason = err instanceof Error ? err.message : String(err);
        this.log.append(
          makeEvent("step_failed", `${step.description} — ${reason}`, {
            planId: plan.id,
            stepId: step.id,
          }),
        );
        this.log.append(
          makeEvent("plan_failed", `Plan stopped at step "${step.description}"`, {
            planId: plan.id,
          }),
        );
        return false;
      }
    }

    this.log.append(
      makeEvent("plan_completed", "All steps succeeded", { planId: plan.id }),
    );
    return true;
  }
}