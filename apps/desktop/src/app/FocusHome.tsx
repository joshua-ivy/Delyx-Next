import { useState } from "react";
import type { ModelSettingsView } from "../features/models/modelTypes";
import type { WorkspaceProject } from "../features/workspace/workspaceTypes";
import { FocusIcon, Pipe } from "./focusAtoms";
import { focusModes, modeLabel, modeStep, repoLabel, selectedModel, selectedProvider, type FocusMode } from "./focusFormat";

interface FocusHomeProps {
  mode: FocusMode;
  modelSettings: ModelSettingsView;
  onModeChange: (mode: FocusMode) => void;
  onOpenModels: () => void;
  onOpenPalette: () => void;
  onOpenWorkspace: () => void;
  onSend: (value: string) => void;
  project: WorkspaceProject;
}

export function FocusHome({
  mode,
  modelSettings,
  onModeChange,
  onOpenModels,
  onOpenPalette,
  onOpenWorkspace,
  onSend,
  project,
}: FocusHomeProps) {
  const [value, setValue] = useState("");
  const model = selectedModel(modelSettings);
  const provider = selectedProvider(modelSettings);
  const send = () => {
    const trimmed = value.trim();
    if (trimmed) {
      setValue("");
      onSend(trimmed);
    }
  };

  return (
    <div className="stage" data-mode={mode} data-screen-label="Home / new thread">
      <div className="strip">
        <div className="name"><strong>{project.name}</strong> / local</div>
        <div className="right">
          <button className="gchip" onClick={onOpenWorkspace} type="button">
            <span className={`dot ${project.git.isRepo ? "success" : "muted"}`} />
            {repoLabel(project)}
          </button>
          <button className="gchip" onClick={onOpenModels} type="button">
            {model || `${provider?.label ?? "Model"} / ${provider?.status ?? "not loaded"}`}
          </button>
        </div>
      </div>

      <div className="center">
        <div className="cstage">
          <Pipe active={modeStep(mode)} label="new thread" />
          <h1 className="home-h1 disp">What should Delyx do?</h1>
          <p className="home-sub">Send one real local instruction to open a thread.</p>

          <div className="bigcomp">
            <textarea
              className="in"
              onChange={(event) => setValue(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === "Enter" && !event.shiftKey) {
                  event.preventDefault();
                  send();
                }
              }}
              placeholder="Message Delyx with a real local instruction..."
              rows={1}
              value={value}
            />
            <div className="ctl">
              <button className="icon-btn" title="Open workspace" type="button" onClick={onOpenWorkspace}><FocusIcon name="plus" /></button>
              <div className="seg">
                {focusModes.slice(0, 3).map((item) => (
                  <button className={mode === item ? "on" : ""} key={item} onClick={() => onModeChange(item)} type="button">
                    {modeLabel(item)}
                  </button>
                ))}
              </div>
              <button className="btn-send" onClick={send} type="button">Send <span className="kbd">Enter</span></button>
            </div>
          </div>

          <div className="hints">
            {!project.git.isRepo && <button className="hint-link" onClick={onOpenWorkspace} type="button"><span className="warn"><FocusIcon name="git" /></span>Load a repository</button>}
            {!model && <button className="hint-link" onClick={onOpenModels} type="button"><span className="warn"><FocusIcon name="cpu" /></span>Choose a model</button>}
            <button className="hint-link" onClick={onOpenPalette} type="button"><span className="kbd-mini">Ctrl K</span>Commands</button>
          </div>
        </div>
      </div>

      <div className="footkeys mono">
        <span><b>Enter</b> send</span><span><b>Ctrl K</b> commands</span><span><b>Esc</b> close</span>
      </div>
    </div>
  );
}
