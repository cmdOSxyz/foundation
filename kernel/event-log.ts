// Visibility: private
// kernel/event-log.ts
// Append-only event log. Events can only be added and read, never modified or deleted.
// This is the observability + audit foundation for the whole runtime.

import type { Event } from "../schemas/index.js";

/**
 * An append-only log of events. Once appended, an event is immutable.
 * Reads return copies so callers cannot mutate the stored history.
 */
export class EventLog {
  /** Internal store. Private so nothing outside can rewrite history. */
  private readonly events: Event[] = [];

  /** Append one event to the log. Returns the event that was stored. */
  append(event: Event): Event {
    // Freeze to guarantee immutability at runtime.
    const frozen = Object.freeze({ ...event });
    this.events.push(frozen);
    return frozen;
  }

  /** Return all events, oldest first. Returns a copy, not the internal array. */
  all(): Event[] {
    return [...this.events];
  }

  /** Return only events matching a given type. */
  byType(type: Event["type"]): Event[] {
    return this.events.filter((e) => e.type === type);
  }

  /** Return all events related to a given plan id. */
  byPlan(planId: string): Event[] {
    return this.events.filter((e) => e.subject.planId === planId);
  }

  /** Number of events recorded so far. */
  count(): number {
    return this.events.length;
  }
}