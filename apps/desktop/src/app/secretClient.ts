import { invoke } from "@tauri-apps/api/core";

export interface SecretProviderView {
  id: string;
  label: string;
  hasKey: boolean;
}

export interface SecretStatusView {
  providers: SecretProviderView[];
}

export async function loadSecretStatus(): Promise<SecretStatusView | undefined> {
  try {
    return await invoke<SecretStatusView>("secret_status");
  } catch {
    return undefined;
  }
}

export async function setSecret(providerId: string, value: string): Promise<SecretStatusView> {
  return invoke<SecretStatusView>("secret_set", { providerId, value });
}

export async function clearSecret(providerId: string): Promise<SecretStatusView> {
  return invoke<SecretStatusView>("secret_clear", { providerId });
}
