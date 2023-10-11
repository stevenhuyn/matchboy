import { defineConfig } from 'vite'
import solid from 'vite-plugin-solid'
import wasmPack from 'vite-plugin-wasm-pack';

export default defineConfig({
  build: {
    minify: false
  },
  server: {
    // https://github.com/vitejs/vite/issues/14565
    host: "127.0.0.2",
    open: true,
  },
  preview: {
    // https://github.com/vitejs/vite/issues/14565
    host: "127.0.0.2",
    open: true,
  },
  plugins: [solid(), wasmPack('./matchlib')],
})
