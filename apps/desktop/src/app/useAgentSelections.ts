import { useEffect, useRef, useState } from "react";

import type { ExternalAgentAdapterView } from "../features/externalAgents/externalAgentTypes";
import { ensureProject } from "../features/projects/projectClient";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";

export function useAgentSelections(adapters: ExternalAgentAdapterView[], activeProject: WorkspaceProject) {
  const [qaqcAdapterId, setQaqcAdapterId] = useState<string | undefined>(undefined);
  const [qaqcModel, setQaqcModel] = useState<string | undefined>(undefined);
  const [workerAdapterId, setWorkerAdapterId] = useState<string | undefined>(undefined);
  const [workerMode, setWorkerMode] = useState<"read_only" | "workspace_write">("read_only");
  // Auto-enable QA/QC once when a reviewer CLI is first detected. Tracked by a ref
  // so turning it off manually afterwards isn't re-enabled on the next render.
  const qaqcAutoSelected = useRef(false);
  const [nativeProjectId, setNativeProjectId] = useState<string | undefined>(undefined);
  // Default QA/QC on: pick the first detected reviewer CLI (Claude Code, else
  // Codex) so generated code is checked out of the box. Runs once.
  useEffect(() => {
    if (qaqcAutoSelected.current || qaqcAdapterId) {
      return;
    }
    const reviewer =
      adapters.find((adapter) => adapter.id === "claude-code" && adapter.status === "available")
      ?? adapters.find((adapter) => adapter.id === "codex-cli" && adapter.status === "available");
    if (reviewer) {
      qaqcAutoSelected.current = true;
      setQaqcAdapterId(reviewer.id);
    }
  }, [adapters, qaqcAdapterId]);
  // Resolve (or create) the native project record for the active workspace so the
  // attachment pipeline has a real project id + policy to classify against.
  useEffect(() => {
    let cancelled = false;
    setNativeProjectId(undefined);
    void ensureProject(activeProject.name, activeProject.path)
      .then((record) => {
        if (!cancelled) setNativeProjectId(record.id);
      })
      .catch(() => {
        // Desktop runtime unavailable — attachments stay disabled.
      });
    return () => {
      cancelled = true;
    };
  }, [activeProject.name, activeProject.path]);
  const selectQaqc = (adapterId: string | undefined) => {
    setQaqcAdapterId(adapterId);
    // Reset to the new reviewer's economical default model.
    setQaqcModel(undefined);
  };
  const selectQaqcModel = (model: string) => {
    setQaqcModel(model);
  };
  const selectWorker = (adapterId: string | undefined, mode: "read_only" | "workspace_write" = "read_only") => {
    setWorkerAdapterId(adapterId);
    setWorkerMode(mode);
  };
  return {
    nativeProjectId,
    qaqcAdapterId,
    qaqcModel,
    selectQaqc,
    selectQaqcModel,
    selectWorker,
    workerAdapterId,
    workerMode,
  };
}
