import { readdirSync, readFileSync, statSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { join } from "node:path";
import { forbiddenRenderedDemoStrings, requiredChecks } from "./verify-workbench-checks.mjs";

const root = fileURLToPath(new URL("..", import.meta.url));
const sourceExtensions = new Set([".css", ".rs", ".ts", ".tsx"]);
const noInlineStyleFiles = [
  "src/app/cockpitMarkup.ts",
  "src/app/cockpitView.ts",
  "src/app/cockpitReview.ts",
  "src/app/cockpitEvidence.ts",
];

const failures = [];

for (const [file, expected] of requiredChecks) {
  const contents = readFileSync(join(root, file), "utf8");
  if (!contents.includes(expected)) {
    failures.push(`${file} is missing ${expected}`);
  }
}

for (const file of noInlineStyleFiles) {
  const contents = readFileSync(join(root, file), "utf8");
  if (contents.includes('style="')) {
    failures.push(`${file} contains inline style attributes; use cockpit.css classes`);
  }
}

for (const file of listFiles(join(root, "src"))) {
  const extension = file.slice(file.lastIndexOf("."));
  if (!sourceExtensions.has(extension)) {
    continue;
  }

  const contents = readFileSync(file, "utf8");
  for (const forbidden of forbiddenRenderedDemoStrings) {
    if (contents.includes(forbidden)) {
      failures.push(`${file} still contains demo string ${forbidden}`);
    }
  }
}

for (const file of listFiles(root)) {
  const extension = file.slice(file.lastIndexOf("."));
  if (!sourceExtensions.has(extension)) {
    continue;
  }

  const lineCount = readFileSync(file, "utf8").split(/\r?\n/).length;
  if (lineCount > 300) {
    failures.push(`${file} has ${lineCount} lines; limit is 300`);
  }
}

if (failures.length > 0) {
  console.error(failures.join("\n"));
  process.exit(1);
}

console.log("Workbench verifier passed: workspace, thread wiring, and file-size budget are intact.");

function listFiles(directory) {
  return readdirSync(directory).flatMap((entry) => {
    const path = join(directory, entry);
    const stats = statSync(path);

    if (stats.isDirectory()) {
      if (["dist", "node_modules", "target"].includes(entry)) {
        return [];
      }

      return listFiles(path);
    }

    return [path];
  });
}

