import type { ReleaseStateView } from "./releaseTypes";

export const currentReleaseState: ReleaseStateView = {
  platform: "windows",
  bundleTarget: "nsis",
  installer: "unsigned dev installer",
  smokeStatus: "configured",
  signing: {
    status: "unsigned_dev",
    message: "Signing checks are clear: no certificate, digest, timestamp, or sign command is configured for dev builds.",
  },
  supportBundle: {
    exportStatus: "available",
    secretPolicy: "Support bundle exports logs/config summary without secrets.",
  },
  updateMetadata: {
    status: "placeholder",
    channel: "dev-local",
  },
};
