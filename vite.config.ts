import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    proxy: {
      "/api": {
        target: "http://127.0.0.1:8000",
        changeOrigin: true,
      },
    },
    watch: {
      ignored: [
        "**/src-tauri/**",
        "**/bin/**",
        "**/models/**",
        "**/target/**",
        "**/*.dll",
        "**/*.gguf",
        "**/*.exe",
        "**/*.db*",
      ],
    },
  },
  build: {
    target: ["es2022", "chrome105", "safari15"],
    outDir: "dist",
    emptyOutDir: true,
  },
  test: {
    globals: true,
    environment: "node",
  },
});
