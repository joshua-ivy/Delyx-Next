import { useEffect, useState } from "react";

type ThemePreference = "dark" | "light";

const themeKey = "delyx-next.theme";

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
    const toggleTheme = () => setTheme((current) => nextTheme(current));
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

  return null;
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
