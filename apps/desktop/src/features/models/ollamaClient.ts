import { invoke } from "@tauri-apps/api/core";
import type { ModelProviderView, ModelSettingsView, ThreadRoleMessage } from "./modelTypes";

const endpoint = "http://127.0.0.1:11434";
const ollamaId = "ollama-local";

interface OllamaTag {
  name?: unknown;
  model?: unknown;
}

interface OllamaTagsResponse {
  models?: OllamaTag[];
}

interface OllamaChatResponse {
  message?: {
    content?: unknown;
  };
  response?: unknown;
}

interface OllamaChatResult {
  model: string;
  providerId: string;
  text: string;
}

export async function refreshOllamaSettings(settings: ModelSettingsView) {
  try {
    const response = await fetchWithTimeout(`${endpoint}/api/tags`, { method: "GET" }, 2500);
    if (!response.ok) {
      return withOllamaProvider(settings, [], "unreachable", `Ollama returned HTTP ${response.status}${await responseDetail(response)}.`);
    }
    const models = modelNames(await response.json() as OllamaTagsResponse);
    const status = models.length > 0 ? "ready" : "not_configured";
    const detail = models.length > 0 ? `${models.length} local model(s) available.` : "Ollama is running, but no local models are installed.";
    return withOllamaProvider(settings, models, status, detail);
  } catch {
    return withOllamaProvider(settings, [], "unreachable", "Ollama is not reachable at 127.0.0.1:11434.");
  }
}

export async function sendOllamaChat(settings: ModelSettingsView, messages: ThreadRoleMessage[]) {
  const model = selectedOllamaModel(settings);
  if (!model) {
    throw new Error("Ollama is not ready. Start Ollama and pull a model, then send again.");
  }
  if (hasTauriRuntime()) {
    return sendOllamaChatViaRuntime(model, messages);
  }
  return sendOllamaChatOverHttp(model, messages);
}

async function sendOllamaChatViaRuntime(model: string, messages: ThreadRoleMessage[]): Promise<OllamaChatResult> {
  try {
    return await invoke<OllamaChatResult>("ollama_chat", { model, messages });
  } catch (error) {
    throw new Error(ollamaBridgeError(error));
  }
}

async function sendOllamaChatOverHttp(model: string, messages: ThreadRoleMessage[]): Promise<OllamaChatResult> {
  const response = await fetchWithTimeout(`${endpoint}/api/chat`, {
    body: JSON.stringify({ model, messages, stream: false }),
    headers: { "Content-Type": "application/json" },
    method: "POST",
  }, 120000);
  if (!response.ok) {
    throw new Error(`Ollama chat failed with HTTP ${response.status}${await responseDetail(response)}.`);
  }
  const text = chatText(await response.json() as OllamaChatResponse);
  if (!text) {
    throw new Error("Ollama returned an empty response.");
  }
  return { model, providerId: ollamaId, text };
}

export function selectedOllamaModel(settings: ModelSettingsView) {
  const provider = settings.providers.find((item) => item.id === ollamaId && item.status === "ready");
  if (!provider || provider.models.length === 0) {
    return undefined;
  }
  return settings.routes.find((route) => route.providerId === ollamaId && route.role === "coding")?.modelId
    ?? provider.models[0];
}

function withOllamaProvider(
  settings: ModelSettingsView,
  models: string[],
  status: ModelProviderView["status"],
  detail: string,
): ModelSettingsView {
  const providers = settings.providers.map((provider) => (
    provider.id === ollamaId ? { ...provider, detail, models, status } : provider
  ));
  const selectedProviderId = status === "ready" ? ollamaId : settings.selectedProviderId;
  const routes = status === "ready" ? upsertOllamaRoute(settings.routes, models[0]) : settings.routes.filter((route) => route.providerId !== ollamaId);
  return { ...settings, providers, routes, selectedProviderId };
}

function upsertOllamaRoute(routes: ModelSettingsView["routes"], modelId: string | undefined) {
  if (!modelId) {
    return routes;
  }
  const keep = routes.filter((route) => !(route.providerId === ollamaId && route.role === "coding"));
  return [
    { modelId, providerId: ollamaId, role: "coding" as const, saved: true },
    ...keep,
  ];
}

function modelNames(payload: OllamaTagsResponse) {
  return (payload.models ?? [])
    .map((item) => (typeof item.name === "string" ? item.name : typeof item.model === "string" ? item.model : ""))
    .filter(Boolean);
}

function chatText(payload: OllamaChatResponse) {
  if (typeof payload.message?.content === "string") {
    return payload.message.content.trim();
  }
  return typeof payload.response === "string" ? payload.response.trim() : "";
}

function fetchWithTimeout(input: RequestInfo | URL, init: RequestInit, timeoutMs: number) {
  const controller = new AbortController();
  const timer = window.setTimeout(() => controller.abort(), timeoutMs);
  return fetch(input, { ...init, signal: controller.signal }).finally(() => window.clearTimeout(timer));
}

async function responseDetail(response: Response) {
  const text = (await response.text()).trim();
  return text ? `: ${text.slice(0, 180)}` : "";
}

function hasTauriRuntime() {
  return Boolean((globalThis as Record<string, unknown>).__TAURI_INTERNALS__);
}

function ollamaBridgeError(error: unknown) {
  if (typeof error === "string" && error.trim()) {
    return error;
  }
  if (error instanceof Error && error.message.trim()) {
    return error.message;
  }
  return "Ollama runtime bridge failed.";
}
