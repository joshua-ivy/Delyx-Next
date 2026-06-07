import type { ModelProviderView, ModelSettingsView } from "../features/models/modelTypes";
import { escapeHtml } from "./html";

export function modelStatusChip(settings: ModelSettingsView) {
  const selected = selectedProvider(settings);
  return `<span class="chip"><span class="k">model</span><b>${escapeHtml(selected.label)}</b> ${statusLabel(selected)}</span>`;
}

export function emptyModelSettingsBlock() {
  return `<div class="dfile model-settings">
        <div class="dh"><span class="fn">Model routing</span><span class="dst">empty</span></div>
        <div class="dc">
          <div class="dr"><span class="g">-</span><span class="x">No provider settings have been loaded.</span></div>
        </div>
      </div>`;
}

export function modelSettingsBlock(settings: ModelSettingsView) {
  const selected = selectedProvider(settings);
  return `<div class="dfile model-settings">
        <div class="dh"><span class="fn">Model routing</span><span class="dst">${statusLabel(selected)}</span></div>
        <div class="dc">
          ${settings.providers.map(providerLine).join("")}
          ${settings.routes.map(routeLine).join("")}
        </div>
      </div>`;
}

function providerLine(provider: ModelProviderView) {
  const secret = provider.requiresSecret ? "external secret required" : "no secret";
  return `<div class="dr ${provider.status === "missing_key" ? "m" : ""}"><span class="g">${provider.kind}</span><span class="x">${escapeHtml(provider.label)}: ${statusLabel(provider)} &middot; ${escapeHtml(secret)} &middot; ${escapeHtml(provider.detail)}</span></div>`;
}

function routeLine(route: ModelSettingsView["routes"][number]) {
  const saved = route.saved ? "saved" : "unsaved";
  return `<div class="dr"><span class="g">role</span><span class="x">${escapeHtml(route.role)} -> ${escapeHtml(route.providerId)} / ${escapeHtml(route.modelId)} &middot; ${saved}</span></div>`;
}

function selectedProvider(settings: ModelSettingsView) {
  return settings.providers.find((provider) => provider.id === settings.selectedProviderId) ?? settings.providers[0] ?? {
    detail: "No provider settings loaded.",
    id: "none",
    kind: "unavailable",
    label: "No provider",
    models: [],
    requiresSecret: false,
    status: "not_configured",
  };
}

function statusLabel(provider: ModelProviderView) {
  const labels: Record<ModelProviderView["status"], string> = {
    missing_key: "missing API key",
    not_configured: "not configured",
    ready: "ready",
    unreachable: "unreachable",
  };
  return labels[provider.status];
}
