import githubMarkdownLightCss from "github-markdown-css/github-markdown-light.css?inline";
import githubMarkdownDarkCss from "github-markdown-css/github-markdown-dark.css?inline";

// Type definitions for theme management
export type Theme = "light" | "dark";
export type ThemePreference = Theme | "auto";

// Extend the Window type
declare global {
  interface Window {
    setMarkdownTheme: (theme: ThemePreference) => void;
    getCurrentMarkdownTheme: () => ThemePreference;
    applyMarkdownResolvedTheme: (theme: Theme) => void;
  }
}

type ManagedStyle = {
  element: HTMLStyleElement;
  mount: () => void;
};

function createManagedStyle(css: string, dataset: Record<string, string>): ManagedStyle {
  const element = document.createElement("style");
  element.disabled = true;
  element.textContent = css;
  Object.entries(dataset).forEach(([key, value]) => {
    element.dataset[key] = value;
  });

  return {
    element,
    mount: () => {
      if (!element.isConnected) {
        document.head.append(element);
      }
    },
  };
}

function applyMarkdownStyles(theme: Theme) {
  lightTheme.mount();
  darkTheme.mount();

  if (theme === "light") {
    enableTheme(lightTheme.element);
    disableTheme(darkTheme.element);
    return;
  }

  enableTheme(darkTheme.element);
  disableTheme(lightTheme.element);
}

function enableTheme(style: HTMLStyleElement) {
  style.media = "all";
  style.disabled = false;
}

function disableTheme(style: HTMLStyleElement) {
  style.media = "all";
  style.disabled = true;
}

// Get the current theme from local storage, defaults to auto
export function getStoredPreference(): ThemePreference {
  const stored = localStorage.getItem("markdown-theme") as ThemePreference;
  switch (stored) {
    case "light":
    case "dark":
    case "auto":
      return stored;
    default:
      return "auto";
  }
}

function getDefaultResolvedTheme(): Theme {
  try {
    return window.matchMedia && window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "dark"
      : "light";
  } catch {
    return "light";
  }
}

export function applyResolvedTheme(theme: Theme, preference: ThemePreference) {
  document.documentElement.dataset.markdownThemePreference = preference;
  document.documentElement.setAttribute("data-theme", theme);
  applyMarkdownStyles(theme);

  window.dispatchEvent(
    new CustomEvent("theme-change", {
      detail: { preference, resolved: theme, theme },
    })
  );
}

// Set the theme (user preference)
export function setTheme(theme: ThemePreference) {
  localStorage.setItem("markdown-theme", theme);

  if (theme === "light" || theme === "dark") {
    applyResolvedTheme(theme, theme);
  } else {
    const resolved = getDefaultResolvedTheme();
    applyResolvedTheme(resolved, "auto");
  }
}

function setupAutoThemeListener() {
  try {
    const query = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = (event: MediaQueryListEvent) => {
      if (getStoredPreference() === "auto") {
        applyResolvedTheme(event.matches ? "dark" : "light", "auto");
      }
    };

    if ("addEventListener" in query) {
      query.addEventListener("change", handler);
    } else {
      // @ts-expect-error addListener is deprecated but still in use on older browsers
      query.addListener(handler);
    }
  } catch {
    // Ignore errors from matchMedia on unsupported platforms
  }
}

const lightTheme = createManagedStyle(githubMarkdownLightCss, { markdownTheme: "light" });
const darkTheme = createManagedStyle(githubMarkdownDarkCss, { markdownTheme: "dark" });

const initialPreference = getStoredPreference();
setTheme(initialPreference);
setupAutoThemeListener();
