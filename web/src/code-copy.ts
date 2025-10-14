import iconCopy from "@tabler/icons/outline/copy.svg?raw";
import iconCheck from "@tabler/icons/outline/check.svg?raw";
import iconX from "@tabler/icons/outline/x.svg?raw";
import iconPhoto from "@tabler/icons/outline/photo.svg?raw";

/**
 * Add copy buttons to code blocks
 */
export function addCopyButtons(container: Element): void {
  const preElements = container.querySelectorAll("pre:not([data-copy-button-added])");

  if (preElements.length === 0) {
    return;
  }

  preElements.forEach((pre) => {
    addCopyButton(pre as HTMLPreElement);
  });
}

function addCopyButton(pre: HTMLPreElement): void {
  // Mark as processed
  pre.dataset.copyButtonAdded = "yes";

  // Make pre element relative for absolute positioning of button
  pre.style.position = "relative";

  // Check if this is a Mermaid diagram
  const isMermaid = pre.classList.contains("preprocessed-mermaid");

  // Create text copy button
  const textButton = document.createElement("button");
  textButton.className = isMermaid ? "copy-button copy-button-text" : "copy-button";
  textButton.setAttribute("aria-label", "Copy code to clipboard");
  textButton.innerHTML = getCopyIcon();

  // Handle click event
  textButton.addEventListener("click", async (e) => {
    e.preventDefault();
    e.stopPropagation();
    await copyToClipboard(pre, textButton);
  });

  // Add button to pre element
  pre.appendChild(textButton);

  // Add image copy button for Mermaid
  if (isMermaid) {
    addImageCopyButton(pre);
  }
}

function addImageCopyButton(pre: HTMLPreElement): void {
  const button = document.createElement("button");
  button.className = "copy-button copy-button-image";
  button.setAttribute("aria-label", "Copy diagram as image");
  button.innerHTML = getPhotoIcon();

  button.addEventListener("click", async (e) => {
    e.preventDefault();
    e.stopPropagation();
    await copyMermaidAsImage(pre, button);
  });

  pre.appendChild(button);
}

async function copyToClipboard(pre: HTMLPreElement, button: HTMLButtonElement): Promise<void> {
  try {
    const content = getContentToCopy(pre);
    await navigator.clipboard.writeText(content);
    showSuccessFeedback(button);
  } catch (error) {
    console.error("Failed to copy text to clipboard", error);
    showErrorFeedback(button);
  }
}

function getContentToCopy(pre: HTMLPreElement): string {
  // Check if data-original-content exists (for math and mermaid)
  const originalContent = pre.dataset.originalContent;
  if (originalContent) {
    return originalContent;
  }

  // Otherwise, get text content from code element or pre itself
  const codeElement = pre.querySelector("code");
  if (codeElement) {
    return codeElement.textContent || "";
  }

  return pre.textContent || "";
}

function showSuccessFeedback(button: HTMLButtonElement): void {
  button.innerHTML = getCheckIcon();
  button.classList.add("copied");

  // Reset after 2 seconds
  setTimeout(() => {
    button.innerHTML = getCopyIcon();
    button.classList.remove("copied");
  }, 2000);
}

function showErrorFeedback(button: HTMLButtonElement): void {
  button.innerHTML = getErrorIcon();
  button.classList.add("error");

  // Reset after 2 seconds
  setTimeout(() => {
    button.innerHTML = getCopyIcon();
    button.classList.remove("error");
  }, 2000);
}

// SVG Icons from @tabler/icons
function getCopyIcon(): string {
  return iconCopy;
}

function getCheckIcon(): string {
  return iconCheck;
}

function getErrorIcon(): string {
  return iconX;
}

function getPhotoIcon(): string {
  return iconPhoto;
}

async function copyMermaidAsImage(pre: HTMLPreElement, button: HTMLButtonElement): Promise<void> {
  if (!navigator.clipboard?.write) {
    showErrorFeedback(button);
    return;
  }

  try {
    const svg = findSvgElement(pre);
    const dimensions = getSvgDimensions(svg);
    const canvas = createCanvasFromSvg(svg, dimensions);
    const svgDataUrl = convertSvgToDataUrl(svg, dimensions);

    // Create blob promise synchronously to preserve user gesture context
    const blobPromise = createBlobPromise(canvas, svgDataUrl);

    // Write to clipboard with promise (WebKit-compatible approach)
    await navigator.clipboard.write([new ClipboardItem({ "image/png": blobPromise })]);

    showSuccessFeedback(button);
  } catch (error) {
    console.error("Failed to copy image to clipboard", error);
    showErrorFeedback(button);
  }
}

function findSvgElement(pre: HTMLPreElement): SVGElement {
  const svg = pre.querySelector("svg");
  if (!svg) {
    throw new Error("No SVG element found");
  }
  return svg;
}

function getSvgDimensions(svg: SVGElement): { width: number; height: number } {
  const bbox = svg.getBBox();
  const width = bbox.width;
  const height = bbox.height;

  if (width === 0 || height === 0) {
    throw new Error("Invalid SVG dimensions");
  }

  return { width, height };
}

function createCanvasFromSvg(
  svg: SVGElement,
  dimensions: { width: number; height: number }
): HTMLCanvasElement {
  const scale = 2; // High resolution
  const canvas = document.createElement("canvas");
  canvas.width = dimensions.width * scale;
  canvas.height = dimensions.height * scale;

  const ctx = canvas.getContext("2d");
  if (!ctx) {
    throw new Error("Failed to get canvas context");
  }

  ctx.scale(scale, scale);
  ctx.fillStyle = "#ffffff";
  ctx.fillRect(0, 0, dimensions.width, dimensions.height);

  return canvas;
}

function convertSvgToDataUrl(
  svg: SVGElement,
  dimensions: { width: number; height: number }
): string {
  const svgClone = svg.cloneNode(true) as SVGElement;
  svgClone.setAttribute("width", String(dimensions.width));
  svgClone.setAttribute("height", String(dimensions.height));

  const svgString = new XMLSerializer().serializeToString(svgClone);
  const base64SVG = btoa(unescape(encodeURIComponent(svgString)));

  return `data:image/svg+xml;base64,${base64SVG}`;
}

function createBlobPromise(canvas: HTMLCanvasElement, dataUrl: string): Promise<Blob> {
  return new Promise<Blob>((resolve, reject) => {
    const img = new Image();

    img.onload = () => {
      const ctx = canvas.getContext("2d");
      if (!ctx) {
        reject(new Error("Canvas context lost"));
        return;
      }

      try {
        ctx.drawImage(img, 0, 0);

        canvas.toBlob((blob) => {
          if (blob) {
            resolve(blob);
          } else {
            reject(new Error("Failed to create blob"));
          }
        }, "image/png");
      } catch (error) {
        reject(error);
      }
    };

    img.onerror = () => reject(new Error("Failed to load image"));
    img.src = dataUrl;
  });
}
