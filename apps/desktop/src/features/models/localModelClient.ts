import { invoke } from "@tauri-apps/api/core";

export interface LocalModelProfile {
  id: string;
  displayName: string;
  runtime: string;
  format: string;
  modelPath: string;
  chatTemplatePath?: string;
  tokenizerPath?: string;
  contextWindow: number;
  supportsTools: boolean;
  sha256?: string;
  sizeBytes?: number;
  loadStatus: string;
  lastError?: string;
  temperature?: number;
  topP?: number;
  topK?: number;
  repeatPenalty?: number;
  maxTokens?: number;
}

export interface ModelSamplingRequest {
  id: string;
  temperature?: number;
  topP?: number;
  topK?: number;
  repeatPenalty?: number;
  maxTokens?: number;
}

export interface ImportLocalModelRequest {
  modelPath: string;
  displayName?: string;
  chatTemplatePath?: string;
  tokenizerPath?: string;
  contextWindow?: number;
}

export interface LocalModelLifecycleView {
  status: string;
  message: string;
  profile?: LocalModelProfile;
}

export function importLocalModel(request: ImportLocalModelRequest) {
  return invoke<LocalModelLifecycleView>("local_model_import", { request });
}

export function listLocalModels() {
  return invoke<LocalModelProfile[]>("local_model_list");
}

export function setLocalModelSampling(request: ModelSamplingRequest) {
  return invoke<LocalModelLifecycleView>("local_model_set_sampling", { request });
}

export function unloadLocalModel(id: string) {
  return invoke<LocalModelLifecycleView>("local_model_unload", { request: { id } });
}

export function removeLocalModelProfile(id: string) {
  return invoke<LocalModelLifecycleView>("local_model_remove_profile", { request: { id } });
}
