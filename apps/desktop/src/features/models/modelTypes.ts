export type ProviderKind = "delyx_local" | "ollama" | "openai_compatible" | "cli" | "unavailable";
export type ProviderStatus = "ready" | "loading" | "missing_key" | "model_missing" | "not_configured" | "unreachable" | "failed";

export interface ModelSelectionKey {
  providerId: string;
  modelId: string;
}
export type ModelRole = "answer" | "helper" | "deepResearch" | "maxReasoning" | "coding" | "embedding" | "scoring";

export interface ModelProviderView {
  id: string;
  kind: ProviderKind;
  label: string;
  status: ProviderStatus;
  detail: string;
  models: string[];
  requiresSecret: boolean;
  version?: string;
}

export interface RoleRouteView {
  role: ModelRole;
  providerId: string;
  modelId: string;
  saved: boolean;
}

export interface ModelSettingsView {
  selectedProviderId: string;
  providers: ModelProviderView[];
  routes: RoleRouteView[];
}

export interface ThreadRoleMessage {
  role: "assistant" | "system" | "user";
  content: string;
}
