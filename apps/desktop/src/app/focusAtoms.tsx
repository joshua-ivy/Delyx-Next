import {
  Bolt,
  Check,
  ChevronDown,
  Command,
  Cpu,
  Dices,
  FileText,
  FlaskConical,
  GitBranch,
  GitCompare,
  Home,
  KeyRound,
  List,
  ListChecks,
  Paintbrush,
  Plus,
  Search,
  Settings,
  Shield,
  SlidersHorizontal,
  Zap,
} from "lucide-react";

const iconMap = {
  bolt: Bolt,
  check: Check,
  cmd: Command,
  cpu: Cpu,
  dice: Dices,
  diff: GitCompare,
  doc: FileText,
  down: ChevronDown,
  flask: FlaskConical,
  git: GitBranch,
  home: Home,
  key: KeyRound,
  paint: Paintbrush,
  plan: ListChecks,
  plus: Plus,
  search: Search,
  settings: Settings,
  shield: Shield,
  sliders: SlidersHorizontal,
  threads: List,
  zap: Zap,
} as const;

export type FocusIconName = keyof typeof iconMap;

export function FocusIcon({ name }: { name: FocusIconName }) {
  const Icon = iconMap[name];
  return <Icon aria-hidden="true" />;
}

export function RailIconButton({
  active,
  icon,
  label,
  onClick,
}: {
  active?: boolean;
  icon: FocusIconName;
  label: string;
  onClick: () => void;
}) {
  return (
    <button
      aria-label={label}
      className={`rail-btn${active ? " on" : ""}`}
      onClick={onClick}
      title={label}
      type="button"
    >
      <FocusIcon name={icon} />
    </button>
  );
}

const steps = ["explore", "plan", "build", "test", "review"];

export function Pipe({ active, label }: { active: number; label?: string }) {
  return (
    <div className="pipe">
      {steps.map((step, index) => (
        <span className={`pd${index === active ? " on" : ""}`} key={step} title={step} />
      ))}
      {label && <span className="pl">{label}</span>}
    </div>
  );
}

export function Think() {
  return (
    <span className="think" aria-label="Thinking">
      <i />
      <i />
      <i />
    </span>
  );
}
