import { useEffect, useState } from "react";
import {
  importLocalModel,
  listLocalModels,
  removeLocalModelProfile,
  unloadLocalModel,
  type LocalModelProfile,
} from "../features/models/localModelClient";

export function LocalModelSettingsPanel() {
  const [profiles, setProfiles] = useState<LocalModelProfile[] | undefined>(undefined);
  const [path, setPath] = useState("");
  const [name, setName] = useState("");
  const [context, setContext] = useState("");
  const [status, setStatus] = useState<string | undefined>(undefined);
  const [busy, setBusy] = useState(false);
  const [desktopOnly, setDesktopOnly] = useState(false);

  async function refresh() {
    try {
      setProfiles(await listLocalModels());
    } catch {
      setDesktopOnly(true);
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
        <input aria-label="Model file path" className="pal-input" onChange={(event) => setPath(event.target.value)} placeholder="C:\\models\\your-model.Q4_K_M.gguf" value={path} />
      </Row>
      <Row detail="Optional display name and context window." title="Details">
        <input aria-label="Display name" className="pal-input" onChange={(event) => setName(event.target.value)} placeholder="Display name (optional)" value={name} />
        <input aria-label="Context window" className="pal-input" onChange={(event) => setContext(event.target.value)} placeholder="ctx (e.g. 8192)" value={context} />
        <button className="select" disabled={busy || !path.trim()} onClick={() => void doImport()} type="button">{busy ? "Importing…" : "Import"}</button>
      </Row>
      {status && <Row detail={status} title="Status"><span className="tag live">ok</span></Row>}
      {profiles && profiles.length === 0 && <Row detail="No Delyx-managed local models imported yet." title="No models"><span className="tag off">empty</span></Row>}
      {(profiles ?? []).map((profile) => (
        <Row detail={`${profile.format} · ${profile.modelPath}${profile.lastError ? ` · error: ${profile.lastError}` : ""}`} key={profile.id} title={profile.displayName}>
          <span className={`tag ${profile.loadStatus === "failed" ? "off" : "live"}`}>{profile.loadStatus}</span>
          <button className="select" onClick={() => void unload(profile.id)} type="button">Unload</button>
          <button className="select" onClick={() => void remove(profile.id)} type="button">Remove</button>
        </Row>
      ))}
    </div>
  );
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
