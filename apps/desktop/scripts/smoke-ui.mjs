import { existsSync, readFileSync, readdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { join } from "node:path";

const desktopRoot = fileURLToPath(new URL("..", import.meta.url));
const failures = [];
const source = [
  "src/app/cockpitMarkup.ts",
  "src/features/workspace/WorkspaceOverlay.tsx",
  "src/features/threads/ThreadOverlay.tsx",
  "src/app/cockpitReview.ts",
  "src/app/cockpitEvidence.ts",
].map((file) => readFileSync(join(desktopRoot, file), "utf8")).join("\n");

check(existsSync(join(desktopRoot, "dist", "index.html")), "dist/index.html must exist; run npm run build first");
check(assetExists("assets", ".js"), "built JS asset must exist");
check(assetExists("assets", ".css"), "built CSS asset must exist");

for (const label of ["Projects", "Threads", "Plan", "Approvals", "Diff", "Tests", "Evidence"]) {
  check(source.includes(label), `UI source must include ${label}`);
}
for (const state of ["Loading state", "Error state", "blocked", "failed", "done"]) {
  check(source.includes(state), `UI source must include ${state}`);
}
for (const action of ["Create plan", "Approve", "Revise", "Cancel"]) {
  check(source.includes(action), `primary workflow action must include ${action}`);
}

if (failures.length > 0) {
  console.error(failures.join("\n"));
  process.exit(1);
}

console.log("UI smoke passed: built assets and primary workflow states are present.");

function assetExists(directory, extension) {
  const path = join(desktopRoot, "dist", directory);
  return existsSync(path) && readdirSync(path).some((entry) => entry.endsWith(extension));
}

function check(condition, message) {
  if (!condition) {
    failures.push(message);
  }
}
