export type TestStatus = "passed" | "failed";

export interface TestArtifactView {
  id: string;
  runId: string;
  approvalId: string;
  command: string;
  workingDirectory: string;
  exitCode: number | null;
  durationMs: number;
  stdout: string;
  stderr: string;
  status: TestStatus;
  failureSummary?: string;
  createdAt: string;
}
