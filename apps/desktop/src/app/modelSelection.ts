import type { ModelSelectionKey, ModelSettingsView } from "../features/models/modelTypes";

/**
 * Provider-aware coding-route selection. Disambiguates models by (provider, model)
 * so a model name shared by two providers (e.g. delyx-local and ollama-local)
 * selects the chosen provider instead of always defaulting to one.
 */
export function selectCodingModel(
  settings: ModelSettingsView,
  selection: ModelSelectionKey,
): ModelSettingsView {
  const provider = settings.providers.find((item) => item.id === selection.providerId);
  if (!provider || provider.status !== "ready" || !provider.models.includes(selection.modelId)) {
    return settings;
  }
  return {
    ...settings,
    routes: [
      { modelId: selection.modelId, providerId: selection.providerId, role: "coding", saved: false },
      ...settings.routes.filter((route) => route.role !== "coding"),
    ],
    selectedProviderId: selection.providerId,
  };
}
