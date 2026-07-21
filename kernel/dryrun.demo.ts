// Visibility: private
// kernel/dryrun.demo.ts
import { dryRunFilesystemStep } from "../capabilities/filesystem.js";

function step(action, params) {
  return { id: "t", description: action, capability: "filesystem", action, parameters: params, dependsOn: [], requiresPermission: false, status: "pending" };
}

async function main() {
  console.log("RENAME preview:");
  console.log(await dryRunFilesystemStep(step("rename", { from: "sandbox/report.txt", to: "done.txt" })));
  console.log("\nDELETE preview:");
  console.log(await dryRunFilesystemStep(step("delete", { path: "sandbox/report.txt" })));
}
main().catch(e => console.error("FAILED:", e.message));
