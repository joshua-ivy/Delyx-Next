import { describe, expect, it } from "vitest";

import type { ExternalAgentAdapterView } from "../features/externalAgents/externalAgentTypes";
import type { ModelSettingsView } from "../features/models/modelTypes";
import { cliAdapterForSelection, mergeCliProviders, selectModelRoute } from "./cliModels";

function baseSettings(): ModelSettingsView {
  return {
    providers: [
      { detail: "local", id: "ollama-local", kind: "ollama", label: "Ollama", models: ["local-test-coder"], requiresSecret: false, status: "ready" },
    ],
    routes: [],
    selectedProviderId: "ollama-local",
  };
}

function adapters(): ExternalAgentAdapterView[] {
  return [
    { detail: "found", id: "claude-code", kind: "claude_code", label: "Claude Code", status: "available" },
    { detail: "missing", id: "codex-cli", kind: "codex_cli", label: "Codex CLI", status: "missing" },
    { detail: "n/a", id: "generic-terminal", kind: "generic_terminal", label: "Generic", status: "available" },
  ];
}

describe("cliModels", () => {
  it("adds only available agent CLIs as selectable cli providers", () => {
    const merged = mergeCliProviders(baseSettings(), adapters());
    const ids = merged.providers.map((provider) => provider.id);
    expect(ids).toContain("claude-code");
    // Codex is missing and the generic terminal is not a chat CLI.
    expect(ids).not.toContain("codex-cli");
    expect(ids).not.toContain("generic-terminal");
    const claude = merged.providers.find((provider) => provider.id === "claude-code");
    expect(claude?.kind).toBe("cli");
    expect(claude?.status).toBe("ready");
  });

  it("selecting a CLI model points the selection at that provider", () => {
    const merged = mergeCliProviders(baseSettings(), adapters());
    const selected = selectModelRoute(merged, adapters(), { modelId: "claude-code", providerId: "claude-code" });
    expect(selected.selectedProviderId).toBe("claude-code");
    expect(cliAdapterForSelection(selected)).toBe("claude-code");
  });

  it("selecting a non-CLI model leaves CLI routing off", () => {
    const merged = mergeCliProviders(baseSettings(), adapters());
    const selected = selectModelRoute(merged, adapters(), { modelId: "local-test-coder", providerId: "ollama-local" });
    expect(cliAdapterForSelection(selected)).toBeUndefined();
  });
});
