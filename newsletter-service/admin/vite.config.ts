import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Served by the newsletter service under /admin, so assets live at /admin/assets.
const API = "http://localhost:8787";
const apiPaths = [
  "/admin/config",
  "/admin/stats",
  "/admin/subscribers",
  "/admin/sent",
  "/admin/template",
  "/admin/recipient-count",
  "/admin/send",
  "/admin/send-status",
];

export default defineConfig({
  base: "/admin/",
  plugins: [react()],
  build: { outDir: "dist", emptyOutDir: true },
  // `npm run dev` proxies the JSON API to the running service (needs it up on :8787).
  server: {
    proxy: Object.fromEntries(apiPaths.map((p) => [p, { target: API, changeOrigin: true }])),
  },
});
