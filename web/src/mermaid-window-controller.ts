import mermaid from "mermaid";
import type { Theme } from "./theme";

interface ViewerState {
  scale: number;
  offsetX: number;
  offsetY: number;
  isDragging: boolean;
  lastMouseX: number;
  lastMouseY: number;
}

class MermaidWindowController {
  #container: HTMLElement | null = null;
  #wrapper: HTMLElement | null = null;
  #diagramContainer: HTMLElement | null = null;
  #maxZoom: number = 100.0;
  #state: ViewerState = {
    scale: 1.0,
    offsetX: 0,
    offsetY: 0,
    isDragging: false,
    lastMouseX: 0,
    lastMouseY: 0,
  };

  async init(source: string, diagramId: string): Promise<void> {
    this.#container = document.getElementById("mermaid-window-canvas");
    this.#wrapper = document.getElementById("mermaid-diagram-wrapper");
    this.#diagramContainer = document.getElementById("mermaid-diagram-container");

    if (!this.#container || !this.#wrapper || !this.#diagramContainer) {
      throw new Error("Viewer container not found");
    }

    // Initialize mermaid with current theme
    const currentTheme = document.body.getAttribute("data-theme") as Theme;
    this.#initializeMermaidTheme(currentTheme || "light");

    // Render the Mermaid diagram
    await this.#renderDiagram(source, diagramId);

    // Setup event listeners
    this.#setupEventListeners();

    // Listen for theme changes
    document.addEventListener("arto:theme-changed", ((event: CustomEvent) => {
      this.setTheme(event.detail);
    }) as EventListener);

    // Initial fit to window
    setTimeout(() => this.#fitToWindow(), 100);
  }

  setTheme(theme: string): void {
    // Update body theme attribute
    document.body.setAttribute("data-theme", theme);

    // Re-initialize mermaid with new theme
    this.#initializeMermaidTheme(theme as Theme);

    // Re-render the diagram with new theme asynchronously
    if (this.#diagramContainer) {
      const source = this.#diagramContainer.getAttribute("data-mermaid-source");
      const diagramId = this.#diagramContainer.getAttribute("data-diagram-id");

      if (source && diagramId) {
        // Re-render asynchronously without blocking
        this.#renderDiagram(source, diagramId)
          .then(() => {
            // Restore zoom and position after re-render
            this.#updateTransform();
          })
          .catch((error) => {
            console.error("Failed to re-render diagram:", error);
          });
      }
    }

    console.log("Theme changed to:", theme);
  }

  #initializeMermaidTheme(theme: Theme): void {
    const mermaidTheme = theme === "dark" ? "dark" : "default";
    mermaid.initialize({
      startOnLoad: false,
      theme: mermaidTheme,
      securityLevel: "loose",
      fontFamily: "inherit",
    });
  }

  async #renderDiagram(source: string, diagramId: string): Promise<void> {
    try {
      const { svg } = await mermaid.render(`viewer-${diagramId}`, source);
      if (this.#diagramContainer) {
        this.#diagramContainer.innerHTML = svg;

        // Fix SVG dimensions for CSS zoom to work properly
        const svgElement = this.#diagramContainer.querySelector("svg");
        if (svgElement) {
          const bbox = svgElement.getBBox();
          svgElement.setAttribute("width", String(bbox.width));
          svgElement.setAttribute("height", String(bbox.height));
        }

        // Store source and ID for theme switching
        this.#diagramContainer.setAttribute("data-mermaid-source", source);
        this.#diagramContainer.setAttribute("data-diagram-id", diagramId);
      }
    } catch (error) {
      console.error("Failed to render diagram:", error);
      if (this.#diagramContainer) {
        this.#diagramContainer.innerHTML = `
          <div style="color: red; padding: 2rem;">
            <strong>Rendering Error:</strong><br/>
            <pre style="white-space: pre-wrap;">${error}</pre>
          </div>
        `;
      }
    }
  }

  #setupEventListeners(): void {
    if (!this.#container) return;

    // Keyboard shortcuts
    document.addEventListener("keydown", this.#handleKeyDown.bind(this));

    // Mouse events for dragging
    this.#container.addEventListener("mousedown", this.#handleMouseDown.bind(this));
    document.addEventListener("mousemove", this.#handleMouseMove.bind(this));
    document.addEventListener("mouseup", this.#handleMouseUp.bind(this));

    // Scroll events
    this.#container.addEventListener("wheel", this.#handleWheel.bind(this), { passive: false });

    // Double-click to fit
    this.#container.addEventListener("dblclick", this.#handleDoubleClick.bind(this));
  }

  #handleKeyDown(event: KeyboardEvent): void {
    const isCmdOrCtrl = event.metaKey || event.ctrlKey;

    if (isCmdOrCtrl) {
      if (event.key === "=" || event.key === "+") {
        event.preventDefault();
        this.#zoom(0.1);
      } else if (event.key === "-") {
        event.preventDefault();
        this.#zoom(-0.1);
      } else if (event.key === "0") {
        event.preventDefault();
        this.#fitToWindow();
      }
    }
  }

  #handleMouseDown(event: MouseEvent): void {
    if (event.button === 0) {
      // Left click
      this.#state.isDragging = true;
      this.#state.lastMouseX = event.clientX;
      this.#state.lastMouseY = event.clientY;
      if (this.#container) {
        this.#container.style.cursor = "grabbing";
      }
    }
  }

  #handleMouseMove(event: MouseEvent): void {
    if (this.#state.isDragging) {
      const dx = event.clientX - this.#state.lastMouseX;
      const dy = event.clientY - this.#state.lastMouseY;

      // When using CSS zoom, translate values are in the zoomed coordinate space
      // So we need to divide by scale to get the correct movement
      this.#state.offsetX += dx;
      this.#state.offsetY += dy;

      this.#state.lastMouseX = event.clientX;
      this.#state.lastMouseY = event.clientY;

      this.#updateTransform();
    }
  }

  #handleMouseUp(): void {
    this.#state.isDragging = false;
    if (this.#container) {
      this.#container.style.cursor = "grab";
    }
  }

  #handleWheel(event: WheelEvent): void {
    // Always zoom with scroll (no modifier key needed)
    event.preventDefault();

    // Exponential zoom: scale relative to current zoom level
    // This provides natural feel - same perceived change at any zoom level
    const zoomFactor = event.deltaY > 0 ? 0.9 : 1.1; // 10% change per tick
    const oldScale = this.#state.scale;
    const newScale = Math.max(0.1, Math.min(this.#maxZoom, oldScale * zoomFactor));

    if (newScale !== oldScale) {
      // Get mouse position relative to container
      const rect = this.#container!.getBoundingClientRect();
      const mouseX = event.clientX - rect.left;
      const mouseY = event.clientY - rect.top;

      // Point in diagram space (unaffected by wrapper transform)
      const diagramX = (mouseX - this.#state.offsetX) / oldScale;
      const diagramY = (mouseY - this.#state.offsetY) / oldScale;

      // New offset to keep the diagram point at the mouse position
      this.#state.offsetX = mouseX - diagramX * newScale;
      this.#state.offsetY = mouseY - diagramY * newScale;
      this.#state.scale = newScale;

      this.#updateTransform();
      this.#updateZoomDisplay();
    }
  }

  #handleDoubleClick(): void {
    this.#fitToWindow();
  }

  #zoom(delta: number): void {
    const newScale = Math.max(0.1, Math.min(this.#maxZoom, this.#state.scale + delta));

    // Zoom to center
    if (this.#container) {
      const centerX = this.#container.clientWidth / 2;
      const centerY = this.#container.clientHeight / 2;

      // With CSS zoom, we need to adjust for the zoom factor
      const oldScale = this.#state.scale;
      const scaleRatio = newScale / oldScale;

      // Adjust offset: the point that was at centerX/Y should stay at centerX/Y
      this.#state.offsetX = centerX - (centerX - this.#state.offsetX) * scaleRatio;
      this.#state.offsetY = centerY - (centerY - this.#state.offsetY) * scaleRatio;
    }

    this.#state.scale = newScale;
    this.#updateTransform();
    this.#updateZoomDisplay();
  }

  #fitToWindow(): void {
    if (!this.#container || !this.#diagramContainer) return;

    const svg = this.#diagramContainer.querySelector("svg");
    if (!svg) return;

    const bbox = svg.getBBox();
    const padding = 40; // padding on each side

    // Available space in the canvas
    const availableWidth = this.#container.clientWidth - padding * 2;
    const availableHeight = this.#container.clientHeight - padding * 2;

    // Calculate scale to fit (allow up to max zoom)
    const scaleX = availableWidth / bbox.width;
    const scaleY = availableHeight / bbox.height;
    const scale = Math.min(scaleX, scaleY, this.#maxZoom);

    this.#state.scale = scale;

    // Center the diagram in the container
    // The diagram's rendered size after zoom is: bbox.width * scale, bbox.height * scale
    const scaledWidth = bbox.width * scale;
    const scaledHeight = bbox.height * scale;

    // Center horizontally and vertically
    this.#state.offsetX = (this.#container.clientWidth - scaledWidth) / 2;
    this.#state.offsetY = (this.#container.clientHeight - scaledHeight) / 2;

    this.#updateTransform(false); // No animation for instant fit
    this.#updateZoomDisplay();
  }

  #updateTransform(animate = false): void {
    if (!this.#wrapper || !this.#diagramContainer) return;

    if (animate) {
      this.#wrapper.style.transition = "transform 0.3s ease-out";
      this.#diagramContainer.style.transition = "zoom 0.3s ease-out";
    } else {
      this.#wrapper.style.transition = "none";
      this.#diagramContainer.style.transition = "none";
    }

    // Separate zoom and translate to avoid coordinate space issues
    // wrapper handles position (translate)
    this.#wrapper.style.transform = `translate(${this.#state.offsetX}px, ${this.#state.offsetY}px)`;
    // inner container handles zoom
    this.#diagramContainer.style.zoom = String(this.#state.scale);
  }

  #updateZoomDisplay(): void {
    // Update zoom level display via dioxus bridge
    const zoomPercent = Math.round(this.#state.scale * 100);

    // Call global function to update Rust state
    window.updateZoomLevel(zoomPercent);
  }
}

// Global instance
let controller: MermaidWindowController | null = null;

declare global {
  interface Window {
    handleMermaidWindowOpen: (source: string) => void;
    mermaidWindowController?: MermaidWindowController;
    updateZoomLevel: (zoomPercent: number) => void;
  }
}

export async function initMermaidWindow(source: string, diagramId: string): Promise<void> {
  controller = new MermaidWindowController();
  await controller.init(source, diagramId);

  // Expose globally for Rust to call
  window.mermaidWindowController = controller;
}

// Function called from main markdown viewer to open window
export function openMermaidWindow(source: string): void {
  // Call Rust function via dioxus bridge
  window.handleMermaidWindowOpen(source);
}
