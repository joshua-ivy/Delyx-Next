import type { ModelSettingsView } from "./modelTypes";

export const currentModelSettings: ModelSettingsView = {
  selectedProviderId: "ollama-local",
  providers: [
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
      detail: "Not wired yet. Use Ollama for live local model calls.",
      id: "openai-compatible",
      kind: "unavailable",
      label: "OpenAI-compatible",
      models: [],
      requiresSecret: false,
      status: "not_configured",
    },
  ],
  routes: [],
};
