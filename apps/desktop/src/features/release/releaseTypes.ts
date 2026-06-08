export type ReleaseSmokeStatus = "failed" | "passed" | "not_loaded";
export type SigningStatus = "unsigned_dev" | "signed" | "missing_certificate";

export interface ReleaseStateView {
  platform: string;
  bundleTarget: string;
  installer: string;
  smokeStatus: ReleaseSmokeStatus;
  smoke: ReleaseSmokeStateView;
  signing: SigningStateView;
  supportBundle: SupportBundleStateView;
  updateMetadata: UpdateMetadataStateView;
}

export interface ReleaseSmokeStateView {
  status: ReleaseSmokeStatus;
  detail: string;
  installerPath?: string;
  command?: string;
  capturedAt?: string;
}

export interface SigningStateView {
  status: SigningStatus;
  message: string;
}

export interface SupportBundleStateView {
  exportStatus: "available" | "not_exported";
  secretPolicy: string;
}

export interface UpdateMetadataStateView {
  status: "placeholder" | "published";
  channel: string;
}
