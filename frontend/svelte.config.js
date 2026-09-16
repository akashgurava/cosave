import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  vitePlugin: {
    experimental: {
      compileModule: {
        include: [/\.ts$/],
      },
    },
  },
  kit: {
    adapter: adapter({
      pages: "dist",
      assets: "dist",
      fallback: "index.html",
      precompress: false,
      strict: true,
    }),
    alias: {
      $components: "src/lib/components",
    },
  },
  compilerOptions: {
    runes: true,
  },
};

export default config;
