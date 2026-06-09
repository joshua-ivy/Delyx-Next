# Delyx Next Embedded Model Runtime — 100% Implementation Plan

**Goal:** Make Delyx Next run local models directly inside the Tauri/Rust desktop app without requiring Ollama, LM Studio, llama.cpp server, or any other user-managed app. External providers remain optional adapters.

**Recommended default runtime:** `mistral.rs` embedded through its Rust SDK.

**Resulting architecture:**

```text
Delyx Next UI
  -> Tauri commands
  -> Model registry / role routes
  -> Delyx-owned embedded runtime
  -> mistral.rs Rust SDK
  -> local model profile + local model files
```

This is not a llama.cpp server path. It is an in-process Delyx runtime. Delyx owns model import, profile storage, route selection, load/unload state, chat, PatchDraft, receipts, and fallback routing.

---

## 0. Review verdict on the recommendation

Keep the earlier recommendation, with these corrections:

1. **Use `mistral.rs` as the first embedded runtime**, not llama.cpp sidecars.
2. **Do not remove Ollama.** Keep it as an optional adapter for users who already like it.
3. **Do not hardwire the app to `mistral.rs` everywhere.** Add a provider-aware `send_model_chat` layer, then move composer and PatchDraft through it.
4. **Store models as Delyx model profiles.** Do not store weights in SQLite. Store local paths, metadata, status, and route assignments.
5. **Feature-gate the broad dependency first.** Delyx Next rules say broad dependencies need justification. The embedded runtime should ship behind `embedded_mistral` until validated.
6. **Make the UI truthful.** Users should see imported, loading, ready, failed, unloaded, unsupported format, and fallback states.
7. **The first fully shippable embedded path should be local GGUF import + non-streaming chat + PatchDraft.** Streaming, model download, auto-tune, embeddings, and structured/tool calls can follow after the direct runtime is proven.

---

## 1. Current repo facts this plan is based on

Delyx Next currently says it is local-first, UI-first, and Tauri/React/Rust/SQLite-based.

Current model reality:

- `Ollama` is the only real live model execution path.
- OpenAI-compatible providers are config/health stubs only.
- The Rust model provider enum only has `Mock`, `Ollama`, and `OpenAiCompatible`.
- `runtime_bridge.rs` detects Ollama, maps runtime status to the UI, and exposes `ollama_chat`.
- The composer calls `sendOllamaChat` for normal replies.
- PatchDraft currently calls `send_ollama_chat` directly.
- The frontend model picker currently passes only `modelId`, not a provider/model pair.
- The SQLite migration already has `model_role_routes`, so model routing is partially in place.

That means the implementation must touch both Rust and TypeScript.

---

## 2. Definition of done

This is **done** only when all of the following are true:

- Delyx Next can import/register a local GGUF model profile without Ollama installed.
- `runtime_status` shows a `delyx-local` provider with local models.
- The model picker can select a Delyx-managed local model.
- The composer can send a chat request to the selected Delyx local model.
- PatchDraft can use the selected Delyx local model through the same provider-aware dispatch layer.
- Ollama still works if selected.
- CLI model providers still work if selected.
- Provider/model selection is unambiguous, even if two providers expose the same model name.
- Model load failures are visible and do not crash the app.
- Removing/unloading a local model does not delete the model file unless a future explicit destructive approval is added.
- Tests cover persistence, provider routing, runtime status mapping, model selection, and PatchDraft request propagation.
- Validation passes:

```powershell
.\.tools\npm.cmd run typecheck
.\.tools\npm.cmd test
.\.tools\npm.cmd run build
cargo test --workspace
cargo test --workspace --features embedded_mistral
```

---

## 3. PR-sized implementation sequence

Do this in small PRs. Do not try to do everything in one commit.

```text
PR A  - Provider types and UI model-selection key
PR B  - Local model profile persistence
PR C  - Runtime status includes Delyx Local provider
PR D  - Embedded mistral.rs runtime, feature-gated
PR E  - Provider-aware model chat command
PR F  - Composer uses provider-aware model chat
PR G  - PatchDraft uses provider-aware model chat
PR H  - Lifecycle commands: unload/remove/health
PR I  - UI import/management surface
PR J  - Docs, validation, and cleanup
```

---

# PR A — Provider types and unambiguous model selection

## Files

```text
apps/desktop/src-tauri/src/model_provider.rs
apps/desktop/src-tauri/src/model_provider_tests.rs
apps/desktop/src-tauri/src/runtime_bridge.rs
apps/desktop/src/app/runtimeBridge.ts
apps/desktop/src/features/models/modelTypes.ts
apps/desktop/src/features/models/modelData.ts
apps/desktop/src/app/modelSelection.ts
apps/desktop/src/app/cliModels.ts
apps/desktop/src/app/FocusShell.tsx
apps/desktop/src/app/FocusOverlays.tsx
```

## Rust provider changes

In `model_provider.rs`, change provider kinds:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    Mock,
    DelyxLocal,
    Ollama,
    OpenAiCompatible,
}
```

Expand statuses:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderStatus {
    Ready,
    Loading,
    MissingApiKey,
    ModelMissing,
    NotConfigured,
    Unreachable,
    Failed,
}
```

Expand `ModelInfo`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
    pub context_window: u32,
    pub supports_tools: bool,
    pub format: Option<String>,
    pub runtime: Option<String>,
    pub path: Option<String>,
}
```

Add a helper constructor so tests do not become noisy:

```rust
impl ModelInfo {
    pub fn local_chat_model(id: &str, display_name: &str, context_window: u32) -> Self {
        Self {
            id: id.to_string(),
            display_name: display_name.to_string(),
            context_window,
            supports_tools: false,
            format: Some("gguf".to_string()),
            runtime: Some("mistralrs".to_string()),
            path: None,
        }
    }
}
```

Update the existing private `model(...)` helper:

```rust
fn model(id: &str, display_name: &str, supports_tools: bool) -> ModelInfo {
    ModelInfo {
        id: id.to_string(),
        display_name: display_name.to_string(),
        context_window: 8192,
        supports_tools,
        format: None,
        runtime: None,
        path: None,
    }
}
```

Add Delyx local provider helper:

```rust
pub fn delyx_local_provider(checked_at: u64, models: Vec<ModelInfo>) -> ModelProvider {
    let ready = !models.is_empty();
    ModelProvider {
        id: "delyx-local".to_string(),
        kind: ProviderKind::DelyxLocal,
        label: "Delyx Local".to_string(),
        health: ProviderHealth {
            checked_at,
            status: if ready {
                ProviderStatus::Ready
            } else {
                ProviderStatus::NotConfigured
            },
            message: if ready {
                format!("{} Delyx-managed local model(s) available.", models.len())
            } else {
                "No Delyx-managed local models imported yet.".to_string()
            },
        },
        models,
        secret_policy: SecretPolicy::NoSecretRequired,
    }
}
```

Update `provider_kind` in `runtime_bridge.rs`:

```rust
fn provider_kind(kind: ProviderKind) -> &'static str {
    match kind {
        ProviderKind::Mock => "mock",
        ProviderKind::DelyxLocal => "delyx_local",
        ProviderKind::Ollama => "ollama",
        ProviderKind::OpenAiCompatible => "openai_compatible",
    }
}
```

Update `provider_status_label`:

```rust
fn provider_status_label(status: ProviderStatus) -> &'static str {
    match status {
        ProviderStatus::Failed => "failed",
        ProviderStatus::Loading => "loading",
        ProviderStatus::MissingApiKey => "missing_key",
        ProviderStatus::ModelMissing => "model_missing",
        ProviderStatus::NotConfigured => "not_configured",
        ProviderStatus::Ready => "ready",
        ProviderStatus::Unreachable => "unreachable",
    }
}
```

## TypeScript provider changes

In `modelTypes.ts`:

```ts
export type ProviderKind = "delyx_local" | "ollama" | "openai_compatible" | "cli" | "unavailable";
export type ProviderStatus = "ready" | "loading" | "missing_key" | "model_missing" | "not_configured" | "unreachable" | "failed";
```

Add a typed selection key:

```ts
export interface ModelSelectionKey {
  providerId: string;
  modelId: string;
}
```

Change the picker callback from model-only to provider/model pair:

```ts
onSelectModel: (selection: ModelSelectionKey) => void;
```

Update these files accordingly:

```text
FocusShell.tsx
FocusOverlays.tsx
FocusSettings.tsx, if it also accepts onSelectModel
AppShell.tsx
cliModels.ts
modelSelection.ts
```

In `FocusOverlays.tsx`, change the model button:

```tsx
onClick={() => {
  onSelectModel({ providerId: item.id, modelId: model });
  onClose();
}}
```

This matters because `delyx-local` and `ollama-local` may both have a model named `qwen-coder`.

## Replace `selectOllamaCodingModel`

In `modelSelection.ts`, replace Ollama-only routing with provider-aware routing:

```ts
import type { ModelSelectionKey, ModelSettingsView } from "../features/models/modelTypes";

export function selectCodingModel(settings: ModelSettingsView, selection: ModelSelectionKey): ModelSettingsView {
  const provider = settings.providers.find((item) => item.id === selection.providerId);
  if (!provider || provider.status !== "ready" || !provider.models.includes(selection.modelId)) {
    return settings;
  }
  return {
    ...settings,
    routes: [
      { modelId: selection.modelId, providerId: selection.providerId, role: "coding", saved: false },
      ...settings.routes.filter((route) => route.role !== "coding"),
    ],
    selectedProviderId: selection.providerId,
  };
}
```

Update `cliModels.ts`:

```ts
import type { ExternalAgentAdapterView } from "../features/externalAgents/externalAgentTypes";
import type { ModelProviderView, ModelSelectionKey, ModelSettingsView } from "../features/models/modelTypes";
import { selectCodingModel } from "./modelSelection";

export function selectModelRoute(
  settings: ModelSettingsView,
  adapters: ExternalAgentAdapterView[],
  selection: ModelSelectionKey,
): ModelSettingsView {
  if (adapters.some((adapter) => adapter.id === selection.providerId && selection.modelId === adapter.id && CLI_LABELS[adapter.id])) {
    return { ...settings, selectedProviderId: selection.providerId };
  }
  return selectCodingModel(settings, selection);
}
```

## Tests for PR A

Add or update tests to prove:

- `ProviderKind::DelyxLocal` maps to `delyx_local`.
- TypeScript runtime mapper accepts `delyx_local`.
- Selecting a model preserves its provider ID.
- A duplicate model ID across providers selects the chosen provider, not always Ollama.

---

# PR B — Local model profile persistence

## Files

```text
apps/desktop/src-tauri/migrations/0001_agent_run_ledger.sql
apps/desktop/src-tauri/src/sqlite_store.rs
apps/desktop/src-tauri/src/model_embedded_persistence.rs
apps/desktop/src-tauri/src/model_embedded_persistence_tests.rs
apps/desktop/src-tauri/src/lib.rs
```

## Migration

Append this to `0001_agent_run_ledger.sql`:

```sql
CREATE TABLE IF NOT EXISTS local_model_profiles (
  id TEXT PRIMARY KEY NOT NULL,
  display_name TEXT NOT NULL,
  runtime TEXT NOT NULL,
  format TEXT NOT NULL,
  model_path TEXT NOT NULL,
  chat_template_path TEXT,
  tokenizer_path TEXT,
  context_window INTEGER NOT NULL DEFAULT 8192,
  supports_tools INTEGER NOT NULL DEFAULT 0,
  sha256 TEXT,
  size_bytes INTEGER,
  load_status TEXT NOT NULL DEFAULT 'unloaded',
  last_error TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

Add a safety column checker in `sqlite_store.rs`:

```rust
fn migrate(connection: &Connection) -> rusqlite::Result<()> {
    connection.pragma_update(None, "foreign_keys", "ON")?;
    connection.execute_batch(AGENT_RUN_MIGRATION)?;
    ensure_agent_run_columns(connection)?;
    ensure_evidence_columns(connection)?;
    ensure_patch_record_columns(connection)?;
    ensure_patch_file_columns(connection)?;
    ensure_local_model_profile_columns(connection)?;
    Ok(())
}

fn ensure_local_model_profile_columns(connection: &Connection) -> rusqlite::Result<()> {
    let columns = table_columns(connection, "local_model_profiles")?;
    for (name, definition) in [
        ("tokenizer_path", "TEXT"),
        ("load_status", "TEXT NOT NULL DEFAULT 'unloaded'"),
        ("last_error", "TEXT"),
    ] {
        if !columns.iter().any(|column| column == name) {
            connection.execute(
                &format!("ALTER TABLE local_model_profiles ADD COLUMN {name} {definition}"),
                [],
            )?;
        }
    }
    Ok(())
}
```

## Persistence model

Create `model_embedded_persistence.rs`:

```rust
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalModelProfile {
    pub id: String,
    pub display_name: String,
    pub runtime: String,
    pub format: String,
    pub model_path: String,
    pub chat_template_path: Option<String>,
    pub tokenizer_path: Option<String>,
    pub context_window: u32,
    pub supports_tools: bool,
    pub sha256: Option<String>,
    pub size_bytes: Option<u64>,
    pub load_status: String,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportLocalModelRequest {
    pub model_path: String,
    pub display_name: Option<String>,
    pub chat_template_path: Option<String>,
    pub tokenizer_path: Option<String>,
    pub context_window: Option<u32>,
}

pub fn import_profile_to_path(path: &Path, request: ImportLocalModelRequest) -> Result<LocalModelProfile, String> {
    let profile = profile_from_request(request)?;
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    upsert_profile(&connection, &profile)?;
    Ok(profile)
}

pub fn list_profiles_from_path(path: &Path) -> Result<Vec<LocalModelProfile>, String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    list_profiles(&connection)
}

pub fn load_profile_from_path(path: &Path, id: &str) -> Result<LocalModelProfile, String> {
    list_profiles_from_path(path)?
        .into_iter()
        .find(|profile| profile.id == id)
        .ok_or_else(|| format!("Local model profile `{id}` was not found."))
}

pub fn delete_profile_from_path(path: &Path, id: &str) -> Result<(), String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    connection
        .execute("DELETE FROM local_model_profiles WHERE id = ?1", params![id.trim()])
        .map(|_| ())
        .map_err(sql_string)
}

pub fn mark_profile_status(path: &Path, id: &str, status: &str, error: Option<&str>) -> Result<(), String> {
    let connection = crate::sqlite_store::open_migrated_database(path).map_err(sql_string)?;
    connection
        .execute(
            "UPDATE local_model_profiles
             SET load_status = ?2, last_error = ?3, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?1",
            params![id, status, error],
        )
        .map(|_| ())
        .map_err(sql_string)
}

fn profile_from_request(request: ImportLocalModelRequest) -> Result<LocalModelProfile, String> {
    let model_path = PathBuf::from(request.model_path.trim());
    if !model_path.is_file() {
        return Err(format!("Model file does not exist: {}", model_path.display()));
    }
    let extension = model_path.extension().and_then(|value| value.to_str()).unwrap_or_default().to_ascii_lowercase();
    if extension != "gguf" {
        return Err("First embedded runtime milestone only supports .gguf files.".to_string());
    }
    let file_name = model_path.file_name().and_then(|value| value.to_str()).ok_or_else(|| "Model path has no filename.".to_string())?;
    let id = stable_profile_id(file_name);
    let metadata = std::fs::metadata(&model_path).map_err(|error| error.to_string())?;
    Ok(LocalModelProfile {
        id,
        display_name: request.display_name.unwrap_or_else(|| file_name.to_string()),
        runtime: "mistralrs".to_string(),
        format: "gguf".to_string(),
        model_path: model_path.display().to_string(),
        chat_template_path: non_empty(request.chat_template_path),
        tokenizer_path: non_empty(request.tokenizer_path),
        context_window: request.context_window.unwrap_or(8192),
        supports_tools: false,
        sha256: None,
        size_bytes: Some(metadata.len()),
        load_status: "unloaded".to_string(),
        last_error: None,
    })
}

fn stable_profile_id(file_name: &str) -> String {
    let mut id = file_name
        .trim()
        .trim_end_matches(".gguf")
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch.to_ascii_lowercase() } else { '-' })
        .collect::<String>();
    while id.contains("--") {
        id = id.replace("--", "-");
    }
    id.trim_matches('-').to_string()
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.map(|item| item.trim().to_string()).filter(|item| !item.is_empty())
}

fn upsert_profile(connection: &Connection, profile: &LocalModelProfile) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO local_model_profiles (
                id, display_name, runtime, format, model_path, chat_template_path, tokenizer_path,
                context_window, supports_tools, sha256, size_bytes, load_status, last_error
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
             ON CONFLICT(id) DO UPDATE SET
                display_name = excluded.display_name,
                runtime = excluded.runtime,
                format = excluded.format,
                model_path = excluded.model_path,
                chat_template_path = excluded.chat_template_path,
                tokenizer_path = excluded.tokenizer_path,
                context_window = excluded.context_window,
                supports_tools = excluded.supports_tools,
                sha256 = excluded.sha256,
                size_bytes = excluded.size_bytes,
                load_status = excluded.load_status,
                last_error = excluded.last_error,
                updated_at = CURRENT_TIMESTAMP",
            params![
                profile.id,
                profile.display_name,
                profile.runtime,
                profile.format,
                profile.model_path,
                profile.chat_template_path,
                profile.tokenizer_path,
                profile.context_window,
                profile.supports_tools as i64,
                profile.sha256,
                profile.size_bytes.map(|value| value as i64),
                profile.load_status,
                profile.last_error,
            ],
        )
        .map(|_| ())
        .map_err(sql_string)
}

fn list_profiles(connection: &Connection) -> Result<Vec<LocalModelProfile>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id, display_name, runtime, format, model_path, chat_template_path, tokenizer_path,
                    context_window, supports_tools, sha256, size_bytes, load_status, last_error
             FROM local_model_profiles
             ORDER BY display_name",
        )
        .map_err(sql_string)?;
    let rows = statement
        .query_map([], |row| {
            Ok(LocalModelProfile {
                id: row.get(0)?,
                display_name: row.get(1)?,
                runtime: row.get(2)?,
                format: row.get(3)?,
                model_path: row.get(4)?,
                chat_template_path: row.get(5)?,
                tokenizer_path: row.get(6)?,
                context_window: row.get::<_, i64>(7)? as u32,
                supports_tools: row.get::<_, i64>(8)? != 0,
                sha256: row.get(9)?,
                size_bytes: row.get::<_, Option<i64>>(10)?.map(|value| value as u64),
                load_status: row.get(11)?,
                last_error: row.get(12)?,
            })
        })
        .map_err(sql_string)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(sql_string)
}

fn sql_string(error: rusqlite::Error) -> String {
    error.to_string()
}
```

Add to `lib.rs`:

```rust
pub mod model_embedded_persistence;
```

## Tests for PR B

Add `model_embedded_persistence_tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use crate::model_embedded_persistence::{import_profile_to_path, list_profiles_from_path, ImportLocalModelRequest};

    #[test]
    fn imports_and_lists_local_gguf_profile_without_storing_weights() {
        let dir = std::env::temp_dir().join(format!("delyx-model-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let model = dir.join("qwen-test.Q4_K_M.gguf");
        std::fs::write(&model, b"not real weights, persistence test only").unwrap();
        let db = dir.join("db.sqlite3");

        let profile = import_profile_to_path(&db, ImportLocalModelRequest {
            model_path: model.display().to_string(),
            display_name: Some("Qwen Test".to_string()),
            chat_template_path: None,
            tokenizer_path: None,
            context_window: Some(4096),
        }).unwrap();

        assert_eq!(profile.runtime, "mistralrs");
        assert_eq!(profile.format, "gguf");
        assert_eq!(profile.context_window, 4096);

        let profiles = list_profiles_from_path(&db).unwrap();
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].model_path, model.display().to_string());
    }
}
```

---

# PR C — Runtime status includes Delyx Local provider

## Files

```text
apps/desktop/src-tauri/src/runtime_bridge.rs
apps/desktop/src-tauri/src/runtime_bridge_tests.rs
apps/desktop/src/app/runtimeBridge.ts
apps/desktop/src/features/models/modelData.ts
```

## Rust status integration

In `runtime_bridge.rs`, import profile persistence and provider helper:

```rust
use crate::model_embedded_persistence::list_profiles_from_path;
use crate::model_provider::{delyx_local_provider, ModelInfo, ...};
```

Add a method on bridge state:

```rust
impl RuntimeBridgeState {
    pub fn database_path(&self) -> &Path {
        &self.database_path
    }
}
```

Update `runtime_status_with_provider_and_version`:

```rust
pub fn runtime_status_with_provider_and_version(
    database_path: &Path,
    ollama: ModelProvider,
    ollama_version: Option<String>,
) -> Result<RuntimeStatusView, String> {
    let mut registry = ModelRegistry::with_runtime_defaults(0);
    registry.register_provider(delyx_local_provider(0, local_model_infos(database_path)?));
    registry.register_provider(ollama);

    for route in crate::model_provider_persistence::load_routes_from_path(database_path)? {
        let _ = registry.save_role_route(route.role, &route.provider_id, &route.model_id);
    }
    if registry.route_for(ModelRole::Coding).is_none() {
        save_detected_coding_route(&mut registry, database_path)?;
    }
    Ok(runtime_status_from_registry_with_version(&registry, ollama_version))
}

fn local_model_infos(database_path: &Path) -> Result<Vec<ModelInfo>, String> {
    Ok(list_profiles_from_path(database_path)?
        .into_iter()
        .map(|profile| ModelInfo {
            id: profile.id,
            display_name: profile.display_name,
            context_window: profile.context_window,
            supports_tools: profile.supports_tools,
            format: Some(profile.format),
            runtime: Some(profile.runtime),
            path: Some(profile.model_path),
        })
        .collect())
}
```

Update default route priority:

```rust
fn save_detected_coding_route(
    registry: &mut ModelRegistry,
    database_path: &Path,
) -> Result<(), String> {
    if let Some(model_id) = first_ready_model(registry, "delyx-local").map(|model| model.id.clone()) {
        let _ = registry.save_role_route(ModelRole::Coding, "delyx-local", &model_id);
    } else if let Some(model_id) = first_ready_model(registry, "ollama-local").map(|model| model.id.clone()) {
        let _ = registry.save_role_route(ModelRole::Coding, "ollama-local", &model_id);
    }
    crate::model_provider_persistence::save_routes_to_path(database_path, registry.routes())?;
    Ok(())
}

fn first_ready_model(registry: &ModelRegistry, provider_id: &str) -> Option<&ModelInfo> {
    registry
        .list_providers()
        .iter()
        .find(|provider| provider.id == provider_id && provider.health.status == ProviderStatus::Ready)
        .and_then(|provider| provider.models.first())
}
```

## Frontend runtime mapper

In `runtimeBridge.ts`:

```ts
function providerKind(kind: string): ProviderKind {
  if (kind === "delyx_local" || kind === "ollama" || kind === "openai_compatible") {
    return kind;
  }
  return "unavailable";
}
```

Change `runtimeProviderView` detail:

```ts
const unsupportedOpenAi = provider.kind === "openai_compatible";
return {
  detail: unsupportedOpenAi
    ? "OpenAI-compatible calls are not wired yet. Use Delyx Local or Ollama for live local calls."
    : provider.message,
  id: provider.id,
  kind: unsupportedOpenAi ? "unavailable" : providerKind(provider.kind),
  label: provider.label,
  models: provider.models,
  requiresSecret: false,
  status: unsupportedOpenAi ? "not_configured" : providerStatus(provider.status),
  version: provider.version,
};
```

Update `providerStatus`:

```ts
function providerStatus(status: string): ProviderStatus {
  if (["failed", "loading", "missing_key", "model_missing", "not_configured", "ready", "unreachable"].includes(status)) {
    return status as ProviderStatus;
  }
  return "not_configured";
}
```

Update `modelData.ts` so Delyx Local appears first:

```ts
export const currentModelSettings: ModelSettingsView = {
  selectedProviderId: "delyx-local",
  providers: [
    {
      detail: "No Delyx-managed local models imported yet.",
      id: "delyx-local",
      kind: "delyx_local",
      label: "Delyx Local",
      models: [],
      requiresSecret: false,
      status: "not_configured",
    },
    ...
  ],
  routes: [],
};
```

---

# PR D — Embedded `mistral.rs` runtime

## Files

```text
apps/desktop/src-tauri/Cargo.toml
apps/desktop/src-tauri/src/model_embedded.rs
apps/desktop/src-tauri/src/model_chat.rs
apps/desktop/src-tauri/src/lib.rs
apps/desktop/src-tauri/src/main.rs
```

## Cargo changes

Add async dependencies and feature-gated runtime:

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.32", features = ["bundled"] }
tauri = { version = "2.11.2", features = [] }
tauri-plugin-single-instance = "2.4.2"
tokio = { version = "1", features = ["sync", "rt-multi-thread", "macros"] }
futures = "0.3"

mistralrs = { version = "0.8.2", optional = true, default-features = false }

[features]
default = []
embedded_mistral = ["dep:mistralrs"]
embedded_mistral_cuda = ["embedded_mistral", "mistralrs/cuda"]
```

If `mistralrs` latest crate version differs, pin the exact version that compiles and record it in `docs/ARCHITECTURE.md`.

## Embedded runtime state

Create `model_embedded.rs`:

```rust
use crate::model_embedded_persistence::{mark_profile_status, LocalModelProfile};
use crate::model_chat::{ModelChatMessage, ModelChatResult};
use std::{collections::HashMap, path::Path, sync::Arc};
use tokio::sync::Mutex;

pub struct EmbeddedRuntimeState {
    loaded: Mutex<HashMap<String, Arc<LoadedLocalModel>>>,
}

pub struct LoadedLocalModel {
    pub profile_id: String,
    #[cfg(feature = "embedded_mistral")]
    pub model: mistralrs::Model,
}

impl EmbeddedRuntimeState {
    pub fn new() -> Self {
        Self {
            loaded: Mutex::new(HashMap::new()),
        }
    }

    pub async fn unload(&self, profile_id: &str) -> bool {
        self.loaded.lock().await.remove(profile_id).is_some()
    }
}

#[cfg(feature = "embedded_mistral")]
pub async fn send_embedded_chat(
    state: &EmbeddedRuntimeState,
    database_path: &Path,
    profile: LocalModelProfile,
    messages: Vec<ModelChatMessage>,
) -> Result<ModelChatResult, String> {
    use mistralrs::{GgufModelBuilder, PagedAttentionMetaBuilder, TextMessageRole, TextMessages};

    validate_profile(&profile)?;
    let model = load_or_get_model(state, database_path, &profile).await?;
    let request = text_messages(messages)?;
    let response = model
        .model
        .send_chat_request(request)
        .await
        .map_err(|error| {
            let message = format!("Local model chat failed: {error}");
            let _ = mark_profile_status(database_path, &profile.id, "failed", Some(&message));
            message
        })?;
    let text = response
        .choices
        .first()
        .and_then(|choice| choice.message.content.as_ref())
        .cloned()
        .unwrap_or_default()
        .trim()
        .to_string();
    if text.is_empty() {
        return Err("Local model returned an empty response.".to_string());
    }
    Ok(ModelChatResult {
        provider_id: "delyx-local".to_string(),
        model: profile.id,
        text,
    })
}

#[cfg(not(feature = "embedded_mistral"))]
pub async fn send_embedded_chat(
    _state: &EmbeddedRuntimeState,
    _database_path: &Path,
    _profile: LocalModelProfile,
    _messages: Vec<ModelChatMessage>,
) -> Result<ModelChatResult, String> {
    Err("Delyx embedded runtime was not compiled. Build with --features embedded_mistral.".to_string())
}

#[cfg(feature = "embedded_mistral")]
async fn load_or_get_model(
    state: &EmbeddedRuntimeState,
    database_path: &Path,
    profile: &LocalModelProfile,
) -> Result<Arc<LoadedLocalModel>, String> {
    if let Some(existing) = state.loaded.lock().await.get(&profile.id).cloned() {
        return Ok(existing);
    }

    mark_profile_status(database_path, &profile.id, "loading", None)?;

    let model_path = std::path::Path::new(&profile.model_path);
    let model_dir = model_path
        .parent()
        .ok_or_else(|| "Model path has no parent directory.".to_string())?
        .to_string_lossy()
        .to_string();
    let model_file = model_path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "Model path has no filename.".to_string())?
        .to_string();

    let mut builder = GgufModelBuilder::new(model_dir, vec![model_file]).with_logging();

    if let Some(template) = profile.chat_template_path.as_deref().filter(|value| !value.trim().is_empty()) {
        builder = builder.with_chat_template(template);
    }
    if let Some(tokenizer) = profile.tokenizer_path.as_deref().filter(|value| !value.trim().is_empty()) {
        builder = builder.with_tokenizer_json(tokenizer);
    }

    if let Ok(paged) = PagedAttentionMetaBuilder::default().build() {
        builder = builder.with_paged_attn(paged);
    }

    let model = builder.build().await.map_err(|error| {
        let message = format!("Failed to load local model `{}`: {error}", profile.display_name);
        let _ = mark_profile_status(database_path, &profile.id, "failed", Some(&message));
        message
    })?;

    let loaded = Arc::new(LoadedLocalModel {
        profile_id: profile.id.clone(),
        model,
    });
    state.loaded.lock().await.insert(profile.id.clone(), loaded.clone());
    mark_profile_status(database_path, &profile.id, "loaded", None)?;
    Ok(loaded)
}

fn validate_profile(profile: &LocalModelProfile) -> Result<(), String> {
    if profile.runtime != "mistralrs" {
        return Err(format!("Unsupported local runtime `{}`.", profile.runtime));
    }
    if profile.format != "gguf" {
        return Err(format!("Unsupported local model format `{}`.", profile.format));
    }
    if !std::path::Path::new(&profile.model_path).is_file() {
        return Err(format!("Model file is missing: {}", profile.model_path));
    }
    Ok(())
}

#[cfg(feature = "embedded_mistral")]
fn text_messages(messages: Vec<ModelChatMessage>) -> Result<mistralrs::TextMessages, String> {
    let mut request = mistralrs::TextMessages::new();
    for message in messages {
        let role = match message.role.as_str() {
            "assistant" => mistralrs::TextMessageRole::Assistant,
            "system" => mistralrs::TextMessageRole::System,
            "user" => mistralrs::TextMessageRole::User,
            other => return Err(format!("Unsupported message role `{other}`.")),
        };
        let content = message.content.trim();
        if content.is_empty() {
            return Err("Chat messages cannot be empty.".to_string());
        }
        request = request.add_message(role, content);
    }
    Ok(request)
}
```

Register in `lib.rs`:

```rust
pub mod model_embedded;
pub mod model_chat;
```

Register state in `main.rs`:

```rust
.manage(delyx_next_desktop::model_embedded::EmbeddedRuntimeState::new())
```

---

# PR E — Provider-aware chat command

## Files

```text
apps/desktop/src-tauri/src/model_chat.rs
apps/desktop/src-tauri/src/runtime_bridge.rs
apps/desktop/src-tauri/src/main.rs
apps/desktop/src/features/models/modelClient.ts
```

## Rust `model_chat.rs`

```rust
use crate::model_embedded::EmbeddedRuntimeState;
use crate::model_embedded_persistence::load_profile_from_path;
use crate::model_ollama::{send_ollama_chat, OllamaChatMessage};
use serde::{Deserialize, Serialize};
use std::{path::Path, time::Duration};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelChatResult {
    pub provider_id: String,
    pub model: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelChatRequest {
    pub provider_id: String,
    pub model: String,
    pub messages: Vec<ModelChatMessage>,
}

#[tauri::command]
pub async fn model_chat(
    runtime: tauri::State<'_, crate::runtime_bridge::RuntimeBridgeState>,
    embedded: tauri::State<'_, EmbeddedRuntimeState>,
    request: ModelChatRequest,
) -> Result<ModelChatResult, String> {
    send_model_chat(
        runtime.database_path(),
        &embedded,
        request.provider_id,
        request.model,
        request.messages,
    )
    .await
}

pub async fn send_model_chat(
    database_path: &Path,
    embedded: &EmbeddedRuntimeState,
    provider_id: String,
    model: String,
    messages: Vec<ModelChatMessage>,
) -> Result<ModelChatResult, String> {
    validate_messages(&messages)?;
    match provider_id.as_str() {
        "delyx-local" => {
            let profile = load_profile_from_path(database_path, &model)?;
            crate::model_embedded::send_embedded_chat(embedded, database_path, profile, messages).await
        }
        "ollama-local" => {
            let messages = messages.into_iter().map(|message| OllamaChatMessage {
                role: message.role,
                content: message.content,
            }).collect();
            let response = send_ollama_chat(model, messages, Duration::from_secs(120))?;
            Ok(ModelChatResult {
                provider_id: response.provider_id,
                model: response.model,
                text: response.text,
            })
        }
        other => Err(format!("Provider `{other}` is not supported for model chat.")),
    }
}

fn validate_messages(messages: &[ModelChatMessage]) -> Result<(), String> {
    if messages.is_empty() {
        return Err("Model chat requires at least one message.".to_string());
    }
    for message in messages {
        if !matches!(message.role.as_str(), "assistant" | "system" | "user") {
            return Err(format!("Unsupported message role `{}`.", message.role));
        }
        if message.content.trim().is_empty() {
            return Err("Model chat messages cannot be empty.".to_string());
        }
    }
    Ok(())
}
```

Register in `main.rs`:

```rust
delyx_next_desktop::model_chat::model_chat,
```

Keep old `ollama_chat` for compatibility for now. Remove later only after all call sites are migrated.

## Frontend client

Create `apps/desktop/src/features/models/modelClient.ts`:

```ts
import { invoke } from "@tauri-apps/api/core";
import type { ModelSettingsView, ThreadRoleMessage } from "./modelTypes";

export interface ModelChatResult {
  model: string;
  providerId: string;
  text: string;
}

export async function sendModelChat(settings: ModelSettingsView, messages: ThreadRoleMessage[]) {
  const route = selectedCodingRoute(settings);
  if (!route) {
    throw new Error("No ready model is selected. Import a Delyx Local model or select an available provider.");
  }
  if (!hasTauriRuntime()) {
    throw new Error("Model chat requires the Delyx desktop runtime.");
  }
  return invoke<ModelChatResult>("model_chat", {
    request: {
      providerId: route.providerId,
      model: route.modelId,
      messages,
    },
  });
}

export function selectedCodingRoute(settings: ModelSettingsView) {
  const route = settings.routes.find((item) => item.role === "coding");
  if (route && providerReadyForModel(settings, route.providerId, route.modelId)) {
    return route;
  }
  const provider = settings.providers.find((item) => item.status === "ready" && item.models.length > 0);
  if (!provider) {
    return undefined;
  }
  return { providerId: provider.id, modelId: provider.models[0], role: "coding" as const, saved: false };
}

function providerReadyForModel(settings: ModelSettingsView, providerId: string, modelId: string) {
  const provider = settings.providers.find((item) => item.id === providerId);
  return Boolean(provider && provider.status === "ready" && provider.models.includes(modelId));
}

function hasTauriRuntime() {
  return Boolean((globalThis as Record<string, unknown>).__TAURI_INTERNALS__);
}
```

---

# PR F — Composer uses provider-aware model chat

## Files

```text
apps/desktop/src/app/cockpitComposerBindings.ts
apps/desktop/src/features/models/ollamaClient.ts
```

Update imports:

```ts
import { sendModelChat, selectedCodingRoute } from "../features/models/modelClient";
```

Rename `requestOllamaReply` to `requestModelReply`.

Replace the Ollama-only branch:

```ts
async function requestModelReplyInner(state: ComposerBindingState, thread: TaskThread) {
  const route = selectedCodingRoute(state.modelSettings);
  markThread(state, thread.id, "exploring");
  state.setAgentRuns((current) => updateRunsForThreadStatus(current, thread, "exploring", new Date().toISOString()));
  if (!route) {
    recordModelFailure(state, thread, "no-model", "No ready model is selected. Import a Delyx Local model or select Ollama/CLI.");
    return;
  }
  try {
    state.setAgentRuns((current) => recordModelCallStarted(current, thread, route.modelId, new Date().toISOString()));
    const response = await sendModelChat(state.modelSettings, modelMessages(thread));
    const now = new Date().toISOString();
    appendMessage(state, thread.id, { role: "assistant", body: response.text }, "idle");
    state.setAgentRuns((current) => recordModelCallResult(current, thread, response.providerId, response.model, response.text, now));
    notifyLocalAction(`${providerLabel(response.providerId)} replied with ${response.model}`, "success");
    if (state.qaqcAdapterId) {
      void runQaqcReview(state, thread, response.text);
    }
  } catch (error) {
    recordModelFailure(state, thread, route.modelId, error instanceof Error ? error.message : "Model request failed.");
  }
}

function providerLabel(providerId: string) {
  if (providerId === "delyx-local") return "Delyx Local";
  if (providerId === "ollama-local") return "Ollama";
  return providerId;
}
```

Keep `sendOllamaChat` in `ollamaClient.ts` until nothing imports it except compatibility tests.

---

# PR G — PatchDraft uses provider-aware model chat

## Files

```text
apps/desktop/src-tauri/src/agent_patch_draft_bridge.rs
apps/desktop/src-tauri/src/agent_patch_draft_context.rs
apps/desktop/src-tauri/src/agent_patch_draft_dispatch.rs
apps/desktop/src-tauri/src/agent_patch_draft_step.rs
apps/desktop/src-tauri/src/main.rs
apps/desktop/src/features/runs/agentExecutorClient.ts
apps/desktop/src/app/appShellOllamaPatchActions.ts
```

## Rust request types

Add `provider_id` to `AgentPatchDraftExecuteRequest`:

```rust
pub struct AgentPatchDraftExecuteRequest {
    pub client_id: String,
    pub run_id: String,
    pub approval_id: String,
    pub approved_roots: Vec<String>,
    pub project_path: String,
    pub provider_id: String,
    pub model: String,
    pub goal: String,
    pub plan_steps: Vec<String>,
    pub files_likely_involved: Vec<String>,
    pub scope_paths: Vec<String>,
    pub created_at_ms: u64,
    pub max_bytes_per_file: Option<usize>,
}
```

Add `provider_id` to:

```text
AgentPatchDraftContextRequest
AgentPatchDraftStepRequest
```

Then pass it through `context_execute_request`:

```rust
provider_id: request.provider_id.clone(),
model: request.model.clone(),
```

## Rust command state

Update patch-draft Tauri commands to receive embedded runtime and runtime database state:

```rust
embedded: tauri::State<'_, crate::model_embedded::EmbeddedRuntimeState>,
runtime: tauri::State<'_, crate::runtime_bridge::RuntimeBridgeState>,
```

Update `execute_patch_draft_record` signature:

```rust
pub(crate) async fn execute_patch_draft_record(
    threads: &ThreadRunBridgeState,
    patches: &PatchBridgeState,
    approvals: &ApprovalBridgeState,
    embedded: &crate::model_embedded::EmbeddedRuntimeState,
    database_path: &std::path::Path,
    request: AgentPatchDraftExecuteRequest,
) -> Result<AgentPatchDraftBridgeView, String>
```

Because this becomes async, the Tauri commands that call it must also become `async fn` and use `.await`.

Replace:

```rust
send_ollama_chat(request.model.clone(), messages, Duration::from_secs(120))
```

with:

```rust
let model_messages = messages
    .into_iter()
    .map(|message| crate::model_chat::ModelChatMessage {
        role: message.role,
        content: message.content,
    })
    .collect();

match crate::model_chat::send_model_chat(
    database_path,
    embedded,
    request.provider_id.clone(),
    request.model.clone(),
    model_messages,
).await {
    Ok(response) => { ... }
    Err(error) => { ... }
}
```

Keep `AgentPatchDraftBridgeView` returning `provider_id` and `model`.

## Frontend request types

In `agentExecutorClient.ts` add `providerId`:

```ts
export interface AgentPatchDraftExecuteRequest {
  approvalId: string;
  approvedRoots: string[];
  clientId: string;
  createdAtMs: number;
  filesLikelyInvolved: string[];
  goal: string;
  maxBytesPerFile?: number;
  providerId: string;
  model: string;
  planSteps: string[];
  projectPath: string;
  runId: string;
  scopePaths: string[];
}

export interface AgentPatchDraftContextRequest {
  approvalId: string;
  hasSupportedTestCommand: boolean;
  maxBytesPerFile?: number;
  providerId: string;
  model: string;
  nowMs: number;
  projectId: string;
  runId: string;
  testApprovalId?: string;
}

export interface AgentPatchDraftStepRequest {
  maxBytesPerFile?: number;
  providerId: string;
  model: string;
  nowMs: number;
  projectId: string;
  runId: string;
}
```

In `appShellOllamaPatchActions.ts`, rename to `appShellPatchDraftActions.ts` eventually. First, do the minimum change:

```ts
import { selectedCodingRoute } from "../features/models/modelClient";
```

Replace:

```ts
const model = selectedOllamaModel(state.modelSettings);
```

with:

```ts
const route = selectedCodingRoute(state.modelSettings);
if (!route) {
  recordPatchDraftFailure(state, thread, "no-model", "No ready model is selected to draft a patch.");
  return { created: false };
}
```

Then send:

```ts
const result = await runPatchDraftSchedulerStepOverBridge({
  maxBytesPerFile: 20_000,
  providerId: route.providerId,
  model: route.modelId,
  nowMs: createdAtMs,
  projectId: state.activeProject.id,
  runId: run.id,
});
```

Update UI messages:

```ts
appendMessage(
  state,
  thread.id,
  { role: "system", body: `PatchDraftAgent is drafting with ${route.providerId}/${route.modelId}.` },
  "building",
);
```

## Tests for PR G

Add tests proving:

- PatchDraft step rejects missing `provider_id`.
- PatchDraft step preserves `provider_id` from frontend request into model dispatch.
- Existing Ollama PatchDraft still works when provider is `ollama-local`.
- Delyx local PatchDraft calls `send_model_chat` with `delyx-local`.

---

# PR H — Local model lifecycle commands

## Files

```text
apps/desktop/src-tauri/src/model_embedded_bridge.rs
apps/desktop/src-tauri/src/model_embedded.rs
apps/desktop/src-tauri/src/model_embedded_persistence.rs
apps/desktop/src-tauri/src/main.rs
apps/desktop/src/features/models/localModelClient.ts
```

Create `model_embedded_bridge.rs`:

```rust
use crate::model_embedded::EmbeddedRuntimeState;
use crate::model_embedded_persistence::{
    delete_profile_from_path, import_profile_to_path, list_profiles_from_path,
    ImportLocalModelRequest, LocalModelProfile,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalModelLifecycleView {
    pub status: String,
    pub message: String,
    pub profile: Option<LocalModelProfile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalModelIdRequest {
    pub id: String,
}

#[tauri::command]
pub fn local_model_import(
    runtime: tauri::State<crate::runtime_bridge::RuntimeBridgeState>,
    request: ImportLocalModelRequest,
) -> Result<LocalModelLifecycleView, String> {
    let profile = import_profile_to_path(runtime.database_path(), request)?;
    Ok(LocalModelLifecycleView {
        status: "imported".to_string(),
        message: format!("Imported {}.", profile.display_name),
        profile: Some(profile),
    })
}

#[tauri::command]
pub fn local_model_list(
    runtime: tauri::State<crate::runtime_bridge::RuntimeBridgeState>,
) -> Result<Vec<LocalModelProfile>, String> {
    list_profiles_from_path(runtime.database_path())
}

#[tauri::command]
pub async fn local_model_unload(
    embedded: tauri::State<'_, EmbeddedRuntimeState>,
    request: LocalModelIdRequest,
) -> Result<LocalModelLifecycleView, String> {
    let removed = embedded.unload(&request.id).await;
    Ok(LocalModelLifecycleView {
        status: if removed { "unloaded" } else { "not_loaded" }.to_string(),
        message: if removed {
            format!("Unloaded {} from memory.", request.id)
        } else {
            format!("{} was not loaded.", request.id)
        },
        profile: None,
    })
}

#[tauri::command]
pub async fn local_model_remove_profile(
    runtime: tauri::State<'_, crate::runtime_bridge::RuntimeBridgeState>,
    embedded: tauri::State<'_, EmbeddedRuntimeState>,
    request: LocalModelIdRequest,
) -> Result<LocalModelLifecycleView, String> {
    embedded.unload(&request.id).await;
    delete_profile_from_path(runtime.database_path(), &request.id)?;
    Ok(LocalModelLifecycleView {
        status: "removed".to_string(),
        message: format!("Removed model profile {}. The model file was not deleted.", request.id),
        profile: None,
    })
}
```

Register module:

```rust
pub mod model_embedded_bridge;
```

Register commands:

```rust
delyx_next_desktop::model_embedded_bridge::local_model_import,
delyx_next_desktop::model_embedded_bridge::local_model_list,
delyx_next_desktop::model_embedded_bridge::local_model_unload,
delyx_next_desktop::model_embedded_bridge::local_model_remove_profile,
```

Frontend client:

```ts
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
}

export interface ImportLocalModelRequest {
  modelPath: string;
  displayName?: string;
  chatTemplatePath?: string;
  tokenizerPath?: string;
  contextWindow?: number;
}

export function importLocalModel(request: ImportLocalModelRequest) {
  return invoke("local_model_import", { request });
}

export function listLocalModels() {
  return invoke<LocalModelProfile[]>("local_model_list");
}

export function unloadLocalModel(id: string) {
  return invoke("local_model_unload", { request: { id } });
}

export function removeLocalModelProfile(id: string) {
  return invoke("local_model_remove_profile", { request: { id } });
}
```

---

# PR I — UI model management surface

## Minimal path-input UI first

To avoid adding another dependency immediately, add a simple Settings panel section with:

- Model file path input
- Display name input
- Context window input
- Optional chat template path input
- Import button
- Refresh runtime button
- List imported models
- Unload button
- Remove profile button

Later, add a native file picker with `tauri-plugin-dialog`.

## Files

```text
apps/desktop/src/app/FocusSettings.tsx
apps/desktop/src/features/models/localModelClient.ts
apps/desktop/src/app/AppShell.tsx
apps/desktop/src/styles/focus-surfaces.css or model-specific CSS file
```

Add a local model management component, but keep the file under the repo line-budget rule. If `FocusSettings.tsx` is already large, extract:

```text
apps/desktop/src/app/LocalModelSettingsPanel.tsx
```

UI states required:

```text
empty          No Delyx local models imported
importing      Import request running
ready          Imported model profiles shown
loading        Model runtime loading during first chat
failed         Last model error visible
unloaded       Imported but not in memory
removed        Profile removed; file not deleted
```

## Native file picker later

If you want a native picker, add:

```toml
tauri-plugin-dialog = "2"
```

Register plugin in `main.rs`:

```rust
.plugin(tauri_plugin_dialog::init())
```

Frontend:

```ts
import { open } from "@tauri-apps/plugin-dialog";

const selected = await open({
  multiple: false,
  filters: [{ name: "GGUF models", extensions: ["gguf"] }],
});
```

This is not required for the first complete embedded runtime. A text path import is enough to finish the direct model path.

---

# PR J — Docs and architecture updates

## Files

```text
docs/ARCHITECTURE.md
README.md
DELYX_NEXT_UI_FIRST_CODEX_BUILD_PLAN.md
apps/desktop/scripts/verify-*.mjs if any script still assumes Ollama-only wording
```

Add to `docs/ARCHITECTURE.md`:

```md
## Embedded local model runtime

Delyx Next now has a first-class Delyx Local provider. The provider runs local models in-process through the feature-gated `mistral.rs` Rust SDK. This avoids requiring Ollama, LM Studio, llama.cpp server, or other user-managed runtime apps for the default local path.

Provider policy:

- `delyx-local`: first-class local embedded provider.
- `ollama-local`: optional convenience adapter.
- `openai-compatible`: optional HTTP adapter.
- CLI providers: optional external agent/chat adapters with explicit off-device behavior.

The first embedded milestone supports local GGUF profiles. SQLite stores profile metadata and paths only; model weight files remain on disk outside SQLite.
```

Update README:

```md
## Local models

Delyx Next can run Delyx-managed local GGUF models directly through the desktop runtime. Ollama is optional.
```

Update the build plan current reality:

- Replace “Ollama is the only real live model execution path” with the new truth after merged.
- Add a Phase 2 D7 completion item for embedded local model runtime.

---

# 4. Validation commands

Run before changes:

```powershell
.\.tools\npm.cmd run typecheck
.\.tools\npm.cmd test
cargo test --workspace
```

Run after PR A-C:

```powershell
.\.tools\npm.cmd run typecheck
.\.tools\npm.cmd test
cargo test --workspace
```

Run after PR D-G:

```powershell
cargo test --workspace
cargo test --workspace --features embedded_mistral
.\.tools\npm.cmd run typecheck
.\.tools\npm.cmd test
.\.tools\npm.cmd run build
```

Run desktop:

```powershell
.\.tools\npm.cmd run dev:desktop
```

Package smoke:

```powershell
.\.tools\npm.cmd run package:windows
.\.tools\npm.cmd run smoke:tauri
```

---

# 5. Manual acceptance test

Use a small GGUF first. Do not start with a huge 30B model.

Recommended first test models:

```text
Qwen2.5-Coder-7B-Instruct Q4_K_M GGUF
Qwen3-4B Q4_K_M GGUF
Phi-4-mini GGUF
```

Manual test steps:

1. Start Delyx Next desktop.
2. Go to Settings -> Models.
3. Import a local `.gguf` file.
4. Confirm runtime status shows `Delyx Local` ready.
5. Open model picker.
6. Select the Delyx Local model.
7. Send normal composer message:

```text
Reply with exactly: Delyx local model is working.
```

8. Confirm response arrives without Ollama running.
9. Kill/stop Ollama if it is running.
10. Send another message.
11. Confirm Delyx Local still works.
12. Create a coding plan.
13. Approve scoped PatchDraft.
14. Confirm PatchDraft uses `delyx-local/<model>` in receipts.
15. Switch to Ollama provider, if installed.
16. Confirm Ollama still works.
17. Remove the Delyx profile.
18. Confirm the model file still exists on disk.

---

# 6. Common failure states and exact UI text

Use these messages so failures stay truthful:

```text
No embedded runtime compiled
Delyx embedded runtime was not compiled. Build with --features embedded_mistral.

No models imported
No Delyx-managed local models imported yet.

Missing model file
The model profile exists, but the model file is missing: <path>

Unsupported format
Only .gguf local model files are supported in this milestone.

Load failed
Delyx Local could not load <model>. See the model details for the error.

Out of memory
The local model did not fit available memory. Try a smaller quantized GGUF.

Fallback available
Delyx Local is unavailable. Select Ollama, CLI, or OpenAI-compatible provider if configured.
```

---

# 7. Do-not-do list

Do **not**:

- Require Ollama for the default Delyx Local path.
- Start a llama.cpp server as the “direct” path.
- Delete model files when removing profiles.
- Store model weights in SQLite.
- Hide model load failures.
- Make PatchDraft use a different model path than normal chat.
- Keep `modelId`-only selection after adding Delyx Local.
- Mark done if only runtime status works but chat does not.
- Mark done if composer works but PatchDraft still calls `send_ollama_chat`.
- Break CLI providers.

---

# 8. Final cleanup after everything passes

After Delyx Local works:

1. Rename `appShellOllamaPatchActions.ts` to `appShellPatchDraftActions.ts`.
2. Rename `requestOllamaReply` to `requestModelReply` everywhere.
3. Keep `ollamaClient.ts` only as an adapter.
4. Keep `ollama_chat` command for compatibility until no frontend call uses it.
5. Add a short Architecture Decision Record:

```text
docs/adr/0001-embedded-local-model-runtime.md
```

ADR content:

```md
# ADR 0001 — Embedded local model runtime

## Decision

Delyx Next uses a first-class embedded local runtime through `mistral.rs` for Delyx-managed local GGUF models. Ollama remains an optional adapter.

## Why

Users should not need to install or manage another model application for the default local path. Delyx must own model lifecycle, routing, runtime states, and receipts.

## Consequences

The dependency is broad, so it is feature-gated. CUDA builds are separate. Model profiles are persisted in SQLite, but model files remain on disk.
```

---

# 9. One-shot Codex prompt

Use this prompt inside the repo when you want Codex to execute the work in order:

```text
Implement the Delyx Next embedded local model runtime exactly as described in DELYX_NEXT_EMBEDDED_MODEL_RUNTIME_100_PERCENT_PLAN.md.

Rules:
- Do not remove Ollama; keep it as an optional adapter.
- Add Delyx Local as the first-class embedded provider.
- Use provider/model pair selection, not model-only selection.
- Store model profiles in SQLite, not model weights.
- Add feature-gated mistral.rs support behind embedded_mistral.
- Implement provider-aware model_chat and route composer + PatchDraft through it.
- Keep all risky file writes approval-gated.
- Keep source files under the AGENTS.md line-budget by extracting modules.
- Add deterministic tests for persistence, runtime status, route selection, and PatchDraft provider propagation.
- Update docs and build plan truthfully.
- Run npm typecheck, npm test, npm build, cargo test --workspace, and cargo test --workspace --features embedded_mistral.
```

---

# 10. Completion checklist

Mark these only when actually true:

```md
- [x] ProviderKind has DelyxLocal / `delyx_local`.
- [x] ProviderStatus supports loading/model_missing/failed.
- [x] ModelInfo carries runtime/format/path metadata.
- [x] Frontend ProviderKind supports `delyx_local`.
- [x] Model picker passes providerId + modelId.
- [x] SQLite has local_model_profiles.
- [x] Local model profile import/list/delete works.
- [x] runtime_status lists Delyx Local profiles.
- [x] default coding route prefers Delyx Local, then Ollama.
- [x] mistral.rs is feature-gated.
- [x] EmbeddedRuntimeState loads/unloads models.
- [x] `model_chat` command dispatches Delyx Local and Ollama.
- [x] Composer uses `sendModelChat`, not `sendOllamaChat`.
- [ ] PatchDraft uses `send_model_chat`, not `send_ollama_chat`. (PR G — async migration still open.)
- [x] UI shows no imported model / ready / loading / failed / unloaded states.
- [x] Removing profile does not delete model file.
- [x] Ollama adapter still works.
- [x] CLI providers still work.
- [x] Tests cover local profile persistence.
- [ ] Tests cover runtime status Delyx Local provider. (Registration compiles + is exercised, but no dedicated test yet.)
- [x] Tests cover provider-aware model selection.
- [ ] Tests cover PatchDraft provider propagation. (PR G.)
- [x] README updated.
- [x] docs/ARCHITECTURE.md updated.
- [x] build plan current reality updated.
- [x] npm typecheck passes.
- [x] npm test passes.
- [x] npm build passes.
- [x] cargo test --workspace passes.
- [x] cargo test --workspace --features embedded_mistral passes.
- [ ] Desktop manual test works with Ollama stopped. (Needs a build with --features embedded_mistral + a real GGUF on your machine.)
```

Status: PRs A–F and H–I are landed and green (chat works end-to-end through the
embedded runtime). PR G (PatchDraft via `model_chat`, an async migration) and PR
J's remaining doc/test tail are the only open items. When all boxes are checked,
Delyx Next has a real direct local model path for both chat and PatchDraft.
