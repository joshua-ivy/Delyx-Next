import { useEffect, useMemo, useState } from "react";
import { Search, X } from "lucide-react";

import { IconButton } from "./IconButton";

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
          <Search size={18} />
          <input
            aria-label="Command search"
            autoFocus
            onChange={(event) => setQuery(event.currentTarget.value)}
            placeholder="Search commands"
            value={query}
          />
          <IconButton icon={<X size={16} />} label="Close command palette" onClick={onClose} />
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
