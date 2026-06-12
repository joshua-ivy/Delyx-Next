import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { ModelSettingsView, RoleRouteView, ThreadRoleMessage } from "./modelTypes";

export interface ModelChatResult {
  model: string;
  providerId: string;
  text: string;
}

export interface ModelStreamResult extends ModelChatResult {
  cancelled: boolean;
}

interface ModelStreamEvent {
  requestId: string;
  kind: "token" | "done" | "cancelled";
  text: string;
}

let activeStreamRequestId: string | undefined;

/** Stop the in-flight local-model stream, keeping the partial reply. */
export async function cancelActiveModelStream(): Promise<void> {
  if (!activeStreamRequestId || !hasTauriRuntime()) {
    return;
  }
  try {
    await invoke("model_chat_cancel", { requestId: activeStreamRequestId });
  } catch {
    // Cancel is best-effort; the stream finishing first is fine.
  }
}

interface ToolLoopEvent {
  requestId: string;
  kind: "tool" | "tool_result" | "tool_warning";
  summary: string;
}

export interface ToolLoopHandlers {
  onToken: (accumulated: string, delta: string) => void;
  /** Fires when the model calls a read-only project tool. */
  onTool?: (summary: string) => void;
  /** Fires when a tool result contained instruction-shaped (possible prompt-injection) content. */
  onToolWarning?: (summary: string) => void;
}

/**
 * Agentic chat for Delyx Local: the model may inspect the project with
 * read-only tools (narrated via `onTool`) before its final answer streams
 * through `onToken`. Other providers fall back to the non-streaming call.
 */
export async function sendModelChatTools(
  settings: ModelSettingsView,
  messages: ThreadRoleMessage[],
  projectRoot: string,
  handlers: ToolLoopHandlers,
): Promise<ModelStreamResult> {
  const route = selectedCodingRoute(settings);
  if (!route) {
    throw new Error("No ready model is selected. Import a Delyx Local model or select an available provider.");
  }
  if (route.providerId !== "delyx-local" || !hasTauriRuntime()) {
    const result = await sendModelChat(settings, messages);
    handlers.onToken(result.text, result.text);
    return { ...result, cancelled: false };
  }
  const requestId = `tools-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`;
  activeStreamRequestId = requestId;
  let accumulated = "";
  let cancelled = false;
  const unlistenToken = await listen<ModelStreamEvent>("model-stream", (event) => {
    if (event.payload.requestId !== requestId) {
      return;
    }
    if (event.payload.kind === "token" && event.payload.text) {
      accumulated += event.payload.text;
      handlers.onToken(accumulated, event.payload.text);
    }
    if (event.payload.kind === "cancelled") {
      cancelled = true;
    }
  });
  const unlistenTool = await listen<ToolLoopEvent>("tool-loop", (event) => {
    if (event.payload.requestId !== requestId) {
      return;
    }
    if (event.payload.kind === "tool") {
      handlers.onTool?.(event.payload.summary);
    }
    if (event.payload.kind === "tool_warning") {
      handlers.onToolWarning?.(event.payload.summary);
    }
  });
  try {
    const result = await invoke<ModelChatResult>("model_chat_tools", {
      request: { messages, model: route.modelId, providerId: route.providerId, requestId, projectRoot },
    });
    return { ...result, cancelled };
  } finally {
    unlistenToken();
    unlistenTool();
    if (activeStreamRequestId === requestId) {
      activeStreamRequestId = undefined;
    }
  }
}

/**
 * Streamed chat for the Delyx Local provider: `onToken` fires with the
 * accumulated text as deltas arrive. Other providers transparently fall back to
 * the non-streaming call (one `onToken` with the full text).
 */
export async function sendModelChatStream(
  settings: ModelSettingsView,
  messages: ThreadRoleMessage[],
  onToken: (accumulated: string, delta: string) => void,
): Promise<ModelStreamResult> {
  const route = selectedCodingRoute(settings);
  if (!route) {
    throw new Error("No ready model is selected. Import a Delyx Local model or select an available provider.");
  }
  if (route.providerId !== "delyx-local" || !hasTauriRuntime()) {
    const result = await sendModelChat(settings, messages);
    onToken(result.text, result.text);
    return { ...result, cancelled: false };
  }
  const requestId = `stream-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`;
  activeStreamRequestId = requestId;
  let accumulated = "";
  let cancelled = false;
  const unlisten = await listen<ModelStreamEvent>("model-stream", (event) => {
    if (event.payload.requestId !== requestId) {
      return;
    }
    if (event.payload.kind === "token" && event.payload.text) {
      accumulated += event.payload.text;
      onToken(accumulated, event.payload.text);
    }
    if (event.payload.kind === "cancelled") {
      cancelled = true;
    }
  });
  try {
    const result = await invoke<ModelChatResult>("model_chat_stream", {
      request: { messages, model: route.modelId, providerId: route.providerId, requestId },
    });
    return { ...result, cancelled };
  } finally {
    unlisten();
    if (activeStreamRequestId === requestId) {
      activeStreamRequestId = undefined;
    }
  }
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
