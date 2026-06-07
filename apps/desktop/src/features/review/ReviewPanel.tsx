import { GitPullRequest } from "lucide-react";

import { Badge } from "../../design-system/Badge";
import { Panel } from "../../design-system/Panel";
import { Tabs } from "../../design-system/Tabs";
import { ApprovalList } from "../approvals/ApprovalList";
import { EvidencePanel } from "../evidence/EvidencePanel";
import { TestPanel } from "../tests/TestPanel";
import type {
  ApprovalViewModel,
  DiffFile,
  EvidenceViewModel,
  ReviewTab,
  TestRunViewModel,
} from "../../app/types";

const tabs = [
  { id: "diff", label: "Diff" },
  { id: "tests", label: "Tests" },
  { id: "approvals", label: "Approvals" },
  { id: "evidence", label: "Evidence" },
] satisfies { id: ReviewTab; label: string }[];

interface ReviewPanelProps {
  activeTab: ReviewTab;
  approvals: ApprovalViewModel[];
  diffFiles: DiffFile[];
  evidence: EvidenceViewModel[];
  tests: TestRunViewModel[];
  onTabChange: (tab: ReviewTab) => void;
}

export function ReviewPanel(props: ReviewPanelProps) {
  return (
    <aside className="review-panel">
      <Panel
        action={<Badge tone="warning">{props.approvals.filter((item) => item.status === "pending").length} pending</Badge>}
        eyebrow="Review"
        title="Changes and receipts"
      >
        <Tabs active={props.activeTab} onChange={props.onTabChange} tabs={tabs} />
        {props.activeTab === "diff" && <DiffPanel diffFiles={props.diffFiles} />}
        {props.activeTab === "tests" && <TestPanel tests={props.tests} />}
        {props.activeTab === "approvals" && <ApprovalList approvals={props.approvals} />}
        {props.activeTab === "evidence" && <EvidencePanel evidence={props.evidence} />}
      </Panel>
    </aside>
  );
}

function DiffPanel({ diffFiles }: { diffFiles: DiffFile[] }) {
  if (diffFiles.length === 0) {
    return <div className="diff-panel" data-testid="diff-panel">No diff artifact has been created.</div>;
  }

  return (
    <div className="diff-panel" data-testid="diff-panel">
      {diffFiles.map((file) => (
        <article className="diff-file" key={file.path}>
          <header>
            <GitPullRequest size={16} />
            <strong>{file.path}</strong>
            <Badge tone="success">+{file.additions}</Badge>
            <Badge tone="danger">-{file.deletions}</Badge>
          </header>
          <pre>
            {file.patch.map((line) => (
              <code key={line}>{line}</code>
            ))}
          </pre>
        </article>
      ))}
      <div className="review-actions">
        <Badge tone="info">Patch actions require an approval-gated binding.</Badge>
      </div>
    </div>
  );
}
