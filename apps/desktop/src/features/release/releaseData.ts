import type { ReleaseStateView } from "./releaseTypes";

export const currentReleaseState: ReleaseStateView = {
  platform: "windows",
  bundleTarget: "nsis",
  installer: "unsigned dev installer",
  smokeStatus: "not_loaded",
  signing: {
    status: "unsigned_dev",
    message: "Signing checks are clear: no certificate, digest, timestamp, or sign command is configured for dev builds.",
  },
  supportBundle: {
    exportStatus: "not_exported",
    secretPolicy: "No support bundle export is loaded in this UI session.",
  },
  updateMetadata: {
    status: "placeholder",
    channel: "dev-local",
  },
};
