// Import highlight.js with all languages (desktop app)
import hljs from "highlight.js";

// Remove some languages that other libraries handle better
hljs.getLanguage("mermaid") && hljs.unregisterLanguage("mermaid");
hljs.getLanguage("math") && hljs.unregisterLanguage("math");

// Import highlight.js themes
import hljsLightTheme from "highlight.js/styles/github.css?inline";
import hljsDarkTheme from "highlight.js/styles/github-dark.css?inline";

import { createThemeStyle, type Theme } from "./theme";

const lightThemeStyle = createThemeStyle(hljsLightTheme, { enabled: true });
const darkThemeStyle = createThemeStyle(hljsDarkTheme);

export function mount(): void {
  lightThemeStyle.mount();
  darkThemeStyle.mount();
}

export function init(): void {
  setupAutoHighlighting();
}

export function setTheme(theme: Theme): void {
  switch (theme) {
    case "light":
      lightThemeStyle.enable();
      darkThemeStyle.disable();
      console.debug("Light theme of highlight.js applied");
      break;
    case "dark":
      lightThemeStyle.disable();
      darkThemeStyle.enable();
      console.debug("Dark theme of highlight.js applied");
      break;
  }
}

function highlightCodeBlock(element: HTMLElement): void {
  // Skip if already highlighted
  if (element.dataset.highlighted === "yes") {
    return;
  }

  // Extract language from class name (e.g., "language-rust" -> "rust")
  const langMatch = element.className.match(/language-(\w+)/);
  if (langMatch) {
    const lang = langMatch[1];

    // Only highlight if the language is registered
    if (hljs.getLanguage(lang)) {
      try {
        // Highlight the code block
        hljs.highlightElement(element);
        console.debug(`Highlighted code block with language: ${lang}`);
      } catch (error) {
        console.warn(`Failed to highlight code block (${lang}):`, error);
        element.dataset.highlighted = "yes";
      }
    } else {
      console.debug(`Language not registered: ${lang}`);
      element.dataset.highlighted = "yes";
    }
  }
}

function highlightCodeBlocks(container: Element): void {
  container.querySelectorAll("pre code[class*='language-']").forEach((block) => {
    highlightCodeBlock(block as HTMLElement);
  });
}

function setupAutoHighlighting(): void {
  // Create a MutationObserver to watch for changes in markdown-viewer
  const observer = new MutationObserver((mutations) => {
    for (const mutation of mutations) {
      if (mutation.type === "childList" || mutation.type === "attributes") {
        // Find the markdown-body element and highlight code blocks within it
        const markdownBody = document.querySelector(".markdown-body");
        if (markdownBody) {
          highlightCodeBlocks(markdownBody);
        }
      }
    }
  });

  // Start observing the document body for changes
  // Watch for changes in the markdown-viewer area
  const markdownViewer = document.querySelector(".markdown-viewer");
  if (markdownViewer) {
    observer.observe(markdownViewer, {
      childList: true,
      subtree: true,
      attributes: false,
    });
    console.debug("MutationObserver set up for automatic syntax highlighting");
  } else {
    console.warn("Could not find .markdown-viewer element");
  }

  // Also highlight any existing code blocks on initial load
  const markdownBody = document.querySelector(".markdown-body");
  if (markdownBody) {
    highlightCodeBlocks(markdownBody);
  }
}
