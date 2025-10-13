const mediaQuery = "(prefers-color-scheme: dark)";

export type Theme = "light" | "dark";

export type ThemePreference = Theme | "auto";

export type ThemeStyle = {
  element: HTMLStyleElement;
  mount: () => void;
  enable: () => void;
  disable: () => void;
};

export function createThemeStyle(css: string, { enabled }: { enabled?: boolean } = {}): ThemeStyle {
  const element = document.createElement("style");
  element.disabled = !enabled;
  element.textContent = css;
  return {
    element,
    mount: () => {
      if (!element.isConnected) {
        document.head.append(element);
      }
    },
    enable: () => {
      element.disabled = false;
    },
    disable: () => {
      element.disabled = true;
    },
  };
}

export function getSystemTheme(): Theme {
  return window.matchMedia(mediaQuery).matches ? "dark" : "light";
}

export function loadTheme(): ThemePreference {
  const stored = localStorage.getItem("theme");
  switch (stored) {
    case "light":
    case "dark":
    case "auto":
      return stored;
    default:
      return "auto";
  }
}

export function saveTheme(theme: ThemePreference): void {
  localStorage.setItem("theme", theme);
}
