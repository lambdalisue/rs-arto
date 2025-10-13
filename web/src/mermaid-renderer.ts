import mermaid from "mermaid";
import type { Theme } from "./theme";

export function init(): void {
  mermaid.initialize({
    startOnLoad: false, // We'll manually trigger rendering
    theme: "default", // Will be updated based on app theme
    securityLevel: "loose", // Allow more flexibility in diagrams
    fontFamily: "inherit",
  });
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
}

export async function renderDiagrams(container: Element): Promise<void> {
  const mermaidBlocks = container.querySelectorAll("pre.mermaid:not([data-rendered])");

  if (mermaidBlocks.length === 0) {
    return;
  }

  console.debug(`Rendering ${mermaidBlocks.length} mermaid diagrams in parallel`);

  // Render all diagrams in parallel for better performance
  const renderPromises = Array.from(mermaidBlocks).map((block) =>
    renderDiagram(block as HTMLElement).catch((error) => {
      console.error("Failed to render mermaid diagram:", error);
      // Don't let one failure stop others
    })
  );

  await Promise.all(renderPromises);
  console.debug("All mermaid diagrams rendered");
}

async function renderDiagram(element: HTMLElement): Promise<void> {
  // Skip if already rendered (has SVG child or marked as rendered)
  if (element.dataset.rendered === "true" || element.querySelector("svg")) {
    return;
  }

  // Get the mermaid source code from the element
  const mermaidSource = element.textContent?.trim();
  if (!mermaidSource) {
    element.dataset.rendered = "true"; // Mark as processed to skip in future
    return;
  }

  try {
    // Store original source for theme switching
    if (!element.dataset.mermaidSrc) {
      element.dataset.mermaidSrc = mermaidSource;
    }

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
    element.dataset.rendered = "true"; // Mark as processed even on error
  }
}
