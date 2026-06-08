import { useEffect, useState } from "react";

import { loadPatchSnapshot } from "../features/patches/patchClient";
import { currentPatchProposals } from "../features/patches/patchData";
import { currentReviewReports } from "../features/review/reviewData";
import { loadReviewSnapshot } from "../features/review/reviewClient";

export function useRunReceipts(runId: string | undefined) {
  const [patches, setPatches] = useState(currentPatchProposals);
  const [reviews, setReviews] = useState(currentReviewReports);

  useEffect(() => {
    if (!runId) {
      setPatches([]);
      return;
    }
    setPatches([]);
    let cancelled = false;
    void loadPatchSnapshot(runId).then((snapshot) => {
      if (!cancelled) {
        setPatches(snapshot ?? []);
      }
    });
    return () => {
      cancelled = true;
    };
  }, [runId]);

  useEffect(() => {
    if (!runId) {
      setReviews([]);
      return;
    }
    setReviews([]);
    let cancelled = false;
    void loadReviewSnapshot(runId).then((snapshot) => {
      if (!cancelled) {
        setReviews(snapshot ?? []);
      }
    });
    return () => {
      cancelled = true;
    };
  }, [runId]);

  return { patches, reviews, setReviews };
}
