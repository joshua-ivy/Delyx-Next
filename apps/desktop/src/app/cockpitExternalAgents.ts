import type {
  ExternalAgentAdapterView,
  ExternalAgentCommandContractView,
  ExternalAgentRunArtifactView,
  ExternalAgentStateView,
} from "../features/externalAgents/externalAgentTypes";
import { escapeHtml } from "./html";

export function emptyExternalAgentBlock() {
  return `<div class="external-agent-stream output-block">
        <div class="dm" data-log-line>External agent adapter status has not been loaded.</div>
        <div class="dm" data-log-line>No external agent command contract has been proposed or approved.</div>
        <div class="dm" data-log-line>No external agent run has been approved or captured.</div>
      </div>`;
}

export function externalAgentBlock(state: ExternalAgentStateView, runId: string | undefined) {
  const contracts = runId ? state.contracts.filter((contract) => contract.runId === runId) : [];
  const artifacts = runId ? state.artifacts.filter((artifact) => artifact.runId === runId) : [];
  if (contracts.length === 0 && artifacts.length === 0) {
    return `<div class="external-agent-stream output-block">
        ${adapterBlock(state.adapters)}
        <div class="dm" data-log-line>No external agent command contract has been proposed or approved.</div>
        <div class="dm" data-log-line>No external agent run has been approved or captured.</div>
      </div>`;
  }

  return `<div class="external-agent-stream output-block">
        ${adapterBlock(state.adapters)}
        ${contracts.map(contractBlock).join("")}
        ${artifacts.map(artifactBlock).join("")}
      </div>`;
}

function adapterBlock(adapters: ExternalAgentAdapterView[]) {
  if (adapters.length === 0) {
    return '<div class="dm" data-log-line>External agent adapter status has not been loaded.</div>';
  }
  return adapters.map((adapter) => (
    `<div data-log-line><span class="pr">adapter &gt;</span> ${escapeHtml(adapter.label)} &middot; ${escapeHtml(adapter.status)} &middot; ${escapeHtml(adapter.detail)}</div>`
  )).join("");
}

function contractBlock(contract: ExternalAgentCommandContractView) {
  return `<div class="banner" data-log-line>Command contract ${escapeHtml(contract.adapterId)} &middot; ${escapeHtml(contract.status)}</div>
      <div data-log-line><span class="pr">permission &gt;</span> ${permissionLabel(contract)}</div>
      <div data-log-line><span class="pr">program &gt;</span> ${escapeHtml(contract.program)}</div>
      <div data-log-line><span class="pr">args[] &gt;</span> ${argsList(contract.args)}</div>
      <div data-log-line><span class="pr">cwd &gt;</span> ${escapeHtml(contract.workingDirectory)}</div>
      <div data-log-line><span class="pr">transcript &gt;</span> ${escapeHtml(contract.transcriptFormat)}</div>
      <div data-log-line><span class="pr">tools &gt;</span> ${escapeHtml(contract.requiredDelyxTools.join(", "))}</div>
      <div data-log-line><span class="pr">safety &gt;</span> ${escapeHtml(contract.safetySummary)}</div>`;
}

function argsList(args: string[]) {
  if (args.length === 0) {
    return "[]";
  }
  return args.map((arg, index) => `[${index}] ${escapeHtml(arg)}`).join(" ");
}

function artifactBlock(artifact: ExternalAgentRunArtifactView) {
  return `<div class="banner" data-log-line>External agent ${escapeHtml(artifact.adapterId)} &middot; ${escapeHtml(artifact.status)}</div>
      <div data-log-line><span class="pr">scope &gt;</span> ${escapeHtml(artifact.scope)}</div>
      ${artifact.transcript.map(eventLine).join("")}
      <div class="long-output" data-log-line><span class="pr">output &gt;</span> ${escapeHtml(artifact.terminalOutput)}</div>
      ${diffLine(artifact)}
      ${testLine(artifact)}`;
}

function permissionLabel(contract: ExternalAgentCommandContractView) {
  return contract.permissionMode === "workspace_write" ? "workspace write" : "read only";
}

function eventLine(event: ExternalAgentRunArtifactView["transcript"][number]) {
  return `<div data-log-line><span class="pr">${escapeHtml(event.kind)} &gt;</span> ${escapeHtml(event.message)} <span class="bk">${escapeHtml(event.timestamp)}</span></div>`;
}

function diffLine(artifact: ExternalAgentRunArtifactView) {
  if (!artifact.diffSummary) {
    return "";
  }
  const review = artifact.reviewRequired ? "review required" : "no review flag";
  return `<div data-log-line><span class="pr">diff &gt;</span> ${escapeHtml(artifact.diffSummary)} &middot; ${review}</div>`;
}

function testLine(artifact: ExternalAgentRunArtifactView) {
  if (artifact.testArtifactIds.length === 0) {
    return '<div data-log-line><span class="pr">tests &gt;</span> not trusted; no TestArtifact linked</div>';
  }
  return `<div data-log-line><span class="pr">tests &gt;</span> linked ${escapeHtml(artifact.testArtifactIds.join(", "))}</div>`;
}
