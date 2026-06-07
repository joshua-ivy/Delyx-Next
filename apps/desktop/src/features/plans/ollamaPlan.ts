import type { ThreadRoleMessage } from "../models/modelTypes";
import type { TaskThread } from "../threads/threadTypes";
import type { WorkspaceProject } from "../workspace/workspaceTypes";
import type { ExploreView, PlanView } from "./planTypes";

const requiredPermissions = [
  "read approved workspace files",
  "approval required before file edits",
  "approval required before terminal commands",
];

const fallbackRollback = "Create or reuse a checkpoint before edits, then restore that checkpoint if review rejects the diff.";

export function createOllamaPlanMessages(thread: TaskThread, project: WorkspaceProject): ThreadRoleMessage[] {
  return [
    {
      role: "system",
      content: [
        "You are Delyx Next PlanAgent.",
        "Draft a read-only plan for local project work.",
        "Return only JSON. Do not wrap it in commentary.",
        "Do not claim files were read, commands ran, tests passed, or edits happened.",
        "Use only the approved indexed files listed by the user.",
        "Prefer existing project patterns and avoid new dependencies unless clearly justified.",
        "Risky actions must remain approval-gated.",
      ].join(" "),
    },
    {
      role: "user",
      content: [
        `Project: ${project.name}`,
        `Approved root: ${project.approvedRoots[0] ?? project.path}`,
        `Thread goal:\n${thread.goal}`,
        `Thread messages:\n${threadMessages(thread)}`,
        `Approved indexed files:\n${indexedFiles(project)}`,
        `Known validation commands:\n${projectCommands(project).join("\n")}`,
        "JSON keys: goalUnderstanding, architectureSummary, filesLikelyInvolved, relevantSymbols, steps, risks, testsToRun, permissionsNeeded, rollbackStrategy, unknowns, suggestedNextSteps.",
      ].join("\n\n"),
    },
  ];
}

export function createPlanFromOllamaText(thread: TaskThread, project: WorkspaceProject, text: string): PlanView {
  const payload = parseOllamaPlanText(text);
  const files = approvedFiles(payload.filesLikelyInvolved ?? payload.relevantFiles, project);
  const steps = cleanList(payload.steps).slice(0, 8);
  if (steps.length === 0) {
    throw new Error("Ollama plan JSON did not include any plan steps.");
  }

  const unknowns = cleanList(payload.unknowns).slice(0, 8);
  const risks = withDefault(cleanList(payload.risks), "Model-generated plan still requires approval before risky actions.");
  const explore: ExploreView = {
    architectureSummary: cleanText(payload.architectureSummary) || stackSummary(project),
    projectCommands: projectCommands(project),
    relevantFiles: files,
    relevantSymbols: cleanList(payload.relevantSymbols).slice(0, 12),
    risks,
    suggestedNextSteps: withDefault(cleanList(payload.suggestedNextSteps), "Review the model plan before requesting approval.").slice(0, 6),
    unknowns,
  };

  return {
    decision: "pending",
    explore,
    filesLikelyInvolved: files,
    goalUnderstanding: cleanText(payload.goalUnderstanding) || thread.goal,
    permissionsNeeded: mergePermissions(cleanList(payload.permissionsNeeded)),
    risks: unique([...risks, ...unknowns]),
    rollbackStrategy: cleanText(payload.rollbackStrategy) || fallbackRollback,
    steps,
    testsToRun: withDefault(cleanList(payload.testsToRun), projectCommands(project)[0]).slice(0, 6),
    threadId: thread.id,
  };
}

export function parseOllamaPlanText(text: string): Record<string, unknown> {
  const payload = JSON.parse(extractJsonPayload(text)) as unknown;
  if (!isRecord(payload)) {
    throw new Error("Ollama plan response was not a JSON object.");
  }
  return payload;
}

function threadMessages(thread: TaskThread) {
  return thread.messages
    .slice(-8)
    .map((message) => `${message.role}: ${message.body}`)
    .join("\n") || "No thread messages recorded.";
}

function indexedFiles(project: WorkspaceProject) {
  return project.indexedFiles.slice(0, 80).map((file) => `- ${file}`).join("\n") || "- No indexed files are loaded.";
}

function approvedFiles(value: unknown, project: WorkspaceProject) {
  const fileMap = new Map(project.indexedFiles.map((file) => [normalizePath(file), file]));
  return unique(cleanList(value).map((file) => fileMap.get(normalizePath(file))).filter(isString)).slice(0, 8);
}

function cleanList(value: unknown) {
  if (Array.isArray(value)) {
    return unique(value.map(itemText).filter(isString));
  }
  if (typeof value === "string") {
    return unique(value.split(/\r?\n|;/).map(cleanText).filter(isString));
  }
  return [];
}

function itemText(value: unknown) {
  if (isRecord(value)) {
    return cleanText(value.description ?? value.text ?? value.title ?? value.name ?? value.step);
  }
  return cleanText(value);
}

function cleanText(value: unknown) {
  if (typeof value === "number") {
    return `${value}`;
  }
  return typeof value === "string" ? value.trim().replace(/\s+/g, " ").slice(0, 240) : "";
}

function mergePermissions(items: string[]) {
  return unique([...requiredPermissions, ...items]).slice(0, 8);
}

function withDefault(items: string[], fallback: string) {
  return items.length > 0 ? items : [fallback];
}

function projectCommands(project: WorkspaceProject) {
  const commands: string[] = [];
  if (project.indexedFiles.includes("Cargo.toml")) {
    commands.push("cargo test --workspace");
  }
  if (project.indexedFiles.includes("package.json")) {
    commands.push("npm test");
  }
  return commands.length > 0 ? commands : ["No project test command discovered yet."];
}

function stackSummary(project: WorkspaceProject) {
  if (project.indexedFiles.includes("Cargo.toml")) {
    return "Rust workspace detected from approved project files.";
  }
  if (project.indexedFiles.includes("package.json")) {
    return "TypeScript or JavaScript workspace detected from approved project files.";
  }
  return "No dominant stack detected from approved project files.";
}

function extractJsonPayload(text: string) {
  const fenced = text.match(/```(?:json)?\s*([\s\S]*?)```/i)?.[1];
  if (fenced) {
    return fenced.trim();
  }
  const start = text.indexOf("{");
  const end = text.lastIndexOf("}");
  if (start < 0 || end <= start) {
    throw new Error("Ollama plan response did not contain JSON.");
  }
  return text.slice(start, end + 1);
}

function normalizePath(path: string) {
  return path.replace(/\\/g, "/").replace(/^\.\//, "").toLowerCase();
}

function unique(items: string[]) {
  return Array.from(new Set(items));
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return !!value && typeof value === "object" && !Array.isArray(value);
}

function isString(value: string | undefined): value is string {
  return typeof value === "string" && value.length > 0;
}
