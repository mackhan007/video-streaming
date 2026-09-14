import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

/** Dev server proxies EMS + IMS health so the UI can show process status. */
export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    proxy: {
      "/uploader": "http://localhost:8080",
      "/streamer": "http://localhost:8080",
      "/health": "http://localhost:8080",
      "/ready": "http://localhost:8080",
      "/ims-health": {
        target: "http://localhost:8088",
        rewrite: (path) => path.replace(/^\/ims-health/, "/health"),
      },
      "/cdn-health": {
        target: "http://localhost:8081",
        rewrite: () => "/",
      },
    },
  },
});
