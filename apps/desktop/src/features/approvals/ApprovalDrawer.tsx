import { ShieldAlert } from "lucide-react";

import { Badge } from "../../design-system/Badge";
import type { ApprovalViewModel } from "../../app/types";

export function ApprovalDrawer({ approvals }: { approvals: ApprovalViewModel[] }) {
  const pending = approvals.filter((approval) => approval.status === "pending");

  return (
    <aside className="approval-drawer" aria-label="Approval drawer">
      <ShieldAlert size={16} />
      <strong>Approvals</strong>
      <Badge tone={pending.length > 0 ? "warning" : "success"}>{pending.length} pending</Badge>
    </aside>
  );
}
