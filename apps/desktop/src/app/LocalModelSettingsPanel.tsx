import { Fragment, useEffect, useState } from "react";
import {
  importLocalModel,
  importOllamaModel,
  listLocalModels,
  listOllamaModels,
  removeLocalModelProfile,
  setLocalModelSampling,
  unloadLocalModel,
  type LocalModelProfile,
  type ModelSamplingRequest,
  type OllamaModelEntry,
} from "../features/models/localModelClient";

export function LocalModelSettingsPanel() {
  const [profiles, setProfiles] = useState<LocalModelProfile[] | undefined>(undefined);
  const [path, setPath] = useState("");
  const [name, setName] = useState("");
  const [context, setContext] = useState("");
  const [status, setStatus] = useState<string | undefined>(undefined);
  const [busy, setBusy] = useState(false);
  const [desktopOnly, setDesktopOnly] = useState(false);
  const [ollama, setOllama] = useState<OllamaModelEntry[]>([]);

  async function refresh() {
    try {
      setProfiles(await listLocalModels());
    } catch {
      setDesktopOnly(true);
      return;
    }
    try {
      setOllama(await listOllamaModels());
    } catch {
      setOllama([]);
    }
  }

  async function importFromOllama(name: string) {
    try {
      setStatus((await importOllamaModel(name)).message);
      await refresh();
    } catch (cause) {
      setStatus(String(cause));
    }
  }

  useEffect(() => {
    void refresh();
  }, []);

  async function doImport() {
    const modelPath = path.trim();
    if (!modelPath) {
      return;
    }
    setBusy(true);
    try {
      const result = await importLocalModel({
        chatTemplatePath: undefined,
        contextWindow: context.trim() ? Number(context) : undefined,
        displayName: name.trim() || undefined,
        modelPath,
      });
      setStatus(result.message);
      setPath("");
      setName("");
      setContext("");
      await refresh();
    } catch (cause) {
      setStatus(String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function unload(id: string) {
    setStatus((await unloadLocalModel(id)).message);
    await refresh();
  }

  async function remove(id: string) {
    setStatus((await removeLocalModelProfile(id)).message);
    await refresh();
  }

  async function saveSampling(request: ModelSamplingRequest) {
    try {
      setStatus((await setLocalModelSampling(request)).message);
      await refresh();
    } catch (cause) {
      setStatus(String(cause));
    }
  }

  if (desktopOnly) {
    return (
      <div className="set-sec">
        <div className="ey">Delyx Local models</div>
        <Row detail="Importing local models needs the Delyx desktop app." title="Desktop only">
          <span className="tag off">web preview</span>
        </Row>
      </div>
    );
  }

  return (
    <div className="set-sec">
      <div className="ey">Delyx Local models</div>
      <div className="set-lead">Import a local .gguf file to run it in-process — no Ollama required. Weights stay on disk; Delyx stores only the path. Removing a model never deletes the file.</div>
      <Row detail="Absolute path to a .gguf model file on this machine." title="Model file path">
        <input aria-label="Model file path" className="set-input" onChange={(event) => setPath(event.target.value)} placeholder="C:\\models\\your-model.Q4_K_M.gguf" value={path} />
      </Row>
      <Row detail="Optional display name and context window." title="Details">
        <input aria-label="Display name" className="set-input" onChange={(event) => setName(event.target.value)} placeholder="Display name (optional)" value={name} />
        <input aria-label="Context window" className="set-input" onChange={(event) => setContext(event.target.value)} placeholder="ctx (e.g. 8192)" value={context} />
        <button className="select" disabled={busy || !path.trim()} onClick={() => void doImport()} type="button">{busy ? "Importing…" : "Import"}</button>
      </Row>
      {status && <Row detail={status} title="Status"><span className="tag live">ok</span></Row>}
      {profiles && profiles.length === 0 && <Row detail="No Delyx-managed local models imported yet." title="No models"><span className="tag off">empty</span></Row>}
      {(profiles ?? []).map((profile) => (
        <Fragment key={profile.id}>
          <Row detail={`${profile.format} · ${profile.modelPath}${profile.lastError ? ` · error: ${profile.lastError}` : ""}`} title={profile.displayName}>
            <span className={`tag ${profile.loadStatus === "failed" ? "off" : "live"}`}>{profile.loadStatus}</span>
            <button className="select" onClick={() => void unload(profile.id)} type="button">Unload</button>
            <button className="select" onClick={() => void remove(profile.id)} type="button">Remove</button>
          </Row>
          <SamplingEditor onSave={saveSampling} profile={profile} />
        </Fragment>
      ))}
      {ollama.length > 0 && (
        <>
          <div className="ey">Reuse from Ollama</div>
          <div className="set-lead">Models you already pulled with Ollama — import to run them in the embedded runtime, no re-download.</div>
          {ollama.map((entry) => (
            <Row detail={entry.blobPath} key={entry.name} title={entry.name}>
              <button className="select" onClick={() => void importFromOllama(entry.name)} type="button">Import</button>
            </Row>
          ))}
        </>
      )}
    </div>
  );
}

function SamplingEditor({ onSave, profile }: { onSave: (request: ModelSamplingRequest) => void; profile: LocalModelProfile }) {
  const [temperature, setTemperature] = useState(numText(profile.temperature));
  const [topP, setTopP] = useState(numText(profile.topP));
  const [topK, setTopK] = useState(numText(profile.topK));
  const [repeatPenalty, setRepeatPenalty] = useState(numText(profile.repeatPenalty));
  const [maxTokens, setMaxTokens] = useState(numText(profile.maxTokens));
  return (
    <Row detail="Tune sampling for this model. Blank = model default. Applies to chat and PatchDraft." title="Sampling">
      <input aria-label={`${profile.id} temperature`} className="set-input" onChange={(event) => setTemperature(event.target.value)} placeholder="temp" value={temperature} />
      <input aria-label={`${profile.id} top_p`} className="set-input" onChange={(event) => setTopP(event.target.value)} placeholder="top_p" value={topP} />
      <input aria-label={`${profile.id} top_k`} className="set-input" onChange={(event) => setTopK(event.target.value)} placeholder="top_k" value={topK} />
      <input aria-label={`${profile.id} repeat_penalty`} className="set-input" onChange={(event) => setRepeatPenalty(event.target.value)} placeholder="rep" value={repeatPenalty} />
      <input aria-label={`${profile.id} max_tokens`} className="set-input" onChange={(event) => setMaxTokens(event.target.value)} placeholder="max" value={maxTokens} />
      <button className="select" onClick={() => onSave({ id: profile.id, maxTokens: numInt(maxTokens), repeatPenalty: num(repeatPenalty), temperature: num(temperature), topK: numInt(topK), topP: num(topP) })} type="button">Save sampling</button>
    </Row>
  );
}

function numText(value: number | null | undefined): string {
  return value === undefined || value === null ? "" : String(value);
}

function num(value: string): number | undefined {
  const trimmed = value.trim();
  return trimmed === "" || Number.isNaN(Number(trimmed)) ? undefined : Number(trimmed);
}

function numInt(value: string): number | undefined {
  const parsed = num(value);
  return parsed === undefined ? undefined : Math.trunc(parsed);
}

function Row({ children, detail, title }: { children: React.ReactNode; detail: string; title: string }) {
  return (
    <div className="row">
      <div className="rmeta">
        <b>{title}</b>
        {detail && <span>{detail}</span>}
      </div>
      <div className="rctl">{children}</div>
    </div>
  );
}
