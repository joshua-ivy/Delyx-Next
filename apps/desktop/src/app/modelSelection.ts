import type { ModelSettingsView } from "../features/models/modelTypes";

export function selectOllamaCodingModel(settings: ModelSettingsView, modelId: string): ModelSettingsView {
  return {
    ...settings,
    routes: [
      { modelId, providerId: "ollama-local", role: "coding", saved: false },
      ...settings.routes.filter((route) => !(route.providerId === "ollama-local" && route.role === "coding")),
    ],
    selectedProviderId: "ollama-local",
  };
}
