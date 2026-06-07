import React from "react";
import { createRoot } from "react-dom/client";

import { AppShell } from "./app/AppShell";
import "./design-system/tokens.css";
import "./styles/cockpit.css";
import "./styles/threads.css";
import "./styles/workspace.css";

const rootElement = document.getElementById("root");

if (!rootElement) {
  throw new Error("Delyx Next root element was not found.");
}

createRoot(rootElement).render(
  <React.StrictMode>
    <AppShell />
  </React.StrictMode>,
);
