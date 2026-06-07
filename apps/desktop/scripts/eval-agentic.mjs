import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { join } from "node:path";

const desktopRoot = fileURLToPath(new URL("..", import.meta.url));
const cases = JSON.parse(readFileSync(join(desktopRoot, "evals", "agentic-cases.json"), "utf8"));
const failures = [];

for (const testCase of cases) {
  evaluateCase(testCase);
}

if (failures.length > 0) {
  console.error(failures.join("\n"));
  process.exit(1);
}

console.log(`Agentic eval passed: ${cases.length} deterministic fixture(s).`);

function evaluateCase(testCase) {
  const text = `${testCase.candidate.planText} ${testCase.candidate.traceMarkers.join(" ")}`;
  for (const expected of testCase.expect.mustInclude) {
    if (!text.includes(expected)) {
      failures.push(`${testCase.id}: missing ${expected}`);
    }
  }
  for (const forbidden of testCase.expect.mustNotInclude) {
    if (text.includes(forbidden)) {
      failures.push(`${testCase.id}: forbidden ${forbidden}`);
    }
  }
  for (const marker of testCase.expect.requiredTraceMarkers) {
    if (!testCase.candidate.traceMarkers.includes(marker)) {
      failures.push(`${testCase.id}: missing trace marker ${marker}`);
    }
  }
  if (testCase.candidate.executedRiskyActions.length > 0) {
    failures.push(`${testCase.id}: risky action executed before approval`);
  }
}
