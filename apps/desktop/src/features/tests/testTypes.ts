export type TestStatus = "passed" | "failed" | "not_run";

export interface ParsedFailure {
  id: string;
  message: string;
  filePath?: string;
  line?: number;
  assertion?: string;
}

export interface TestRunArtifact {
  id: string;
  runId: string;
  command: string;
  cwd: string;
  exitCode: number | null;
  durationMs: number;
  stdout: string;
  stderr: string;
  parsedFailures?: ParsedFailure[];
  startedAt: string;
  completedAt: string;
  approvalId?: string;
  outputTruncated?: boolean;
  execEvents?: CommandExecEvent[];
}

export interface TestArtifactView extends TestRunArtifact {
  status?: TestStatus;
  failureSummary?: string;
}

export interface CommandExecEvent {
  kind: "started" | "stdout" | "stderr" | "completed" | "failed";
  message: string;
  timestampMs: number;
}
