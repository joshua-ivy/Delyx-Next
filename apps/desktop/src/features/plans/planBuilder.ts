import type { TaskThread } from "../threads/threadTypes";
import type { WorkspaceProject } from "../workspace/workspaceTypes";
import type { ExploreView, PlanView } from "./planTypes";

export function createPlanFromThread(thread: TaskThread, project: WorkspaceProject): PlanView {
  const explore = createExploreView(thread.goal, project);

  return {
    decision: "pending",
    explore,
    filesLikelyInvolved: explore.relevantFiles,
    goalUnderstanding: thread.goal,
    permissionsNeeded: [
      "read approved workspace files",
      "approval required before file edits",
      "approval required before terminal commands",
    ],
    risks: [...explore.risks, ...explore.unknowns],
    rollbackStrategy: "Create or reuse a checkpoint before edits, then restore that checkpoint if review rejects the diff.",
    steps: [
      explore.relevantFiles.length > 0
        ? "Review the relevant approved files listed by Explore."
        : "Review the approved workspace index for a better file target.",
      "Draft a narrow change proposal without editing files.",
      "Request explicit approval before any file write or command.",
    ],
    testsToRun: testsFromProject(project),
    threadId: thread.id,
  };
}

function createExploreView(goal: string, project: WorkspaceProject): ExploreView {
  const terms = goal.toLowerCase().split(/[^a-z0-9]+/).filter((term) => term.length >= 4);
  const relevantFiles = project.indexedFiles
    .filter((file) => terms.some((term) => file.toLowerCase().includes(term)))
    .slice(0, 6);

  return {
    architectureSummary: stackSummary(project),
    projectCommands: testsFromProject(project),
    relevantFiles,
    relevantSymbols: [],
    risks: ["Explore and Plan modes are read-only; edits require a later approval."],
    suggestedNextSteps: relevantFiles.length > 0
      ? ["Create a plan from the relevant files before requesting edits."]
      : ["Refine the goal or approve a narrower file search."],
    unknowns: relevantFiles.length === 0 ? ["No matching approved files found."] : [],
  };
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

function testsFromProject(project: WorkspaceProject) {
  const commands: string[] = [];
  if (project.indexedFiles.includes("Cargo.toml")) {
    commands.push("cargo test --workspace");
  }
  if (project.indexedFiles.includes("package.json")) {
    commands.push("npm test");
  }
  return commands.length > 0 ? commands : ["No project test command discovered yet."];
}
