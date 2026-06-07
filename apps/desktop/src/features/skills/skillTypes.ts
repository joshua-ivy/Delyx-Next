export type SkillTrust = "local" | "third_party";
export type SkillStatus = "inactive" | "active" | "disabled" | "suppressed";

export interface SkillStateView {
  skills: SkillManifestView[];
}

export interface SkillManifestView {
  id: string;
  name: string;
  source: string;
  sourceHash: string;
  trust: SkillTrust;
  status: SkillStatus;
  permissions: SkillPermissionsView;
}

export interface SkillPermissionsView {
  canRunScripts: boolean;
  canEditFiles: boolean;
  canUseNetwork: boolean;
}
