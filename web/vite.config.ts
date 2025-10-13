import { defineConfig } from "vite";
import path from "path";

export default defineConfig({
  base: "/assets/dist/",
  root: ".",
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
