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
  // Set current theme to initialize all components
  // This must be called AFTER renderCoordinator.init()
  // otherwise scheduleRender() in renderCoordinator.init()
  // will be skipped due to renderCoordinator.forceRenderMermaid()
  // called in setCurrentTheme() below
  setCurrentTheme(getCurrentTheme());
}
