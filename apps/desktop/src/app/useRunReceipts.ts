import { useEffect, useState } from "react";

import { loadPatchSnapshot } from "../features/patches/patchClient";
import { currentPatchProposals } from "../features/patches/patchData";
import { currentReviewReports } from "../features/review/reviewData";
import { loadReviewSnapshot } from "../features/review/reviewClient";
import { loadTestSnapshot } from "../features/tests/testClient";
import { currentTestArtifacts } from "../features/tests/testData";

export function useRunReceipts(runId: string | undefined) {
  const [patches, setPatches] = useState(currentPatchProposals);
  const [reviews, setReviews] = useState(currentReviewReports);
  const [tests, setTests] = useState(currentTestArtifacts);

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

  useEffect(() => {
    if (!runId) {
      setTests([]);
      return;
    }
    setTests([]);
    let cancelled = false;
    void loadTestSnapshot(runId).then((snapshot) => {
      if (!cancelled) {
        setTests(snapshot ?? []);
      }
    });
    return () => {
      cancelled = true;
    };
  }, [runId]);

  return { patches, reviews, setPatches, setReviews, tests };
}
