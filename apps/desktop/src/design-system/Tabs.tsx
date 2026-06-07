import type { ReviewTab } from "../app/types";

interface TabDefinition {
  id: ReviewTab;
  label: string;
}

interface TabsProps {
  active: ReviewTab;
  tabs: TabDefinition[];
  onChange: (tab: ReviewTab) => void;
}

export function Tabs({ active, onChange, tabs }: TabsProps) {
  return (
    <div aria-label="Review panel sections" className="tabs" role="tablist">
      {tabs.map((tab) => (
        <button
          aria-selected={active === tab.id}
          className={active === tab.id ? "tab active" : "tab"}
          key={tab.id}
          onClick={() => onChange(tab.id)}
          role="tab"
          type="button"
        >
          {tab.label}
        </button>
      ))}
    </div>
  );
}
