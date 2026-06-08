import { useEffect, useState } from "react";

import { loadApprovalSnapshot } from "../features/approvals/approvalClient";
import { currentActionProposals } from "../features/approvals/approvalData";

export function useRunApprovals(runId: string | undefined) {
  const [actionProposals, setActionProposals] = useState(currentActionProposals);

  useEffect(() => {
    if (!runId) {
      setActionProposals([]);
      return;
    }
    setActionProposals([]);
    let cancelled = false;
    void loadApprovalSnapshot(runId).then((snapshot) => {
      if (!cancelled && snapshot) {
        setActionProposals(snapshot);
      }
    });
    return () => {
      cancelled = true;
    };
  }, [runId]);

  return { actionProposals, setActionProposals };
}
