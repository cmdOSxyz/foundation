// Visibility: private
// kernel/inspect.demo.ts
import { inspectPath } from "../capabilities/filesystem.js";

async function main() {
  console.log("Existing file:");
  console.log(await inspectPath("sandbox/report.txt"));

  console.log("\nMissing file:");
  console.log(await inspectPath("sandbox/nope.txt"));

  console.log("\nFolder:");
  console.log(await inspectPath("sandbox"));
}
main();
