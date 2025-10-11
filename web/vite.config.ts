import { defineConfig } from "vite";
import svgr from "vite-plugin-svgr";
import path from "path";

export default defineConfig({
  base: "/assets/dist/",
  plugins: [svgr()],
  root: ".",
  build: {
    outDir: path.resolve(__dirname, "../assets/dist"),
    emptyOutDir: true,
    cssCodeSplit: true,
    lib: {
      entry: path.resolve(__dirname, "src/main.ts"),
      formats: ["es"],
      fileName: () => "main.js",
    },
    rollupOptions: {
      output: {
        entryFileNames: "main.js",
        assetFileNames: (assetInfo) => {
          if (assetInfo.names && assetInfo.names.some((v) => v.endsWith(".css"))) {
            return "main.css";
          }
          return "assets/[name][extname]";
        },
      },
    },
  },
});
