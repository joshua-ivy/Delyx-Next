import { readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import ts from "typescript";

const root = fileURLToPath(new URL("..", import.meta.url));
const failures = [];
const module = await loadTsModule("src/features/plans/ollamaPlan.ts");

const thread = {
  goal: "Wire Ollama into the PlanAgent",
  id: "thread-ollama-plan",
  messages: [{ body: "Use my local model for the agent plan.", role: "user" }],
};
const project = {
  approvedRoots: ["C:/Users/geaux/Downloads/Delyx Next"],
  indexedFiles: [
    "package.json",
    "apps/desktop/src/app/appShellOllamaPlanActions.ts",
    "apps/desktop/src/features/plans/ollamaPlan.ts",
  ],
  name: "Delyx Next",
  path: "C:/Users/geaux/Downloads/Delyx Next",
};

const modelResponse = [
  "Before",
  "```json",
  `{
  "goalUnderstanding": "Use Ollama for read-only plan drafting.",
  "architectureSummary": "React and TypeScript workspace.",
  "filesLikelyInvolved": [
    "apps/desktop/src/features/plans/ollamaPlan.ts",
    "C:/Windows/system32/config"
  ],
  "relevantSymbols": ["createPlanWithOllama"],
  "steps": [{"description": "Check provider readiness"}, "Draft typed plan", "Request approval before writes"],
  "risks": ["Model output may be malformed"],
  "testsToRun": ["npm test"],
  "permissionsNeeded": ["model_call"],
  "rollbackStrategy": "Keep changes in reviewable patches.",
  "unknowns": ["Exact local model name"],
  "suggestedNextSteps": ["Run deterministic verifier"]
}`,
  "```",
].join("\n");

const plan = module.createPlanFromOllamaText(thread, project, modelResponse);

check(plan.steps.length === 3, "fixture plan keeps model-provided steps");
check(plan.filesLikelyInvolved.includes("apps/desktop/src/features/plans/ollamaPlan.ts"), "approved model file is retained");
check(!plan.filesLikelyInvolved.includes("C:/Windows/system32/config"), "outside file reference is rejected");
check(plan.permissionsNeeded.includes("approval required before file edits"), "safety permissions are merged");
check(plan.explore.projectCommands.includes("npm test"), "project commands are derived from approved index");

const messages = module.createOllamaPlanMessages(thread, project);
const promptText = messages.map((message) => message.content).join("\n");
check(promptText.includes("Return only JSON"), "PlanAgent prompt requires JSON");
check(promptText.includes("Use only the approved indexed files"), "PlanAgent prompt limits file claims");
check(promptText.includes("apps/desktop/src/app/appShellOllamaPlanActions.ts"), "PlanAgent prompt includes approved index");

try {
  module.createPlanFromOllamaText(thread, project, "{\"steps\": []}");
  failures.push("empty model steps must fail parsing");
} catch (error) {
  check(String(error).includes("plan steps"), "empty model steps failure is explicit");
}

if (failures.length > 0) {
  console.error(failures.join("\n"));
  process.exit(1);
}

console.log("Ollama agent verifier passed: PlanAgent JSON parsing and approved-file filtering are deterministic.");

async function loadTsModule(file) {
  const source = readFileSync(join(root, file), "utf8");
  const output = ts.transpileModule(source, {
    compilerOptions: {
      module: ts.ModuleKind.ES2022,
      target: ts.ScriptTarget.ES2022,
    },
  }).outputText;
  return import(`data:text/javascript;base64,${Buffer.from(output).toString("base64")}`);
}

function check(condition, message) {
  if (!condition) {
    failures.push(message);
  }
}
