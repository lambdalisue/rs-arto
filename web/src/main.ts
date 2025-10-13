import "../style/main.css";

import { type ThemePreference, getSystemTheme, loadTheme, saveTheme } from "./theme";
import * as markdownViewer from "./markdown-viewer";
import * as syntaxHighlighter from "./syntax-highlighter";
import * as mermaidRenderer from "./mermaid-renderer";
import { renderCoordinator } from "./render-coordinator";

let currentTheme: ThemePreference = loadTheme();

export function getCurrentTheme(): ThemePreference {
  return currentTheme;
}

export function setCurrentTheme(themePreference: ThemePreference) {
  saveTheme(themePreference);
  const theme = themePreference === "auto" ? getSystemTheme() : themePreference;
  document.documentElement.setAttribute("data-theme", theme);
  markdownViewer.setTheme(theme);
  syntaxHighlighter.setTheme(theme);
  mermaidRenderer.setTheme(theme);
  renderCoordinator.forceRenderMermaid();
}

export function init(): void {
  markdownViewer.mount();
  syntaxHighlighter.mount();
  mermaidRenderer.init();
  renderCoordinator.init();
  // Set current theme to initialize all components
  // This must be called AFTER renderCoordinator.init()
  // otherwise scheduleRender() in renderCoordinator.init()
  // will be skipped due to renderCoordinator.forceRenderMermaid()
  // called in setCurrentTheme() below
  setCurrentTheme(currentTheme);
}
