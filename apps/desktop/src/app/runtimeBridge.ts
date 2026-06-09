import { invoke } from "@tauri-apps/api/core";
import type { ModelProviderView, ModelSettingsView, ProviderKind, ProviderStatus } from "../features/models/modelTypes";

export interface RuntimeBridgeState {
  label: string;
  mode: "tauri" | "web";
  status?: RuntimeStatusView;
}

export const webRuntimeBridge: RuntimeBridgeState = {
  label: "Web preview / Rust bridge unavailable",
  mode: "web",
};

export interface RuntimeStatusView {
  appIdentifier: string;
  appName: string;
  codingRoute?: {
    modelId: string;
    providerId: string;
  };
  desktopShell: DesktopShellStatusView;
  milestone: string;
  providers: Array<{
    id: string;
    kind: string;
    label: string;
    message: string;
    models: string[];
    status: string;
    version?: string;
  }>;
}

export interface DesktopShellStatusView {
  mainWindowLabel: string;
  nativeMenuPolicy: string;
  reopenBehavior: string;
  signingPolicy: string;
  startupBehavior: string;
}

export async function loadRuntimeBridgeState(): Promise<RuntimeBridgeState> {
  try {
    const status = await invoke<RuntimeStatusView>("runtime_status");
    return { label: `Rust bridge / ${status.milestone}`, mode: "tauri", status };
  } catch {
    return { label: "Web preview / Rust bridge unavailable", mode: "web" };
  }
}

export function modelSettingsFromRuntimeStatus(
  settings: ModelSettingsView,
  status: RuntimeStatusView,
): ModelSettingsView {
  const providers = status.providers.map(runtimeProviderView);
  const codingRoute = status.codingRoute;
  const routes = codingRoute ? [{
    modelId: codingRoute.modelId,
    providerId: codingRoute.providerId,
    role: "coding" as const,
    saved: true,
  }] : [];
  const selectedProviderId = codingRoute?.providerId
    ?? (providers.some((provider) => provider.id === settings.selectedProviderId)
      ? settings.selectedProviderId
      : providers[0]?.id ?? settings.selectedProviderId);
  return { ...settings, providers, routes, selectedProviderId };
}

function runtimeProviderView(provider: RuntimeStatusView["providers"][number]): ModelProviderView {
  const unsupportedOpenAi = provider.kind === "openai_compatible";
  return {
    detail: unsupportedOpenAi
      ? "OpenAI-compatible calls are not wired yet. Use Delyx Local or Ollama for live local calls."
      : provider.message,
    id: provider.id,
    kind: unsupportedOpenAi ? "unavailable" : providerKind(provider.kind),
    label: provider.label,
    models: provider.models,
    requiresSecret: false,
    status: unsupportedOpenAi ? "not_configured" : providerStatus(provider.status),
    version: provider.version,
  };
}

function providerKind(kind: string): ProviderKind {
  if (kind === "delyx_local" || kind === "ollama" || kind === "openai_compatible") {
    return kind;
  }
  return "unavailable";
}

function providerStatus(status: string): ProviderStatus {
  const known = ["failed", "loading", "missing_key", "model_missing", "not_configured", "ready", "unreachable"];
  return known.includes(status) ? (status as ProviderStatus) : "not_configured";
}
