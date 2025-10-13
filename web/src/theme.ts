export type Theme = "light" | "dark";

export type ThemeStyle = {
  element: HTMLStyleElement;
  mount: () => void;
  enable: () => void;
  disable: () => void;
};

export function createThemeStyle(css: string, { enabled }: { enabled?: boolean } = {}): ThemeStyle {
  const element = document.createElement("style");
  element.disabled = !enabled;
  element.textContent = css;
  return {
    element,
    mount: () => {
      if (!element.isConnected) {
        document.head.append(element);
      }
    },
    enable: () => {
      element.disabled = false;
    },
    disable: () => {
      element.disabled = true;
    },
  };
}
