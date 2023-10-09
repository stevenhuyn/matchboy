import { defineConfig } from 'vite'
import solid from 'vite-plugin-solid'
import wasmPack from 'vite-plugin-wasm-pack';

export default defineConfig({
  build: {
    minify: false
  },
  plugins: [solid(), wasmPack('./matchlib')],
})
