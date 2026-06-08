export type PatchStatus = "proposed" | "applied" | "restored";
export type DiffLineKind = "context" | "added" | "removed";
export type PatchFileChangeKind = "create" | "modify";

export interface DiffLineView {
  kind: DiffLineKind;
  text: string;
}

export interface PatchFileView {
  path: string;
  before: string;
  after: string;
  changeKind: PatchFileChangeKind;
  diff: DiffLineView[];
}

export interface PatchCheckpointFileView {
  path: string;
  contents?: string;
}

export interface PatchProposalView {
  id: string;
  runId: string;
  approvalId: string;
  status: PatchStatus;
  checkpointId?: string;
  restoreApprovalId?: string;
  checkpointFiles: PatchCheckpointFileView[];
  files: PatchFileView[];
}
