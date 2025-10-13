import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const icons = [
  "sun",
  "moon",
  "contrast-2",
  "chevron-left",
  "chevron-right",
  "file",
  "folder-open",
  "command",
  "click",
  "file-upload",
];
const iconsDir = path.join(__dirname, "../node_modules/@tabler/icons/icons/outline");
const outputPath = path.join(__dirname, "../public/icons/tabler-sprite.svg");

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

const sprite = `<svg xmlns="http://www.w3.org/2000/svg" style="display: none">
${symbols}
</svg>`;

fs.mkdirSync(path.dirname(outputPath), { recursive: true });
fs.writeFileSync(outputPath, sprite);
console.log(`✓ Generated icon sprite with ${icons.length} icons at ${outputPath}`);
