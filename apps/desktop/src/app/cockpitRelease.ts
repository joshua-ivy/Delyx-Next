import type { ReleaseStateView } from "../features/release/releaseTypes";
import { escapeHtml } from "./html";

export function emptyReleaseBlock() {
  return `<div class="dfile release-review">
        <div class="dh"><span class="fn">Release readiness</span><span class="dst">pending</span></div>
        <div class="dc">
          <div class="dr"><span class="g">-</span><span class="x">No release smoke artifact or support bundle export loaded.</span></div>
        </div>
      </div>`;
}

export function releaseBlock(state: ReleaseStateView) {
  return `<div class="dfile release-review">
        <div class="dh"><span class="fn">Release readiness</span><span class="dst">${escapeHtml(state.platform)} ${escapeHtml(state.bundleTarget)}</span></div>
        <div class="dc">
          <div class="dr"><span class="g">build</span><span class="x">Windows dev build config &middot; ${escapeHtml(state.installer)}</span></div>
          <div class="dr"><span class="g">smoke</span><span class="x">${escapeHtml(releaseSmokeLabel(state.smokeStatus))}</span></div>
          <div class="dr"><span class="g">sign</span><span class="x">${escapeHtml(state.signing.status)} &middot; ${escapeHtml(state.signing.message)}</span></div>
          <div class="dr"><span class="g">bundle</span><span class="x">${escapeHtml(supportBundleLabel(state))}</span></div>
          <div class="dr"><span class="g">update</span><span class="x">${escapeHtml(state.updateMetadata.status)} &middot; ${escapeHtml(state.updateMetadata.channel)}</span></div>
        </div>
      </div>`;
}

function releaseSmokeLabel(status: ReleaseStateView["smokeStatus"]) {
  const labels: Record<ReleaseStateView["smokeStatus"], string> = {
    not_loaded: "No release smoke artifact loaded in this UI session.",
    passed: "Release smoke passed with a captured artifact.",
  };
  return labels[status];
}

function supportBundleLabel(state: ReleaseStateView) {
  if (state.supportBundle.exportStatus === "not_exported") {
    return state.supportBundle.secretPolicy;
  }
  return `${state.supportBundle.exportStatus} - ${state.supportBundle.secretPolicy}`;
}
