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
  return {
    detail: provider.message,
    id: provider.id,
    kind: providerKind(provider.kind),
    label: provider.label,
    models: provider.models,
    requiresSecret: provider.kind === "openai_compatible",
    status: providerStatus(provider.status),
  };
}

function providerKind(kind: string): ProviderKind {
  if (kind === "ollama" || kind === "openai_compatible") {
    return kind;
  }
  return "unavailable";
}

function providerStatus(status: string): ProviderStatus {
  return status === "missing_key" || status === "ready" || status === "unreachable"
    ? status
    : "not_configured";
}
