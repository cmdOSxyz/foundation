// Visibility: private
// kernel/event-log.demo.ts
// A quick manual run to see the EventLog working. Not a real test suite — just proof it runs.

import { EventLog } from "./event-log.js";
import type { Event } from "../schemas/index.js";

const log = new EventLog();

// Append two events.
const e1: Event = {
  id: "evt-1",
  type: "intent_received",
  severity: "info",
  timestamp: new Date().toISOString(),
  subject: { intentId: "intent-1" },
  message: "User asked to rename 3 files",
};

const e2: Event = {
  id: "evt-2",
  type: "plan_created",
  severity: "info",
  timestamp: new Date().toISOString(),
  subject: { intentId: "intent-1", planId: "plan-1" },
  message: "Plan created with 3 steps",
};

log.append(e1);
log.append(e2);

// Read them back.
console.log("Total events:", log.count());
console.log("All events:");
for (const e of log.all()) {
  console.log(`  [${e.type}] ${e.message}`);
}

// Prove immutability: try to change a stored event — it must NOT change.
const stored = log.all()[0];
try {
  stored.message = "HACKED";
} catch {
  // frozen objects throw in strict mode; that's fine
}
console.log("First event message after tamper attempt:", log.all()[0]?.message);