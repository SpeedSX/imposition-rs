import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";

const rootDir = path.dirname(fileURLToPath(import.meta.url));
const pkgDir = path.join(rootDir, "pkg");

export default defineConfig({
  root: path.join(rootDir, "web"),
  publicDir: "public",
  server: {
    port: 5174,
    open: true,
    fs: {
      allow: [rootDir, pkgDir],
    },
  },
  build: {
    outDir: path.resolve(rootDir, "dist-web"),
    emptyOutDir: true,
  },
});
