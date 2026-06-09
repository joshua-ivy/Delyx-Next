import { invoke } from "@tauri-apps/api/core";
import type { ModelSettingsView, RoleRouteView, ThreadRoleMessage } from "./modelTypes";

export interface ModelChatResult {
  model: string;
  providerId: string;
  text: string;
}

/**
 * Send a chat request to the selected non-CLI coding model (Delyx Local or
 * Ollama) through the provider-aware `model_chat` command. CLI providers are
 * handled separately by the cli_chat path, so they are excluded here.
 */
export async function sendModelChat(
  settings: ModelSettingsView,
  messages: ThreadRoleMessage[],
): Promise<ModelChatResult> {
  const route = selectedCodingRoute(settings);
  if (!route) {
    throw new Error("No ready model is selected. Import a Delyx Local model or select an available provider.");
  }
  if (!hasTauriRuntime()) {
    throw new Error("Model chat requires the Delyx desktop runtime.");
  }
  return invoke<ModelChatResult>("model_chat", {
    request: { messages, model: route.modelId, providerId: route.providerId },
  });
}

export function selectedCodingRoute(settings: ModelSettingsView): RoleRouteView | undefined {
  const route = settings.routes.find((item) => item.role === "coding");
  if (route && providerReadyForModel(settings, route.providerId, route.modelId)) {
    return route;
  }
  const provider = settings.providers.find(
    (item) => item.kind !== "cli" && item.status === "ready" && item.models.length > 0,
  );
  if (!provider) {
    return undefined;
  }
  return { modelId: provider.models[0], providerId: provider.id, role: "coding", saved: false };
}

function providerReadyForModel(settings: ModelSettingsView, providerId: string, modelId: string) {
  const provider = settings.providers.find((item) => item.id === providerId);
  return Boolean(
    provider && provider.kind !== "cli" && provider.status === "ready" && provider.models.includes(modelId),
  );
}

function hasTauriRuntime() {
  return Boolean((globalThis as Record<string, unknown>).__TAURI_INTERNALS__);
}
