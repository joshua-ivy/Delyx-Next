export type ProviderKind = "ollama" | "openai_compatible" | "cli" | "unavailable";
export type ProviderStatus = "ready" | "missing_key" | "not_configured" | "unreachable";
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
