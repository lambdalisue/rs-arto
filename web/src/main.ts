import "../style/main.css";

import type { Theme } from "./theme";
import * as markdownViewer from "./markdown-viewer";
import * as syntaxHighlighter from "./syntax-highlighter";
import * as mermaidRenderer from "./mermaid-renderer";
import { renderCoordinator } from "./render-coordinator";

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
  syntaxHighlighter.setTheme(theme);
  mermaidRenderer.setTheme(theme);
  renderCoordinator.forceRenderMermaid();
}

function mount(): void {
  markdownViewer.mount();
  syntaxHighlighter.mount();
}

function init(): void {
  mermaidRenderer.init();
  renderCoordinator.init();
  // Set current theme to initialize all components
  // This must be called AFTER renderCoordinator.init()
  // otherwise scheduleRender() in renderCoordinator.init()
  // will be skipped due to renderCoordinator.forceRenderMermaid()
  // called in setCurrentTheme() below
  setCurrentTheme(getCurrentTheme());
}

mount();

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", init);
} else {
  init();
}

window.getCurrentTheme = getCurrentTheme;
window.setCurrentTheme = setCurrentTheme;
