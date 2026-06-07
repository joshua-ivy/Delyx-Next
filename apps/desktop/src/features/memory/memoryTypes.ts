export type MemoryScope = "project" | "user";
export type MemoryCandidateStatus = "pending" | "promoted" | "suppressed";

export interface MemoryStateView {
  candidates: MemoryCandidateView[];
  records: MemoryRecordView[];
}

export interface MemoryCandidateView {
  id: string;
  scope: MemoryScope;
  key: string;
  value: string;
  sourceRunId: string;
  sourceThreadId: string;
  status: MemoryCandidateStatus;
}

export interface MemoryRecordView {
  id: string;
  scope: MemoryScope;
  key: string;
  value: string;
  sourceRunId: string;
  sourceThreadId: string;
  supersedes?: string;
  suppressed: boolean;
}
