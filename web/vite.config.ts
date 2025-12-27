import { defineConfig, type Plugin } from "vite";
import path from "path";
import fs from "fs";

import icons from "./icons.json";

function iconSpritePlugin(): Plugin {
  return {
    name: "icon-sprite-generator",
    buildStart() {
      const iconsDir = path.join(
        __dirname,
        "node_modules/@tabler/icons/icons/outline",
      );

      const outputPath = path.join(__dirname, "public/icons/tabler-sprite.svg");

      const symbols = icons
        .map((name) => {
          const svgPath = path.join(iconsDir, `${name}.svg`);
          const svg = fs.readFileSync(svgPath, "utf-8");
          const content = svg
            .replace(/<svg[^>]*>/, "")
            .replace(/<\/svg>/, "")
            .trim();
          return `  <symbol id="tabler-${name}" viewBox="0 0 24 24">${content}</symbol>`;
        })
        .join("\n");

      const sprite = `<svg xmlns="http://www.w3.org/2000/svg" style="display: none">${symbols}</svg>`;

      fs.mkdirSync(path.dirname(outputPath), { recursive: true });
      fs.writeFileSync(outputPath, sprite);
      console.log(`✓ Generated icon sprite with ${icons.length} icons`);
    },
  };
}

export default defineConfig({
  base: "/assets/dist/",
  root: ".",
  plugins: [iconSpritePlugin()],
  build: {
    outDir: path.resolve(__dirname, "../assets/dist"),
    emptyOutDir: true,
    cssCodeSplit: false,
    lib: {
      entry: path.resolve(__dirname, "src/main.ts"),
      formats: ["es"],
    },
    rollupOptions: {
      output: {
        inlineDynamicImports: true,
        entryFileNames: "main.js",
        assetFileNames: ({ names }) => {
          if (names.some((n) => n.endsWith(".css"))) return "main.css";
          return "[name][extname]";
        },
      },
    },
  },
});
