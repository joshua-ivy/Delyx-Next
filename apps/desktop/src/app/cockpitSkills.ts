import type { SkillManifestView, SkillStateView } from "../features/skills/skillTypes";
import { escapeHtml } from "./html";

export function emptySkillBlock() {
  return `<div class="dfile skill-review">
        <div class="dh"><span class="fn">Skills</span><span class="dst">inactive</span></div>
        <div class="dc">
          <div class="dr"><span class="g">-</span><span class="x">No skills imported. Third-party skills never auto-activate.</span></div>
        </div>
      </div>`;
}

export function skillBlock(state: SkillStateView) {
  if (state.skills.length === 0) {
    return emptySkillBlock();
  }

  return `<div class="dfile skill-review">
        <div class="dh"><span class="fn">Skills</span><span class="dst">${state.skills.length} imported</span></div>
        <div class="dc">${state.skills.map(skillLine).join("")}</div>
      </div>`;
}

function skillLine(skill: SkillManifestView) {
  const scripts = skill.permissions.canRunScripts ? "scripts allowed" : "scripts blocked";
  const files = skill.permissions.canEditFiles ? "edits allowed" : "edits blocked";
  const network = skill.permissions.canUseNetwork ? "network allowed" : "network blocked";
  return `<div class="dr ${skill.status === "active" ? "p" : ""}"><span class="g">skill</span><span class="x">${escapeHtml(skill.name)} &middot; ${escapeHtml(skill.status)} &middot; ${escapeHtml(skill.trust)} &middot; ${escapeHtml(skill.source)} #${escapeHtml(skill.sourceHash)} &middot; ${scripts}, ${files}, ${network}</span></div>`;
}
