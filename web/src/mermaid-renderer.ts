import mermaid from "mermaid";
import type { Theme } from "./theme";

export function init(): void {
  mermaid.initialize({
    startOnLoad: false, // We'll manually trigger rendering
    theme: "default", // Will be updated based on app theme
    securityLevel: "loose", // Allow more flexibility in diagrams
    fontFamily: "inherit",
  });

  console.debug("Mermaid initialized");
  setupAutoRendering();
}

export function setTheme(theme: Theme): void {
  // Update mermaid theme configuration
  const mermaidTheme = theme === "dark" ? "dark" : "default";
  mermaid.initialize({
    startOnLoad: false,
    theme: mermaidTheme,
    securityLevel: "loose",
    fontFamily: "inherit",
  });

  console.debug(`Mermaid theme set to: ${mermaidTheme}`);

  // Re-render all existing diagrams with the new theme
  rerenderAllDiagrams();
}

async function renderDiagram(element: HTMLElement): Promise<void> {
  // Skip if already rendered (has SVG child)
  if (element.querySelector("svg")) {
    return;
  }

  // Get the mermaid source code from the element
  const mermaidSource = element.textContent?.trim();
  if (!mermaidSource) {
    console.warn("Empty mermaid block found");
    return;
  }

  try {
    // Generate a unique ID for this diagram
    const id = `mermaid-${crypto.randomUUID()}`;

    // Render the diagram
    const { svg } = await mermaid.render(id, mermaidSource);

    // Replace the text content with the rendered SVG
    element.innerHTML = svg;
    element.dataset.rendered = "true";

    console.debug(`Rendered mermaid diagram: ${id}`);
  } catch (error) {
    console.error("Failed to render mermaid diagram:", error);
    // Show error in the diagram
    element.innerHTML = `<div style="color: red; padding: 1rem; border: 1px solid red; border-radius: 4px;">
      <strong>Mermaid Error:</strong><br/>
      <pre style="margin-top: 0.5rem; white-space: pre-wrap;">${error}</pre>
    </div>`;
  }
}

async function renderDiagrams(container: Element): Promise<void> {
  const mermaidBlocks = container.querySelectorAll("pre.mermaid");

  for (const block of Array.from(mermaidBlocks)) {
    await renderDiagram(block as HTMLElement);
  }
}

async function rerenderAllDiagrams(): Promise<void> {
  const markdownBody = document.querySelector(".markdown-body");
  if (!markdownBody) {
    return;
  }

  // Find all rendered diagrams and clear them
  const mermaidBlocks = markdownBody.querySelectorAll("pre.mermaid[data-rendered='true']");

  for (const block of Array.from(mermaidBlocks)) {
    const element = block as HTMLElement;

    // Get original source from data attribute
    const originalSource = element.dataset.mermaidSrc;
    if (originalSource) {
      // Restore original text content
      element.textContent = originalSource;
      element.removeAttribute("data-rendered");
    }
  }

  // Re-render all diagrams
  await renderDiagrams(markdownBody);
}

function setupAutoRendering(): void {
  // Create a MutationObserver to watch for changes in markdown-viewer
  const observer = new MutationObserver((mutations) => {
    for (const mutation of mutations) {
      if (mutation.type === "childList") {
        // Find the markdown-body element and render diagrams within it
        const markdownBody = document.querySelector(".markdown-body");
        if (markdownBody) {
          renderDiagrams(markdownBody);
        }
      }
    }
  });

  // Start observing the markdown-viewer area
  const markdownViewer = document.querySelector(".markdown-viewer");
  if (markdownViewer) {
    observer.observe(markdownViewer, {
      childList: true,
      subtree: true,
      attributes: false,
    });
    console.debug("MutationObserver set up for automatic mermaid rendering");
  } else {
    console.warn("Could not find .markdown-viewer element");
  }

  // Also render any existing diagrams on initial load
  const markdownBody = document.querySelector(".markdown-body");
  if (markdownBody) {
    renderDiagrams(markdownBody);
  }
}
