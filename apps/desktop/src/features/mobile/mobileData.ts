import type { MobileStateView } from "./mobileTypes";

export const currentMobileState: MobileStateView = {
  paired: false,
  policy: {
    allowLowRiskApproval: false,
    maxApprovalRisk: "low",
    canAccessFiles: false,
    canAccessTerminal: false,
  },
  threads: [],
  pendingApprovals: [],
  runs: [],
};
