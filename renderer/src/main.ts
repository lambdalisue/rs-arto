import "../style/main.css";

import { type Theme, getSystemTheme } from "./theme";
import * as markdownViewer from "./markdown-viewer";
import * as syntaxHighlighter from "./syntax-highlighter";
import * as mermaidRenderer from "./mermaid-renderer";
import { renderCoordinator } from "./render-coordinator";

function getCurrentTheme(): Theme {
  const theme = document.body.getAttribute("data-theme");
  switch (theme) {
    case "light":
    case "dark":
      return theme;
    default:
      return getSystemTheme();
  }
}

export function setCurrentTheme(theme: Theme) {
  document.body.setAttribute("data-theme", theme);
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

  // Listen for theme changes from Rust
  document.addEventListener("arto:theme-changed", ((event: CustomEvent) => {
    setCurrentTheme(event.detail);
  }) as EventListener);

  // Set initial theme
  setCurrentTheme(getCurrentTheme());
}

// Re-export mermaid window functions
export { initMermaidWindow } from "./mermaid-window-controller";
