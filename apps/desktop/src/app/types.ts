export type AgentMode =
  | "explore"
  | "plan"
  | "build"
  | "review"
  | "test"
  | "research"
  | "automation";

export type TaskStatus =
  | "idle"
  | "exploring"
  | "planning"
  | "waiting_for_approval"
  | "building"
  | "testing"
  | "reviewing"
  | "blocked"
  | "failed"
  | "done";

export type RiskLevel = "low" | "medium" | "high" | "dangerous";
export type ReviewTab = "diff" | "tests" | "approvals" | "evidence";

export interface ProjectSummary {
  id: string;
  name: string;
  path: string;
  branch: string;
  approvedRoots: string[];
  activeThreads: number;
  modelProfile: string;
  lastRunStatus: TaskStatus;
}

export interface ThreadSummary {
  id: string;
  projectId: string;
  title: string;
  goal: string;
  status: TaskStatus;
  mode: AgentMode;
  branch?: string;
  worktreePath?: string;
  changedFilesCount: number;
  pendingApprovalsCount: number;
  lastRunStatus?: string;
  updatedAt: string;
}

export interface PlanViewModel {
  goal: string;
  understanding: string;
  files: string[];
  steps: string[];
  risks: string[];
  tests: string[];
  permissions: string[];
}

export interface TimelineItem {
  id: string;
  label: string;
  status: "done" | "active" | "waiting" | "failed";
  detail: string;
}

export interface ApprovalViewModel {
  id: string;
  action: string;
  risk: RiskLevel;
  reason: string;
  scope: string;
  expectedResult: string;
  status: "pending" | "approved" | "denied" | "expired";
  expiresAt: string;
}

export interface DiffFile {
  path: string;
  additions: number;
  deletions: number;
  patch: string[];
}

export interface TestRunViewModel {
  id: string;
  command: string;
  cwd: string;
  exitCode: number | null;
  durationMs: number;
  status: "passed" | "failed" | "not_run";
  output: string[];
  approvalId?: string;
}

export interface EvidenceViewModel {
  id: string;
  title: string;
  sourceKind: "local_file" | "test" | "diff" | "terminal" | "external_agent";
  relationship: string;
  detail: string;
}

export interface TerminalBlockViewModel {
  id: string;
  label: string;
  status: "idle" | "running" | "done" | "failed";
  lines: string[];
}

export interface ReviewViewModel {
  changedFiles: string[];
  decision: "pending" | "accept" | "revise" | "reject";
  findings: {
    id: string;
    priority: "P0" | "P1" | "P2" | "P3";
    title: string;
    detail: string;
    filePath?: string;
  }[];
}

export interface BlockerViewModel {
  id: string;
  reason: string;
  severity: "info" | "warning" | "error";
  nextStep: string;
}

export interface ActiveTaskViewModel {
  approvals: ApprovalViewModel[];
  blockers: BlockerViewModel[];
  evidence: EvidenceViewModel[];
  plan?: PlanViewModel;
  review: ReviewViewModel;
  terminal: TerminalBlockViewModel[];
  tests: TestRunViewModel[];
  thread: ThreadSummary;
  timeline: TimelineItem[];
}

export interface WorkbenchData {
  projects: ProjectSummary[];
  threads: ThreadSummary[];
  plan: PlanViewModel;
  timeline: TimelineItem[];
  approvals: ApprovalViewModel[];
  diffFiles: DiffFile[];
  tests: TestRunViewModel[];
  evidence: EvidenceViewModel[];
  terminal: TerminalBlockViewModel[];
}
