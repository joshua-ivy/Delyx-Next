import type { ThreadRoleMessage } from "../models/modelTypes";
import type { PatchProposalRequestView } from "./patchClient";
import type { PlanView } from "../plans/planTypes";
import type { TaskThread } from "../threads/threadTypes";
import type { WorkspaceProject } from "../workspace/workspaceTypes";

export interface WorkspacePatchDraftFile {
  path: string;
  contents: string;
  truncated: boolean;
}

export interface OllamaPatchDraftInput {
  approvalId: string;
  clientId: string;
  plan: PlanView;
  project: WorkspaceProject;
  readFiles: WorkspacePatchDraftFile[];
  runId: string;
  text: string;
}

export function createOllamaPatchDraftMessages(
  thread: TaskThread,
  plan: PlanView,
  project: WorkspaceProject,
  readFiles: WorkspacePatchDraftFile[],
): ThreadRoleMessage[] {
  return [
    {
      role: "system",
      content: [
        "You are Delyx Next PatchDraftAgent.",
        "Return only JSON. Do not wrap it in commentary.",
        "Create complete replacement file contents for the approved plan.",
        "Use only files the user provided. Do not invent paths or claim commands ran.",
        "Do not apply patches, run tests, or say the work is done.",
      ].join(" "),
    },
    {
      role: "user",
      content: [
        `Project: ${project.name}`,
        `Approved root: ${project.approvedRoots[0] ?? project.path}`,
        `Thread goal:\n${thread.goal}`,
        `Approved plan steps:\n${plan.steps.map((step) => `- ${step}`).join("\n")}`,
        `Files with current contents:\n${fileBlocks(readFiles)}`,
        "Return JSON: {\"files\":[{\"path\":\"exact listed path\",\"after\":\"complete replacement contents\"}],\"summary\":\"short rationale\",\"risks\":[\"risk\"]}",
      ].join("\n\n"),
    },
  ];
}

export function createPatchProposalRequestFromOllamaText(
  input: OllamaPatchDraftInput,
): PatchProposalRequestView {
  const payload = parseOllamaPatchDraftText(input.text);
  const files = draftFiles(payload.files);
  const allowed = allowedFileMap(input.project, input.readFiles);
  const seen = new Set<string>();
  const patchFiles = files.map((file) => {
    const source = allowed.get(normalizePath(file.path));
    if (!source) {
      throw new Error(`Ollama patch referenced an unapproved file: ${file.path}`);
    }
    const key = normalizePath(source.path);
    if (seen.has(key)) {
      throw new Error(`Ollama patch duplicated file: ${source.path}`);
    }
    seen.add(key);
    if (file.after.trim().length === 0) {
      throw new Error(`Ollama patch returned empty contents for ${source.path}.`);
    }
    if (file.after === source.contents) {
      throw new Error(`Ollama patch left ${source.path} unchanged.`);
    }
    return {
      after: file.after,
      path: absoluteWorkspacePath(input.project.path, source.path),
    };
  });

  return {
    approvalId: input.approvalId,
    approvedRoots: input.project.approvedRoots,
    clientId: input.clientId,
    files: patchFiles,
    runId: input.runId,
  };
}

export function parseOllamaPatchDraftText(text: string): Record<string, unknown> {
  const payload = JSON.parse(extractJsonPayload(text)) as unknown;
  if (!isRecord(payload)) {
    throw new Error("Ollama patch response was not a JSON object.");
  }
  return payload;
}

function fileBlocks(files: WorkspacePatchDraftFile[]) {
  return files.map((file) => [
    `--- ${file.path}${file.truncated ? " (truncated)" : ""} ---`,
    file.contents,
  ].join("\n")).join("\n\n") || "No approved file contents were read.";
}

function draftFiles(value: unknown) {
  if (!Array.isArray(value) || value.length === 0 || value.length > 4) {
    throw new Error("Ollama patch JSON must include 1-4 file entries.");
  }
  return value.map((item) => {
    if (!isRecord(item) || typeof item.path !== "string" || typeof item.after !== "string") {
      throw new Error("Each Ollama patch file must include path and after strings.");
    }
    return { after: item.after, path: item.path.trim() };
  });
}

function allowedFileMap(project: WorkspaceProject, files: WorkspacePatchDraftFile[]) {
  const map = new Map<string, WorkspacePatchDraftFile>();
  for (const file of files) {
    map.set(normalizePath(file.path), file);
    map.set(normalizePath(absoluteWorkspacePath(project.path, file.path)), file);
  }
  return map;
}

function absoluteWorkspacePath(projectPath: string, relativePath: string) {
  const root = projectPath.replace(/\\/g, "/").replace(/\/+$/, "");
  const relative = relativePath.replace(/\\/g, "/").replace(/^\/+/, "");
  return `${root}/${relative}`;
}

function extractJsonPayload(text: string) {
  const fenced = text.match(/```(?:json)?\s*([\s\S]*?)```/i)?.[1];
  if (fenced) {
    return fenced.trim();
  }
  const start = text.indexOf("{");
  const end = text.lastIndexOf("}");
  if (start < 0 || end <= start) {
    throw new Error("Ollama patch response did not contain JSON.");
  }
  return text.slice(start, end + 1);
}

function normalizePath(path: string) {
  return path.replace(/\\/g, "/").replace(/^\.\//, "").toLowerCase();
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return !!value && typeof value === "object" && !Array.isArray(value);
}
