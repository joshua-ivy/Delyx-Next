import { useEffect, useMemo, useState } from "react";

export interface CommandPaletteItem {
  detail: string;
  id: string;
  label: string;
}

interface CommandPaletteProps {
  commands: readonly CommandPaletteItem[];
  onRun: (commandId: string) => void;
  open: boolean;
  onClose: () => void;
}

export function CommandPalette({ commands, onClose, onRun, open }: CommandPaletteProps) {
  const [query, setQuery] = useState("");
  const visibleCommands = useMemo(() => {
    const needle = query.trim().toLowerCase();
    if (!needle) {
      return commands;
    }
    return commands.filter((command) => `${command.label} ${command.detail}`.toLowerCase().includes(needle));
  }, [commands, query]);

  useEffect(() => {
    if (!open) {
      setQuery("");
    }
  }, [open]);

  if (!open) {
    return null;
  }

  return (
    <div aria-label="Command palette" aria-modal="true" className="palette-backdrop" role="dialog">
      <div className="palette">
        <header>
          <span className="deck-pal-search mono">&#8984;K</span>
          <input
            aria-label="Command search"
            autoFocus
            onChange={(event) => setQuery(event.currentTarget.value)}
            placeholder="Run a command..."
            value={query}
          />
          <button aria-label="Close command palette" className="deck-pal-esc mono" onClick={onClose} type="button">esc</button>
        </header>
        <ul>
          {visibleCommands.map((command) => (
            <li key={command.id}>
              <button onClick={() => onRun(command.id)} type="button">
                <span>{command.label}</span>
                <small>{command.detail}</small>
              </button>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}
