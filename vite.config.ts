import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// https://vitejs.dev/config/
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [react()],

  // Prevent Vite from obscuring Rust errors
  clearScreen: false,

  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // Tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },

  // Env variables starting with these prefixes are exposed to the client
  envPrefix: ["VITE_", "TAURI_ENV_*"],

  build: {
    // Tauri supports es2021
    target: "es2021",
    // Don't minify for debug builds; production uses esbuild defaults
    minify: process.env.TAURI_ENV_DEBUG ? false : "esbuild",
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
});
