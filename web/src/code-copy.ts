import iconCopy from "@tabler/icons/outline/copy.svg?raw";
import iconCheck from "@tabler/icons/outline/check.svg?raw";
import iconX from "@tabler/icons/outline/x.svg?raw";

/**
 * Add copy buttons to code blocks
 */
export function addCopyButtons(container: Element): void {
  // Find all pre elements (code blocks, mermaid, math)
  const preElements = container.querySelectorAll("pre:not([data-copy-button-added])");

  if (preElements.length === 0) {
    return;
  }

  console.debug(`Adding copy buttons to ${preElements.length} code blocks`);

  preElements.forEach((pre) => {
    addCopyButton(pre as HTMLPreElement);
  });
}

function addCopyButton(pre: HTMLPreElement): void {
  // Mark as processed
  pre.dataset.copyButtonAdded = "yes";

  // Make pre element relative for absolute positioning of button
  pre.style.position = "relative";

  // Create copy button
  const button = document.createElement("button");
  button.className = "copy-button";
  button.setAttribute("aria-label", "Copy code to clipboard");
  button.innerHTML = getCopyIcon();

  // Handle click event
  button.addEventListener("click", async (e) => {
    e.preventDefault();
    e.stopPropagation();
    await copyToClipboard(pre, button);
  });

  // Add button to pre element
  pre.appendChild(button);
}

async function copyToClipboard(pre: HTMLPreElement, button: HTMLButtonElement): Promise<void> {
  try {
    // Get content to copy
    const content = getContentToCopy(pre);

    // Copy to clipboard
    await navigator.clipboard.writeText(content);

    // Show success feedback
    showSuccessFeedback(button);

    console.debug("Code copied to clipboard");
  } catch (error) {
    console.error("Failed to copy code:", error);
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
