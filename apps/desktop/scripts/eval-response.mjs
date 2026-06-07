import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { join } from "node:path";

const desktopRoot = fileURLToPath(new URL("..", import.meta.url));
const cases = JSON.parse(readFileSync(join(desktopRoot, "evals", "response-cases.json"), "utf8"));
const failures = [];

for (const testCase of cases) {
  evaluateCase(testCase);
}

if (failures.length > 0) {
  console.error(failures.join("\n"));
  process.exit(1);
}

console.log(`Response eval passed: ${cases.length} deterministic fixture(s).`);

function evaluateCase(testCase) {
  const evidenceIds = new Set(testCase.evidenceRecords.map((record) => record.id));
  for (const claim of testCase.answer.claims) {
    if (claim.requiresEvidence && claim.evidenceIds.length === 0) {
      failures.push(`${testCase.id}: claim ${claim.id} has no evidence`);
    }
    for (const evidenceId of claim.evidenceIds) {
      if (!evidenceIds.has(evidenceId)) {
        failures.push(`${testCase.id}: claim ${claim.id} references missing evidence ${evidenceId}`);
      }
    }
  }
  if (testCase.contradictions.length > 0 && !testCase.answer.markers.includes("contradiction")) {
    failures.push(`${testCase.id}: contradiction marker is missing`);
  }
}
