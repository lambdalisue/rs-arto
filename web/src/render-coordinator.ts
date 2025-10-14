import * as mathRenderer from "./math-renderer";
import * as mermaidRenderer from "./mermaid-renderer";
import * as syntaxHighlighter from "./syntax-highlighter";

class RenderCoordinator {
  #rafId: number | null = null;
  #isRendering = false;

  init(): void {
    const observer = new MutationObserver((mutations) => {
      // Skip if currently rendering to avoid cascade
      if (this.#isRendering) {
        return;
      }

      // Check if there's an actual content change
      const hasContentChange = mutations.some(
        (m) => m.type === "childList" || m.type === "attributes"
      );

      if (hasContentChange) {
        console.debug("RenderCoordinator: Content change detected, scheduling render");
        this.scheduleRender();
      }
    });

    const markdownViewer = document.querySelector(".markdown-viewer");
    if (markdownViewer) {
      observer.observe(markdownViewer, {
        subtree: true,
        childList: true,
        attributes: true,
      });
      console.debug("RenderCoordinator: MutationObserver set up");
    } else {
      console.warn("RenderCoordinator: Could not find .markdown-viewer element");
    }

    // Schedule an initial render
    this.scheduleRender();
  }

  scheduleRender(): void {
    if (this.#rafId !== null) {
      return; // Already scheduled
    }
    this.#rafId = requestAnimationFrame(() => {
      this.#rafId = null;
      this.#executeBatchRender();
    });
  }

  forceRenderMermaid(): void {
    const markdownBody = document.querySelector(".markdown-body");
    if (markdownBody) {
      // Clear only Mermaid diagram flags
      markdownBody.querySelectorAll("pre.mermaid[data-rendered]").forEach((el) => {
        const element = el as HTMLElement;

        // Ensure data-mermaid-src exists, otherwise skip
        if (!element.dataset.mermaidSrc) {
          console.warn("Mermaid diagram missing data-mermaid-src, skipping re-render:", element);
          return;
        }

        const originalSource = JSON.parse(element.dataset.mermaidSrc);

        // Clear the rendered content
        element.innerHTML = "";
        element.textContent = originalSource;
        element.removeAttribute("data-rendered");
      });

      // Schedule only Mermaid rendering
      this.#scheduleMermaidRender();
    }
  }

  #scheduleMermaidRender(): void {
    if (this.#rafId !== null) {
      return; // Already scheduled
    }

    this.#rafId = requestAnimationFrame(async () => {
      this.#rafId = null;

      const markdownBody = document.querySelector(".markdown-body");
      if (markdownBody) {
        this.#isRendering = true;
        try {
          await mermaidRenderer.renderDiagrams(markdownBody);
          console.debug("RenderCoordinator: Mermaid re-render completed");
        } catch (error) {
          console.error("RenderCoordinator: Error during Mermaid re-render:", error);
        } finally {
          this.#isRendering = false;
        }
      }
    });
  }

  async #executeBatchRender(): Promise<void> {
    this.#isRendering = true;

    const markdownBody = document.querySelector(".markdown-body");
    if (!markdownBody) {
      this.#isRendering = false;
      return;
    }

    try {
      mathRenderer.renderMath(markdownBody);
      syntaxHighlighter.highlightCodeBlocks(markdownBody);
      await mermaidRenderer.renderDiagrams(markdownBody);
      console.debug("RenderCoordinator: Batch render completed");
    } catch (error) {
      console.error("RenderCoordinator: Error during batch render:", error);
    } finally {
      this.#isRendering = false;
    }
  }
}

export const renderCoordinator = new RenderCoordinator();
