import "./main.css";

import {
  type ThemePreference,
  type Theme,
  setTheme,
  getStoredPreference,
  applyResolvedTheme,
} from "./ts/theme-selector";

// Extend the Window type
declare global {
  interface Window {
    setMarkdownTheme: (theme: ThemePreference) => void;
    getCurrentMarkdownTheme: () => ThemePreference;
    applyMarkdownResolvedTheme: (theme: Theme) => void;
  }
}
window.setMarkdownTheme = setTheme;
window.getCurrentMarkdownTheme = getStoredPreference;
window.applyMarkdownResolvedTheme = (theme: Theme) => {
  const preference = getStoredPreference();
  applyResolvedTheme(theme, preference);
};
