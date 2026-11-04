import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const PORT = 1420;

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: PORT,
    strictPort: true,
    watch: { ignored: ["**/src-tauri/**", "**/crates/**"] },
  },
  build: {
    target: "es2021",
    sourcemap: false,
  },
});
