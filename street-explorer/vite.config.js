import { defineConfig } from "vite";
import { resolve } from "path";
import wasmPack from "vite-plugin-wasm-pack";
import topLevelAwait from "vite-plugin-top-level-await";

export default defineConfig({
  build: {
    rollupOptions: {
      input: {
        main: resolve(__dirname, "index.html"),
        nested: resolve(__dirname, "lane_editor.html"),
        nested2: resolve(__dirname, "land.html"),
      },
    },
  },
  base: "/osm2streets/",
  plugins: [wasmPack(["../osm2streets-js"]), topLevelAwait()],
});
