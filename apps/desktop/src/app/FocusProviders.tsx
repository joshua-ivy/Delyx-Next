import { useEffect, useState } from "react";
import { loadExternalAgentStatus } from "../features/externalAgents/externalAgentClient";
import type { ExternalAgentAdapterView } from "../features/externalAgents/externalAgentTypes";
import { LocalModelSettingsPanel } from "./LocalModelSettingsPanel";
import { clearSecret, loadSecretStatus, setSecret, type SecretProviderView } from "./secretClient";

const CLI_SETUP: Record<string, { install: string; login: string }> = {
  "claude-code": { install: "npm i -g @anthropic-ai/claude-code", login: "claude login" },
  "codex-cli": { install: "npm i -g @openai/codex", login: "codex login" },
};

export function FocusProviders() {
  const [adapters, setAdapters] = useState<ExternalAgentAdapterView[] | undefined>(undefined);
  const [providers, setProviders] = useState<SecretProviderView[] | undefined>(undefined);
  const [drafts, setDrafts] = useState<Record<string, string>>({});
  const [error, setError] = useState<string | undefined>(undefined);
  const [desktopOnly, setDesktopOnly] = useState(false);

  useEffect(() => {
    let active = true;
    void (async () => {
      const status = await loadSecretStatus();
      if (!active) return;
      if (!status) {
        setDesktopOnly(true);
        return;
      }
      setProviders(status.providers);
      try {
        const cli = await loadExternalAgentStatus();
        if (active) setAdapters(cli.adapters);
      } catch {
        if (active) setAdapters([]);
      }
    })();
    return () => {
      active = false;
    };
  }, []);

  async function save(id: string) {
    const value = (drafts[id] ?? "").trim();
    if (!value) return;
    try {
      const status = await setSecret(id, value);
      setProviders(status.providers);
      setDrafts((current) => ({ ...current, [id]: "" }));
      setError(undefined);
    } catch (cause) {
      setError(String(cause));
    }
  }

  async function remove(id: string) {
    try {
      setProviders((await clearSecret(id)).providers);
      setError(undefined);
    } catch (cause) {
      setError(String(cause));
    }
  }

  if (desktopOnly) {
    return (
      <div className="set-sec">
        <div className="ey">Providers &amp; keys</div>
        <Row detail="Provider keys and CLI detection need the Delyx desktop app; the web preview has no secure key store." title="Desktop only">
          <span className="tag off">web preview</span>
        </Row>
      </div>
    );
  }

  return (
    <>
      <LocalModelSettingsPanel />
      <div className="set-sec">
        <div className="ey">Agent CLIs</div>
        {(adapters ?? []).filter((adapter) => CLI_SETUP[adapter.id]).map((adapter) => (
          <CliRow adapter={adapter} key={adapter.id} />
        ))}
        {adapters === undefined && <Row detail="" title="Checking PATH…"><span className="tag off">…</span></Row>}
      </div>
      <div className="set-sec">
        <div className="ey">Provider API keys</div>
        <div className="set-lead">Keys are stored in your OS keyring on this device — never in the repo or a settings file. Cloud calls send data off-device.</div>
        {(providers ?? []).map((provider) => (
          <KeyRow
            draft={drafts[provider.id] ?? ""}
            key={provider.id}
            onChange={(value) => setDrafts((current) => ({ ...current, [provider.id]: value }))}
            onClear={() => void remove(provider.id)}
            onSave={() => void save(provider.id)}
            provider={provider}
          />
        ))}
        {error && (
          <Row detail={error} title="Could not update key">
            <span className="tag off">error</span>
          </Row>
        )}
      </div>
    </>
  );
}

function CliRow({ adapter }: { adapter: ExternalAgentAdapterView }) {
  const setup = CLI_SETUP[adapter.id];
  const available = adapter.status === "available";
  const detail = available ? adapter.detail : `Install: ${setup.install}  —  then: ${setup.login}`;
  return (
    <Row detail={detail} title={adapter.label}>
      <span className={`tag ${available ? "live" : "off"}`}>{available ? "detected" : "not installed"}</span>
    </Row>
  );
}

function KeyRow(props: {
  draft: string;
  onChange: (value: string) => void;
  onClear: () => void;
  onSave: () => void;
  provider: SecretProviderView;
}) {
  return (
    <Row
      detail={props.provider.hasKey ? "A key is saved for this provider." : "No key saved."}
      title={props.provider.label}
    >
      <input
        aria-label={`${props.provider.label} API key`}
        className="set-input"
        onChange={(event) => props.onChange(event.target.value)}
        placeholder={props.provider.hasKey ? "Replace key…" : "Paste API key…"}
        type="password"
        value={props.draft}
      />
      <button className="select" disabled={!props.draft.trim()} onClick={props.onSave} type="button">Save</button>
      {props.provider.hasKey && (
        <button className="select" onClick={props.onClear} type="button">Clear</button>
      )}
      <span className={`tag ${props.provider.hasKey ? "live" : "off"}`}>{props.provider.hasKey ? "set" : "not set"}</span>
    </Row>
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
