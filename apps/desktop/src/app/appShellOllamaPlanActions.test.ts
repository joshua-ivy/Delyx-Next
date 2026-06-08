import { beforeEach, describe, expect, it, vi } from "vitest";

import { sendOllamaChat, selectedOllamaModel } from "../features/models/ollamaClient";
import { createOllamaPlanMessages, createPlanFromOllamaText } from "../features/plans/ollamaPlan";
import { savePlanOverBridge } from "../features/plans/planClient";
import type { PlanView } from "../features/plans/planTypes";
import { appendThreadMessageOverBridge } from "../features/threads/threadClient";
import { createPlanWithOllama } from "./appShellOllamaPlanActions";

vi.mock("../features/models/ollamaClient", () => ({
  selectedOllamaModel: vi.fn(),
  sendOllamaChat: vi.fn(),
}));
vi.mock("../features/plans/ollamaPlan", () => ({
  createOllamaPlanMessages: vi.fn(),
  createPlanFromOllamaText: vi.fn(),
}));
vi.mock("../features/plans/planClient", () => ({ savePlanOverBridge: vi.fn() }));
vi.mock("../features/threads/threadClient", () => ({ appendThreadMessageOverBridge: vi.fn() }));
vi.mock("./ShellPreferenceController", () => ({ notifyLocalAction: vi.fn() }));

const appendMessage = vi.mocked(appendThreadMessageOverBridge);
const createMessages = vi.mocked(createOllamaPlanMessages);
const createPlan = vi.mocked(createPlanFromOllamaText);
const savePlan = vi.mocked(savePlanOverBridge);
const sendChat = vi.mocked(sendOllamaChat);
const selectedModel = vi.mocked(selectedOllamaModel);

beforeEach(() => {
  vi.clearAllMocks();
  appendMessage.mockResolvedValue(undefined);
  createMessages.mockReturnValue([{ content: "plan", role: "user" }]);
  createPlan.mockReturnValue(plan);
  savePlan.mockResolvedValue({ ...plan, steps: ["Persisted plan step."] });
  selectedModel.mockReturnValue("qwen3-coder:30b");
  sendChat.mockResolvedValue({ model: "qwen3-coder:30b", providerId: "ollama-local", text: "draft" });
});

describe("createPlanWithOllama", () => {
  it("persists drafted plans and renders the saved bridge result", async () => {
    const state = planState();

    await createPlanWithOllama(state);

    expect(savePlan).toHaveBeenCalledWith("project-1", plan);
    const applyPlans = state.setPlans.mock.calls[0][0] as (current: PlanView[]) => PlanView[];
    expect(applyPlans([])).toEqual([{ ...plan, steps: ["Persisted plan step."] }]);
    expect(state.setThreads).toHaveBeenCalled();
  });
});

function planState() {
  return {
    activeProject: project,
    activeRun: run,
    activeThread: thread,
    modelSettings: { providers: [], routes: [], selectedProviderId: "ollama-local" },
    setAgentRuns: vi.fn(),
    setPlans: vi.fn(),
    setThreads: vi.fn(),
    setThreadState: vi.fn(),
    threads: [thread],
  };
}

const plan: PlanView = {
  decision: "pending",
  explore: {
    architectureSummary: "Use the local runtime.",
    projectCommands: ["npm test"],
    relevantFiles: ["src/main.ts"],
    relevantSymbols: [],
    risks: [],
    suggestedNextSteps: ["Review"],
    unknowns: [],
  },
  filesLikelyInvolved: ["src/main.ts"],
  goalUnderstanding: "Draft a real plan.",
  permissionsNeeded: ["edit_file"],
  rollbackStrategy: "Use patch checkpoints.",
  risks: [],
  steps: ["Draft step."],
  testsToRun: ["npm test"],
  threadId: "thread-1",
};

const project = {
  approvalPolicy: "manual",
  approvedRoots: ["C:/repo"],
  git: { branch: "main", isRepo: true, uncommittedChanges: 0 },
  id: "project-1",
  indexedFiles: ["src/main.ts"],
  isolation: { detail: "none", label: "none", mode: "none" as const },
  lastOpenedLabel: "now",
  name: "Repo",
  path: "C:/repo",
  pinned: true,
  rulesFiles: [],
};

const thread = {
  activeRunId: "run-1",
  archived: false,
  createdAt: "2026-06-08T00:00:00.000Z",
  createdLabel: "now",
  goal: "Draft a plan.",
  id: "thread-1",
  messages: [],
  mode: "plan" as const,
  projectId: "project-1",
  runIds: ["run-1"],
  status: "planning" as const,
  title: "Draft plan",
  updatedAt: "2026-06-08T00:00:00.000Z",
};

const run = {
  artifacts: [],
  createdAt: "2026-06-08T00:00:00.000Z",
  events: [],
  evidence: [],
  goal: "Draft a plan.",
  id: "run-1",
  metrics: { approvalCount: 0, artifactCount: 0, commandCount: 0, eventCount: 0, evidenceCount: 0, nodeCount: 0 },
  mode: "plan" as const,
  nodes: [],
  projectId: "project-1",
  status: "running" as const,
  threadId: "thread-1",
  updatedAt: "2026-06-08T00:00:00.000Z",
};
