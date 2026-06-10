import React from "react";
import { createRoot } from "react-dom/client";

import { AppShell } from "./app/AppShell";
import "./design-system/tokens.css";
import "./styles/deck-atoms.css";
import "./styles/deck-layout.css";
import "./styles/deck-interactions.css";
import "./styles/deck-composer.css";
import "./styles/threads.css";
import "./styles/workbench-details.css";
import "./styles/workspace.css";
import "./styles/cockpit.css";
import "./styles/cockpit-markdown.css";
import "./styles/cockpit-stream.css";
import "./styles/cockpit-empty.css";
import "./styles/cockpit-runtime.css";
import "./styles/cockpit-overlays.css";
import "./styles/focus-layout.css";
import "./styles/focus-markdown.css";
import "./styles/focus-scrollbars.css";
import "./styles/focus-surfaces.css";
import "./styles/focus-settings.css";
import "./styles/focus-artifacts.css";
import "./styles/focus-attachments.css";
import "./styles/focus-overlays.css";

const rootElement = document.getElementById("root");

if (!rootElement) {
  throw new Error("Delyx Next root element was not found.");
}

createRoot(rootElement).render(
  <React.StrictMode>
    <AppShell />
  </React.StrictMode>,
);
