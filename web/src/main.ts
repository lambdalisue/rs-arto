import "../style/main.css";

import type { Theme } from "./theme";
import * as markdownViewer from "./markdown-viewer";

// Extend the Window type
declare global {
  interface Window {
    getCurrentTheme: () => Theme;
    setCurrentTheme: (theme: Theme) => void;
  }
}

function getCurrentTheme(): Theme {
  const dataTheme = document.documentElement.getAttribute("data-theme");
  switch (dataTheme) {
    case "light":
    case "dark":
      return dataTheme;
    default:
      return "light";
  }
}

function setCurrentTheme(theme: Theme) {
  document.documentElement.setAttribute("data-theme", theme);
  markdownViewer.setTheme(theme);
}

function mount(): void {
  markdownViewer.mount();
}

function init(): void {
  // Initialize other components if needed
}

mount();
setCurrentTheme(getCurrentTheme());

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", init);
} else {
  init();
}

window.getCurrentTheme = getCurrentTheme;
window.setCurrentTheme = setCurrentTheme;
