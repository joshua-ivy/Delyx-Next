import type { ExternalAgentRunArtifactView, ExternalAgentStateView } from "../features/externalAgents/externalAgentTypes";
import { escapeHtml } from "./html";

export function emptyExternalAgentBlock() {
  return `<div class="external-agent-stream">
        <div class="dm">No external agent run has been approved or captured.</div>
      </div>`;
}

export function externalAgentBlock(state: ExternalAgentStateView, runId: string | undefined) {
  const artifacts = runId ? state.artifacts.filter((artifact) => artifact.runId === runId) : [];
  if (artifacts.length === 0) {
    return emptyExternalAgentBlock();
  }

  return `<div class="external-agent-stream">
        ${artifacts.map(artifactBlock).join("")}
      </div>`;
}

function artifactBlock(artifact: ExternalAgentRunArtifactView) {
  return `<div class="banner">External agent ${escapeHtml(artifact.adapterId)} &middot; ${escapeHtml(artifact.status)}</div>
      <div><span class="pr">scope &gt;</span> ${escapeHtml(artifact.scope)}</div>
      ${artifact.transcript.map(eventLine).join("")}
      <div><span class="pr">output &gt;</span> ${escapeHtml(artifact.terminalOutput)}</div>
      ${diffLine(artifact)}
      ${testLine(artifact)}`;
}

function eventLine(event: ExternalAgentRunArtifactView["transcript"][number]) {
  return `<div><span class="pr">${escapeHtml(event.kind)} &gt;</span> ${escapeHtml(event.message)} <span class="bk">${escapeHtml(event.timestamp)}</span></div>`;
}

function diffLine(artifact: ExternalAgentRunArtifactView) {
  if (!artifact.diffSummary) {
    return "";
  }
  const review = artifact.reviewRequired ? "review required" : "no review flag";
  return `<div><span class="pr">diff &gt;</span> ${escapeHtml(artifact.diffSummary)} &middot; ${review}</div>`;
}

function testLine(artifact: ExternalAgentRunArtifactView) {
  if (artifact.testArtifactIds.length === 0) {
    return '<div><span class="pr">tests &gt;</span> not trusted; no TestArtifact linked</div>';
  }
  return `<div><span class="pr">tests &gt;</span> linked ${escapeHtml(artifact.testArtifactIds.join(", "))}</div>`;
}
