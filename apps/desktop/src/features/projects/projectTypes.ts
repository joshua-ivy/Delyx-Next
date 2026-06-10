// Native project domain, mirroring the Rust `ProjectRecord` (serde camelCase).
// A project is Delyx's durable trust boundary: root path, allowed file scopes,
// and default approval/model/tool/memory policy.

export type ProjectTrustLevel = "local" | "restricted" | "external";

export interface FileScopeView {
  path: string;
  recursive: boolean;
  canRead: boolean;
  canWrite: boolean;
  reason: string;
}

export interface ApprovalPolicyView {
  mode: string;
  largeFileBytes: number;
  folderFileCount: number;
  requireApprovalOutsideScope: boolean;
}

export interface ModelPermissionsView {
  allowLocal: boolean;
  allowCli: boolean;
  allowCloud: boolean;
}

export interface ToolPermissionsView {
  allowFileWrite: boolean;
  allowTerminal: boolean;
  allowMcpTools: boolean;
}

export interface MemoryScopeView {
  mode: string;
}

export interface ProjectView {
  id: string;
  name: string;
  rootPath: string;
  trustLevel: ProjectTrustLevel;
  allowedFileScopes: FileScopeView[];
  approvalPolicy: ApprovalPolicyView;
  modelPermissions: ModelPermissionsView;
  toolPermissions: ToolPermissionsView;
  memoryScope: MemoryScopeView;
  createdAt: string;
  updatedAt: string;
}

export interface ProjectSaveRequest {
  id?: string;
  name: string;
  rootPath: string;
  trustLevel?: ProjectTrustLevel;
  allowedFileScopes?: FileScopeView[];
  approvalPolicy?: ApprovalPolicyView;
  modelPermissions?: ModelPermissionsView;
  toolPermissions?: ToolPermissionsView;
  memoryScope?: MemoryScopeView;
}
