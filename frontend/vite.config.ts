import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [tailwindcss(), sveltekit()],
  server: {
    port: 5172,
    proxy: {
      "/api": {
        target: "http://localhost:5171",
        changeOrigin: true,
      },
    },
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks(id: string) {
          if (id.includes("node_modules/echarts")) {
            return "echarts";
          }
          if (id.includes("node_modules/zrender")) {
            return "zrender";
          }
        },
      },
    },
  },
  test: {
    include: ["src/**/*.{test,spec}.{js,ts}"],
  },
});
