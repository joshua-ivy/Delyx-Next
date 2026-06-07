import { useEffect, useState } from "react";
import { loadRiskTaxonomySnapshot } from "../features/approvals/approvalClient";
import { riskTaxonomy, type RiskTaxonomySnapshotView } from "../features/approvals/approvalTypes";

export function useApprovalPolicy() {
  const [policy, setPolicy] = useState<RiskTaxonomySnapshotView>(riskTaxonomy);

  useEffect(() => {
    let cancelled = false;
    void loadRiskTaxonomySnapshot().then((snapshot) => {
      if (!cancelled && snapshot) {
        setPolicy(snapshot);
      }
    });
    return () => {
      cancelled = true;
    };
  }, []);

  return policy;
}
