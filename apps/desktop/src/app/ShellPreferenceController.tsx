import { useEffect, useState } from "react";

import { bindTerminalDrawerActions } from "./drawerActions";

type ThemePreference = "dark" | "light";
type LayoutPreference = { drawer: number; review: number; side: number };
type ResizeKind = keyof LayoutPreference;
type ToastTone = "info" | "success" | "warning";
type ToastNotice = { id: string; message: string; tone: ToastTone };

const layoutBounds: Record<ResizeKind, [number, number]> = {
  drawer: [120, 260],
  review: [320, 520],
  side: [220, 360],
};
const layoutDefaults: LayoutPreference = { drawer: 150, review: 392, side: 252 };
const layoutKey = "delyx-next.layout";
const themeKey = "delyx-next.theme";
const toastEventName = "delyx-next.toast";

export function ShellPreferenceController() {
  const [theme, setTheme] = useState<ThemePreference>(readThemePreference);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    writeThemePreference(theme);
    const button = document.querySelector(".theme-trigger");
    button?.setAttribute("aria-label", `Switch to ${nextTheme(theme)} theme`);
    button?.setAttribute("title", `Switch to ${nextTheme(theme)} theme`);
    if (button) {
      button.textContent = theme === "light" ? "D" : "L";
    }
  }, [theme]);

  useEffect(() => {
    const button = document.querySelector(".theme-trigger");
    const toggleTheme = () => setTheme((current) => {
      const next = nextTheme(current);
      notifyLocalAction(`${next} theme enabled`, "success");
      return next;
    });
    const activateOnKeyboard = (event: Event) => {
      const key = (event as KeyboardEvent).key;
      if (key === "Enter" || key === " ") {
        event.preventDefault();
        toggleTheme();
      }
    };
    button?.setAttribute("role", "button");
    button?.setAttribute("tabindex", "0");
    button?.addEventListener("click", toggleTheme);
    button?.addEventListener("keydown", activateOnKeyboard);
    return () => {
      button?.removeEventListener("click", toggleTheme);
      button?.removeEventListener("keydown", activateOnKeyboard);
    };
  }, []);

  useEffect(() => {
    applyLayoutPreference(readLayoutPreference());
    const cleanups = [
      bindLayoutGrip(".resize-side", "side"),
      bindLayoutGrip(".resize-review", "review"),
      bindLayoutGrip(".resize-drawer", "drawer"),
      bindLogSearch(),
      bindOutputCollapse(),
      ...bindTerminalDrawerActions(),
      bindSafeAction(".plan-question", "Ask question", "Question capture is not wired yet; no model call ran.", "warning"),
      bindSafeAction(".diff-approve", "Approve apply", "Patch apply still requires approval; no file changed.", "warning"),
      bindSafeAction(".diff-reject", "Reject diff", "Diff rejected locally; no file changed."),
      bindSafeAction(".diff-revert", "Revert checkpoint", "No checkpoint was restored; a checkpoint artifact is required.", "warning"),
      bindSafeAction(".diff-revise", "Ask for revision", "Revision request stayed local; no file changed."),
    ];
    return () => cleanups.forEach((cleanup) => cleanup());
  }, []);

  return <ToastViewport />;
}

export function notifyLocalAction(message: string, tone: ToastTone = "info") {
  window.dispatchEvent(new CustomEvent(toastEventName, { detail: { message, tone } }));
}

function ToastViewport() {
  const [toasts, setToasts] = useState<ToastNotice[]>([]);

  useEffect(() => {
    const onToast = (event: Event) => {
      const detail = (event as CustomEvent<Omit<ToastNotice, "id">>).detail;
      if (!detail?.message) {
        return;
      }
      const id = `toast-${Date.now()}-${Math.round(Math.random() * 1000)}`;
      setToasts((current) => [...current.slice(-2), { id, message: detail.message, tone: detail.tone }]);
      window.setTimeout(() => {
        setToasts((current) => current.filter((toast) => toast.id !== id));
      }, 3200);
    };
    window.addEventListener(toastEventName, onToast);
    return () => window.removeEventListener(toastEventName, onToast);
  }, []);

  return (
    <div aria-live="polite" className="toast-viewport">
      {toasts.map((toast) => (
        <div className={`toast toast-${toast.tone}`} key={toast.id} role="status">
          {toast.message}
        </div>
      ))}
    </div>
  );
}

function nextTheme(theme: ThemePreference): ThemePreference {
  return theme === "dark" ? "light" : "dark";
}

function readThemePreference(): ThemePreference {
  try {
    return window.localStorage.getItem(themeKey) === "light" ? "light" : "dark";
  } catch {
    return "dark";
  }
}

function writeThemePreference(theme: ThemePreference) {
  try {
    window.localStorage.setItem(themeKey, theme);
  } catch {
    // Preference persistence is optional; the visible theme still applies.
  }
}

function bindLayoutGrip(selector: string, kind: ResizeKind) {
  const grip = document.querySelector(selector);
  if (!(grip instanceof HTMLElement)) {
    return () => undefined;
  }
  grip.setAttribute("aria-label", `Resize ${kind} pane`);
  grip.setAttribute("role", "separator");
  grip.setAttribute("tabindex", "0");
  const onPointerDown = (event: Event) => startResize(event as PointerEvent, kind);
  const onKeyDown = (event: Event) => resizeWithKeyboard(event as KeyboardEvent, kind);
  grip.addEventListener("pointerdown", onPointerDown);
  grip.addEventListener("keydown", onKeyDown);
  return () => {
    grip.removeEventListener("pointerdown", onPointerDown);
    grip.removeEventListener("keydown", onKeyDown);
  };
}

function bindLogSearch() {
  const input = document.querySelector(".log-search");
  if (!(input instanceof HTMLInputElement)) {
    return () => undefined;
  }
  const onInput = () => filterLogLines(input.value);
  input.addEventListener("input", onInput);
  return () => input.removeEventListener("input", onInput);
}

function bindOutputCollapse() {
  const button = document.querySelector(".output-collapse");
  const term = document.querySelector(".term");
  if (!(button instanceof HTMLButtonElement) || !(term instanceof HTMLElement)) {
    return () => undefined;
  }
  const toggle = () => {
    const collapsed = term.dataset.outputCollapsed !== "true";
    term.dataset.outputCollapsed = `${collapsed}`;
    button.ariaPressed = `${collapsed}`;
    button.textContent = collapsed ? "Expand output" : "Collapse output";
  };
  button.addEventListener("click", toggle);
  return () => button.removeEventListener("click", toggle);
}

function bindSafeAction(selector: string, label: string, message: string, tone: ToastTone = "info") {
  const button = document.querySelector(selector);
  if (!(button instanceof HTMLElement)) {
    return () => undefined;
  }
  const run = () => notifyLocalAction(message, tone);
  const onKeyDown = (event: Event) => {
    const key = (event as KeyboardEvent).key;
    if (key === "Enter" || key === " ") {
      event.preventDefault();
      run();
    }
  };
  button.setAttribute("aria-label", label);
  button.setAttribute("role", "button");
  button.setAttribute("tabindex", "0");
  button.addEventListener("click", run);
  button.addEventListener("keydown", onKeyDown);
  return () => {
    button.removeEventListener("click", run);
    button.removeEventListener("keydown", onKeyDown);
  };
}

function filterLogLines(query: string) {
  const needle = query.trim().toLowerCase();
  document.querySelectorAll<HTMLElement>("[data-log-line]").forEach((line) => {
    line.hidden = Boolean(needle) && !line.textContent?.toLowerCase().includes(needle);
  });
}

function startResize(event: PointerEvent, kind: ResizeKind) {
  event.preventDefault();
  const start = readLayoutPreference();
  const startX = event.clientX;
  const startY = event.clientY;
  const onMove = (move: PointerEvent) => {
    const delta = kind === "drawer" ? move.clientY - startY : move.clientX - startX;
    saveLayoutPreference(resizedLayout(start, kind, delta));
  };
  const onUp = () => {
    window.removeEventListener("pointermove", onMove);
    window.removeEventListener("pointerup", onUp);
  };
  window.addEventListener("pointermove", onMove);
  window.addEventListener("pointerup", onUp);
}

function resizeWithKeyboard(event: KeyboardEvent, kind: ResizeKind) {
  const delta = keyDelta(event.key);
  if (delta === 0 || (kind === "drawer" && !["ArrowUp", "ArrowDown"].includes(event.key))) {
    return;
  }
  if (kind !== "drawer" && !["ArrowLeft", "ArrowRight"].includes(event.key)) {
    return;
  }
  event.preventDefault();
  saveLayoutPreference(resizedLayout(readLayoutPreference(), kind, delta));
}

function resizedLayout(start: LayoutPreference, kind: ResizeKind, delta: number): LayoutPreference {
  const direction = kind === "side" ? 1 : -1;
  return { ...start, [kind]: clamp(start[kind] + delta * direction, ...layoutBounds[kind]) };
}

function applyLayoutPreference(layout: LayoutPreference) {
  document.documentElement.style.setProperty("--side-w", `${layout.side}px`);
  document.documentElement.style.setProperty("--review-w", `${layout.review}px`);
  document.documentElement.style.setProperty("--drawer-h", `${layout.drawer}px`);
}

function readLayoutPreference(): LayoutPreference {
  try {
    const parsed = JSON.parse(window.localStorage.getItem(layoutKey) ?? "{}") as Partial<LayoutPreference>;
    return {
      drawer: normalizeLayoutValue(parsed.drawer, "drawer"),
      review: normalizeLayoutValue(parsed.review, "review"),
      side: normalizeLayoutValue(parsed.side, "side"),
    };
  } catch {
    return layoutDefaults;
  }
}

function saveLayoutPreference(layout: LayoutPreference) {
  applyLayoutPreference(layout);
  try {
    window.localStorage.setItem(layoutKey, JSON.stringify(layout));
  } catch {
    // Layout persistence is optional; the visible resize still applies.
  }
}

function normalizeLayoutValue(value: unknown, kind: ResizeKind) {
  return typeof value === "number" ? clamp(value, ...layoutBounds[kind]) : layoutDefaults[kind];
}

function keyDelta(key: string) {
  return key === "ArrowRight" || key === "ArrowDown" ? 16 : key === "ArrowLeft" || key === "ArrowUp" ? -16 : 0;
}

function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value));
}
