import { invoke } from "@tauri-apps/api/core";

export interface RuntimeBridgeState {
  label: string;
  mode: "tauri" | "web";
  status?: RuntimeStatusView;
}

export interface RuntimeStatusView {
  appIdentifier: string;
  appName: string;
  codingRoute?: {
    modelId: string;
    providerId: string;
  };
  milestone: string;
  providers: Array<{
    id: string;
    kind: string;
    label: string;
    message: string;
    models: string[];
    status: string;
  }>;
}

export async function loadRuntimeBridgeState(): Promise<RuntimeBridgeState> {
  try {
    const status = await invoke<RuntimeStatusView>("runtime_status");
    return { label: `Rust bridge / ${status.milestone}`, mode: "tauri", status };
  } catch {
    return { label: "Web preview / Rust bridge unavailable", mode: "web" };
  }
}
