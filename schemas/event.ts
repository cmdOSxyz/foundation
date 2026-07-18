// Visibility: public
// schemas/event.ts
// An Event is an immutable record of something that happened in cmdOS.
// Events form an append-only log: they are never modified or deleted, only added.
// This is the foundation of observability and audit — every meaningful action is traceable.

import type { Id, Timestamp } from "./intent";

/** The kind of thing that happened. */
export type EventType =
  | "intent_received"
  | "intent_understood"
  | "plan_created"
  | "permission_requested"
  | "permission_approved"
  | "permission_denied"
  | "step_started"
  | "step_succeeded"
  | "step_failed"
  | "plan_completed"
  | "plan_failed"
  | "plan_rolled_back";

/** How severe / important the event is. */
export type EventSeverity = "info" | "warning" | "error";

/**
 * A single immutable event in the append-only log.
 * Once created, an Event is never changed.
 */
export interface Event {
  id: Id;
  type: EventType;
  severity: EventSeverity;
  timestamp: Timestamp;
  subject: {
    intentId?: Id;
    planId?: Id;
    stepId?: Id;
    permissionRequestId?: Id;
  };
  message: string;
  data?: Record<string, unknown>;
}