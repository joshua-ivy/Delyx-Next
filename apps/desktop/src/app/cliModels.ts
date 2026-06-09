import type { ExternalAgentAdapterView } from "../features/externalAgents/externalAgentTypes";
import type { ModelProviderView, ModelSelectionKey, ModelSettingsView } from "../features/models/modelTypes";
import { selectCodingModel } from "./modelSelection";

// CLI agents that can answer chat read-only via their subscription (CLI-first).
const CLI_LABELS: Record<string, string> = {
  "claude-code": "Claude Code (CLI)",
  "codex-cli": "Codex CLI",
};

const CLI_DETAIL = "Runs on your machine via the CLI subscription; read-only for chat. Prompts go off-device to the provider.";

/** Add detected, installed CLI agents to the model picker as selectable providers. */
export function mergeCliProviders(
  settings: ModelSettingsView,
  adapters: ExternalAgentAdapterView[],
): ModelSettingsView {
  const cliProviders = adapters
    .filter((adapter) => CLI_LABELS[adapter.id] && adapter.status === "available")
    .map<ModelProviderView>((adapter) => ({
      detail: CLI_DETAIL,
      id: adapter.id,
      kind: "cli",
      label: CLI_LABELS[adapter.id],
      models: [adapter.id],
      requiresSecret: false,
      status: "ready",
    }));
  if (cliProviders.length === 0) {
    return settings;
  }
  const withoutCli = settings.providers.filter((provider) => provider.kind !== "cli");
  return { ...settings, providers: [...withoutCli, ...cliProviders] };
}

/** Route a model selection to the CLI provider or fall back to the Ollama coding route. */
export function selectModelRoute(
  settings: ModelSettingsView,
  adapters: ExternalAgentAdapterView[],
  selection: ModelSelectionKey,
): ModelSettingsView {
  if (adapters.some((adapter) => adapter.id === selection.providerId && CLI_LABELS[adapter.id])) {
    return { ...settings, selectedProviderId: selection.providerId };
  }
  return selectCodingModel(settings, selection);
}

/** The selected CLI adapter id, or undefined when a non-CLI model is selected. */
export function cliAdapterForSelection(settings: ModelSettingsView): string | undefined {
  const provider = settings.providers.find((item) => item.id === settings.selectedProviderId);
  return provider?.kind === "cli" ? provider.id : undefined;
}
