import katex from "katex";

/**
 * Initialize math rendering
 */
export function init(): void {
  console.debug("KaTeX initialized");
  setupAutoRendering();
}

function renderInlineMath(container: Element): void {
  // Process inline math: <span class="math math-inline">...</span>
  const inlineMathElements = container.querySelectorAll("span.math.math-inline");
  for (const element of Array.from(inlineMathElements)) {
    const mathContent = element.textContent?.trim() || "";
    if (mathContent && element.getAttribute("data-katex-rendered") !== "true") {
      try {
        katex.render(mathContent, element as HTMLElement, {
          throwOnError: false,
          displayMode: false,
        });
        element.setAttribute("data-katex-rendered", "true");
        console.debug("Rendered inline math (pulldown-cmark)");
      } catch (error) {
        console.error("Failed to render inline math:", error);
        (element as HTMLElement).style.color = "red";
      }
    }
  }

  // Process display math: <span class="math math-display">...</span>
  const displayMathElements = container.querySelectorAll("span.math.math-display");
  for (const element of Array.from(displayMathElements)) {
    const mathContent = element.textContent?.trim() || "";
    if (mathContent && element.getAttribute("data-katex-rendered") !== "true") {
      try {
        katex.render(mathContent, element as HTMLElement, {
          throwOnError: false,
          displayMode: true,
        });
        element.setAttribute("data-katex-rendered", "true");
        console.debug("Rendered display math (pulldown-cmark)");
      } catch (error) {
        console.error("Failed to render display math:", error);
        (element as HTMLElement).style.color = "red";
      }
    }
  }
}

function renderBlockMath(container: Element): void {
  const mathBlocks = container.querySelectorAll(".language-math");

  for (const block of Array.from(mathBlocks)) {
    const element = block as HTMLElement;

    // Skip if already rendered
    if (element.dataset.rendered === "true") {
      continue;
    }

    const mathContent = element.textContent?.trim() || "";

    try {
      katex.render(mathContent, element, {
        throwOnError: false,
        displayMode: true,
      });
      element.dataset.rendered = "true";
      console.debug("Rendered math block");
    } catch (error) {
      console.error("Failed to render math block:", error);
      element.style.color = "red";
    }
  }
}

/**
 * Render all math expressions in a container
 */
function renderMath(container: Element): void {
  renderInlineMath(container);
  renderBlockMath(container);
}

/**
 * Sets up MutationObserver to automatically render math expressions
 * when markdown content changes
 */
function setupAutoRendering(): void {
  // Create a MutationObserver to watch for changes in markdown-viewer
  const observer = new MutationObserver((mutations) => {
    for (const mutation of mutations) {
      if (mutation.type === "childList") {
        // Find the markdown-body element and render math within it
        const markdownBody = document.querySelector(".markdown-body");
        if (markdownBody) {
          renderMath(markdownBody);
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
    console.debug("MutationObserver set up for automatic math rendering");
  } else {
    console.warn("Could not find .markdown-viewer element");
  }

  // Also render any existing math on initial load
  const markdownBody = document.querySelector(".markdown-body");
  if (markdownBody) {
    renderMath(markdownBody);
  }
}
