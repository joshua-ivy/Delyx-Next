import { Search, X } from "lucide-react";

import { IconButton } from "./IconButton";

interface CommandPaletteProps {
  open: boolean;
  onClose: () => void;
}

const commands = [
  "Switch project",
  "Create thread",
  "Focus composer",
  "Open approvals",
  "Open diff",
  "Open tests",
  "Open evidence",
  "Toggle bottom drawer",
];

export function CommandPalette({ onClose, open }: CommandPaletteProps) {
  if (!open) {
    return null;
  }

  return (
    <div aria-modal="true" className="palette-backdrop" role="dialog">
      <div className="palette">
        <header>
          <Search size={18} />
          <input aria-label="Command search" autoFocus placeholder="Search commands" />
          <IconButton icon={<X size={16} />} label="Close command palette" onClick={onClose} />
        </header>
        <ul>
          {commands.map((command) => (
            <li key={command}>
              <button onClick={onClose} type="button">
                {command}
              </button>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}
