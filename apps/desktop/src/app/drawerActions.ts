type ToastTone = "info" | "success" | "warning";

const toastEventName = "delyx-next.toast";

export function bindTerminalDrawerActions() {
  return [
    bindCopyOutput(),
    bindDrawerAction(".terminal-jump-error", "Jump to error", "No error line is available in the current terminal output.", "warning"),
    bindDrawerAction(".terminal-open-file", "Open referenced file", "No referenced file is available in the current terminal output.", "warning"),
    bindDrawerAction(".terminal-rerun", "Rerun command", "Rerun requires a captured command artifact and approval.", "warning"),
    bindDrawerAction(".terminal-approve-rerun", "Approve rerun", "Approve rerun requires a pending approval proposal.", "warning"),
  ];
}

function bindCopyOutput() {
  const button = document.querySelector(".terminal-copy");
  const term = document.querySelector(".term");
  if (!(button instanceof HTMLButtonElement) || !(term instanceof HTMLElement)) {
    return () => undefined;
  }
  const run = () => {
    const output = term.textContent?.trim() ?? "";
    if (!output) {
      notify("No terminal output to copy.", "warning");
      return;
    }
    void navigator.clipboard.writeText(output)
      .then(() => notify("Terminal output copied.", "success"))
      .catch(() => notify("Clipboard copy failed.", "warning"));
  };
  button.addEventListener("click", run);
  return () => button.removeEventListener("click", run);
}

function bindDrawerAction(selector: string, label: string, message: string, tone: ToastTone) {
  const button = document.querySelector(selector);
  if (!(button instanceof HTMLButtonElement)) {
    return () => undefined;
  }
  if (button.disabled) {
    button.setAttribute("aria-label", `${label} unavailable`);
    return () => undefined;
  }
  const run = () => notify(message, tone);
  button.setAttribute("aria-label", label);
  button.addEventListener("click", run);
  return () => button.removeEventListener("click", run);
}

function notify(message: string, tone: ToastTone = "info") {
  window.dispatchEvent(new CustomEvent(toastEventName, { detail: { message, tone } }));
}
