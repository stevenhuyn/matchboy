import { defineConfig } from 'vite'
import solid from 'vite-plugin-solid'
import wasmPack from 'vite-plugin-wasm-pack';

export default defineConfig({
  build: {
    minify: false
  },
  server: {
    host: "127.0.0.2",
    open: true,
  },
  preview: {
    host: "127.0.0.2",
    open: true,
  },
  plugins: [solid(), wasmPack('./matchlib')],
})
