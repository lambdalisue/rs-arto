import mermaid from "mermaid";
import type { Theme } from "./theme";
import { openMermaidWindow } from "./mermaid-window-controller";

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
  const mermaidBlocks = collectMermaidBlocks(container);

  if (mermaidBlocks.length === 0) {
    return;
  }

  console.debug(`Rendering ${mermaidBlocks.length} mermaid diagrams in parallel`);

  // Render all diagrams in parallel for better performance
  const renderPromises = mermaidBlocks.map((block) =>
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

  // Get the mermaid source code from the element data attribute
  // This data attribute is embedded during markdown parsing phase
  // in Rust code.
  const mermaidSource = element.dataset.originalContent || element.textContent || "";
  if (!mermaidSource) {
    element.dataset.rendered = "true"; // Mark as processed to skip in future
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

    // Make diagram clickable to open viewer
    const svgElement = element.querySelector("svg");
    if (svgElement) {
      svgElement.style.cursor = "pointer";
      svgElement.style.transition = "opacity 0.2s ease";

      svgElement.addEventListener("click", () => {
        openMermaidWindow(mermaidSource);
      });

      // Add hover effect
      svgElement.addEventListener("mouseenter", () => {
        svgElement.style.opacity = "0.7";
      });
      svgElement.addEventListener("mouseleave", () => {
        svgElement.style.opacity = "1.0";
      });
    }

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

function collectMermaidBlocks(container: Element): HTMLElement[] {
  const blocks = new Map<HTMLElement, string>();

  const preprocessed = container.querySelectorAll("pre.preprocessed-mermaid:not([data-rendered])");
  preprocessed.forEach((block) => {
    const element = block as HTMLElement;
    blocks.set(element, element.dataset.originalContent || element.textContent || "");
  });

  const codeBlocks = container.querySelectorAll("pre code.language-mermaid:not([data-rendered])");
  codeBlocks.forEach((code) => {
    const pre = code.closest("pre");
    if (!pre) {
      return;
    }
    const element = pre as HTMLElement;
    if (blocks.has(element)) {
      return;
    }
    const source = code.textContent || "";
    element.classList.add("preprocessed-mermaid");
    element.dataset.originalContent = source;
    blocks.set(element, source);
  });

  return Array.from(blocks.keys());
}
