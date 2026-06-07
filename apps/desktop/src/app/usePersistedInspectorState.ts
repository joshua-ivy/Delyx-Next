import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { currentAutomationState } from "../features/automations/automationData";
import type { AutomationStateView } from "../features/automations/automationTypes";
import { currentMemoryState } from "../features/memory/memoryData";
import type { MemoryStateView } from "../features/memory/memoryTypes";
import { currentReleaseState } from "../features/release/releaseData";
import type { ReleaseStateView } from "../features/release/releaseTypes";
import { currentSkillState } from "../features/skills/skillData";
import type { SkillStateView } from "../features/skills/skillTypes";

export interface PersistedInspectorState {
  automationState: AutomationStateView;
  memoryState: MemoryStateView;
  releaseState: ReleaseStateView;
  skillState: SkillStateView;
}

const fallbackInspectorState: PersistedInspectorState = {
  automationState: currentAutomationState,
  memoryState: currentMemoryState,
  releaseState: currentReleaseState,
  skillState: currentSkillState,
};

export function usePersistedInspectorState() {
  const [state, setState] = useState<PersistedInspectorState>(fallbackInspectorState);
  useEffect(() => {
    let cancelled = false;
    void loadPersistedInspectorState().then((snapshot) => {
      if (!cancelled) {
        setState(snapshot);
      }
    });
    return () => {
      cancelled = true;
    };
  }, []);
  return state;
}

async function loadPersistedInspectorState(): Promise<PersistedInspectorState> {
  if (!hasTauriRuntime()) {
    return fallbackInspectorState;
  }
  const [memoryState, skillState, automationState, releaseState] = await Promise.all([
    invokeOrFallback<MemoryStateView>("memory_snapshot", currentMemoryState),
    invokeOrFallback<SkillStateView>("skill_snapshot", currentSkillState),
    invokeOrFallback<AutomationStateView>("automation_snapshot", currentAutomationState),
    invokeOrFallback<ReleaseStateView>("release_snapshot", currentReleaseState),
  ]);
  return { automationState, memoryState, releaseState, skillState };
}

async function invokeOrFallback<T>(command: string, fallback: T): Promise<T> {
  try {
    return await invoke<T>(command);
  } catch {
    return fallback;
  }
}

function hasTauriRuntime() {
  return Boolean((globalThis as Record<string, unknown>).__TAURI_INTERNALS__);
}
