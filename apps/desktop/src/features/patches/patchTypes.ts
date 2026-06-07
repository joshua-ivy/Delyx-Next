export type PatchStatus = "proposed" | "applied" | "restored";
export type DiffLineKind = "context" | "added" | "removed";

export interface DiffLineView {
  kind: DiffLineKind;
  text: string;
}

export interface PatchFileView {
  path: string;
  diff: DiffLineView[];
}

export interface PatchProposalView {
  id: string;
  runId: string;
  approvalId: string;
  status: PatchStatus;
  checkpointId?: string;
  files: PatchFileView[];
}
