import { existsSync, readFileSync, readdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { join } from "node:path";
import { forbiddenRenderedDemoStrings } from "./verify-workbench-checks.mjs";

const desktopRoot = fileURLToPath(new URL("..", import.meta.url));
const failures = [];
const distRoot = join(desktopRoot, "dist");
const indexPath = join(distRoot, "index.html");
const builtOutput = existsSync(indexPath)
  ? [readFileSync(indexPath, "utf8"), ...assetText("assets", [".js", ".css"])].join("\n")
  : "";
const source = [
  "src/app/cockpitMarkup.ts",
  "src/app/cockpitMessageFormat.ts",
  "src/app/cockpitWorkPane.ts",
  "src/app/appShellModelRunActions.ts",
  "src/app/cockpitPlanBindings.ts",
  "src/app/runtimeBridge.ts",
  "src/app/workspaceBridge.ts",
  "src/app/cockpitStats.ts",
  "src/features/workspace/WorkspaceOverlay.tsx",
  "src/features/threads/ThreadOverlay.tsx",
  "src/app/cockpitReview.ts",
  "src/app/cockpitEvidence.ts",
].map((file) => readFileSync(join(desktopRoot, file), "utf8")).join("\n");

check(existsSync(indexPath), "dist/index.html must exist; run npm run build first");
check(assetExists("assets", ".js"), "built JS asset must exist");
check(assetExists("assets", ".css"), "built CSS asset must exist");

for (const label of ["Projects", "Threads", "Plan", "Approval", "Diff", "Tests", "Evidence"]) {
  check(source.includes(label), `UI source must include ${label}`);
}
for (const state of ["Loading:", "Error:", "blocked", "failed", "done"]) {
  check(source.includes(state), `UI source must include ${state}`);
}
for (const action of ["Create plan", "Approve", "Ask question", "Show review"]) {
  check(source.includes(action), `primary workflow action must include ${action}`);
}
for (const marker of [
  "Command Deck workbench",
  "No active thread",
  "No approval requests",
  "No patch or file change has been proposed",
  "No test command artifact has been captured",
  "No terminal command has run",
  "No external agent run has been approved or captured",
  "Web preview / Rust bridge unavailable",
  "No indexed files are loaded for this query",
]) {
  check(builtOutput.includes(marker), `built UI must include real first-run marker: ${marker}`);
}
for (const forbidden of forbiddenRenderedDemoStrings) {
  check(!builtOutput.includes(forbidden), `built UI must not render demo string: ${forbidden}`);
}

if (failures.length > 0) {
  console.error(failures.join("\n"));
  process.exit(1);
}

console.log("UI smoke passed: built assets, real empty states, and primary workflow markers are present.");

function assetExists(directory, extension) {
  const path = join(distRoot, directory);
  return existsSync(path) && readdirSync(path).some((entry) => entry.endsWith(extension));
}

function check(condition, message) {
  if (!condition) {
    failures.push(message);
  }
}

function assetText(directory, extensions) {
  const path = join(distRoot, directory);
  if (!existsSync(path)) {
    return [];
  }

  return readdirSync(path)
    .filter((entry) => extensions.some((extension) => entry.endsWith(extension)))
    .map((entry) => readFileSync(join(path, entry), "utf8"));
}
