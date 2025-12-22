import * as mathRenderer from "./math-renderer";
import * as mermaidRenderer from "./mermaid-renderer";
import * as syntaxHighlighter from "./syntax-highlighter";
import * as codeCopy from "./code-copy";

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

    const root = document.body;
    observer.observe(root, {
      subtree: true,
      childList: true,
      attributes: true,
    });
    console.debug("RenderCoordinator: MutationObserver set up on document.body");

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
    const markdownBodies = document.querySelectorAll(".markdown-body");
    if (markdownBodies.length === 0) {
      return;
    }

    markdownBodies.forEach((markdownBody) => {
      markdownBody.querySelectorAll("pre.preprocessed-mermaid[data-rendered]").forEach((el) => {
        const element = el as HTMLElement;

        // Clear the rendered content and copy button flag
        element.innerHTML = "";
        element.removeAttribute("data-rendered");
        element.removeAttribute("data-copy-button-added");
      });
    });

    // Schedule only Mermaid rendering
    this.#scheduleMermaidRender();
  }

  #scheduleMermaidRender(): void {
    if (this.#rafId !== null) {
      return; // Already scheduled
    }

    this.#rafId = requestAnimationFrame(async () => {
      this.#rafId = null;

      const markdownBodies = document.querySelectorAll(".markdown-body");
      if (markdownBodies.length === 0) {
        return;
      }

      this.#isRendering = true;
      try {
        await Promise.all(
          Array.from(markdownBodies).map(async (markdownBody) => {
            await mermaidRenderer.renderDiagrams(markdownBody);
            // Re-add copy buttons after Mermaid re-render
            codeCopy.addCopyButtons(markdownBody);
          })
        );
        console.debug("RenderCoordinator: Mermaid re-render completed");
      } catch (error) {
        console.error("RenderCoordinator: Error during Mermaid re-render:", error);
      } finally {
        this.#isRendering = false;
      }
    });
  }

  async #executeBatchRender(): Promise<void> {
    this.#isRendering = true;

    const markdownBodies = document.querySelectorAll(".markdown-body");
    if (markdownBodies.length === 0) {
      this.#isRendering = false;
      return;
    }

    try {
      await Promise.all(
        Array.from(markdownBodies).map(async (markdownBody) => {
          mathRenderer.renderMath(markdownBody);
          syntaxHighlighter.highlightCodeBlocks(markdownBody);
          await mermaidRenderer.renderDiagrams(markdownBody);
          codeCopy.addCopyButtons(markdownBody);
        })
      );
      console.debug("RenderCoordinator: Batch render completed");
    } catch (error) {
      console.error("RenderCoordinator: Error during batch render:", error);
    } finally {
      this.#isRendering = false;
    }
  }
}

export const renderCoordinator = new RenderCoordinator();
