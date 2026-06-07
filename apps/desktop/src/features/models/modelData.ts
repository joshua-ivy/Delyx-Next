import type { ModelSettingsView } from "./modelTypes";

export const currentModelSettings: ModelSettingsView = {
  selectedProviderId: "mock-local",
  providers: [
    {
      detail: "Deterministic local provider for offline development.",
      id: "mock-local",
      kind: "mock",
      label: "Mock provider",
      models: ["delyx-mock-coder", "delyx-mock-reasoner"],
      requiresSecret: false,
      status: "ready",
    },
    {
      detail: "No local endpoint configured yet.",
      id: "ollama-local",
      kind: "ollama",
      label: "Ollama",
      models: [],
      requiresSecret: false,
      status: "not_configured",
    },
    {
      detail: "API key missing. Secrets must stay outside the repo.",
      id: "openai-compatible",
      kind: "openai_compatible",
      label: "OpenAI-compatible",
      models: [],
      requiresSecret: true,
      status: "missing_key",
    },
  ],
  routes: [
    { modelId: "delyx-mock-reasoner", providerId: "mock-local", role: "answer", saved: true },
    { modelId: "delyx-mock-coder", providerId: "mock-local", role: "coding", saved: true },
    { modelId: "delyx-mock-reasoner", providerId: "mock-local", role: "helper", saved: true },
  ],
};
