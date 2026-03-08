import { defineConfig } from "vite";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  // Tauri's IPC uses a custom protocol; prevent Vite from clearing the base.
  base: "./",

  build: {
    // Tauri's Rust side picks up the output from here.
    outDir: "dist",
    // Don't minify for easier debugging; Tauri's release profile handles it.
    minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_DEBUG,
  },

  server: {
    host: host ?? false,
    port: 1420,
    strictPort: true,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
  },
});
