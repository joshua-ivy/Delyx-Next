import { describe, expect, it } from "vitest";

import type { ModelSettingsView } from "../features/models/modelTypes";
import { selectCodingModel } from "./modelSelection";

function settings(): ModelSettingsView {
  return {
    providers: [
      { detail: "", id: "delyx-local", kind: "delyx_local", label: "Delyx Local", models: ["shared-coder"], requiresSecret: false, status: "ready" },
      { detail: "", id: "ollama-local", kind: "ollama", label: "Ollama", models: ["shared-coder"], requiresSecret: false, status: "ready" },
    ],
    routes: [],
    selectedProviderId: "ollama-local",
  };
}

describe("selectCodingModel", () => {
  it("disambiguates a model name shared across providers by the chosen provider", () => {
    const picked = selectCodingModel(settings(), { modelId: "shared-coder", providerId: "delyx-local" });
    expect(picked.selectedProviderId).toBe("delyx-local");
    expect(picked.routes.find((route) => route.role === "coding")).toEqual({
      modelId: "shared-coder",
      providerId: "delyx-local",
      role: "coding",
      saved: false,
    });
  });

  it("ignores a selection whose provider is not ready or lacks the model", () => {
    const notReady = selectCodingModel(
      { ...settings(), providers: settings().providers.map((p) => ({ ...p, status: "loading" as const })) },
      { modelId: "shared-coder", providerId: "delyx-local" },
    );
    expect(notReady.selectedProviderId).toBe("ollama-local");
    const missingModel = selectCodingModel(settings(), { modelId: "nope", providerId: "delyx-local" });
    expect(missingModel.selectedProviderId).toBe("ollama-local");
  });
});
