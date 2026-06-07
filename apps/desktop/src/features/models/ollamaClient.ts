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

export async function refreshOllamaSettings(settings: ModelSettingsView) {
  try {
    const response = await fetchWithTimeout(`${endpoint}/api/tags`, { method: "GET" }, 2500);
    if (!response.ok) {
      return withOllamaProvider(settings, [], "unreachable", `Ollama returned HTTP ${response.status}.`);
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
  const response = await fetchWithTimeout(`${endpoint}/api/chat`, {
    body: JSON.stringify({ model, messages, stream: false }),
    headers: { "Content-Type": "application/json" },
    method: "POST",
  }, 120000);
  if (!response.ok) {
    throw new Error(`Ollama chat failed with HTTP ${response.status}.`);
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
  const routes = status === "ready" ? upsertOllamaRoutes(settings.routes, models[0]) : settings.routes;
  return { ...settings, providers, routes, selectedProviderId };
}

function upsertOllamaRoutes(routes: ModelSettingsView["routes"], modelId: string | undefined) {
  if (!modelId) {
    return routes;
  }
  const keep = routes.filter((route) => !(route.providerId === ollamaId && ["answer", "coding"].includes(route.role)));
  return [
    { modelId, providerId: ollamaId, role: "coding" as const, saved: true },
    { modelId, providerId: ollamaId, role: "answer" as const, saved: true },
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
