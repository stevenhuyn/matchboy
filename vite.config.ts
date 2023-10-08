import { defineConfig } from 'vite'
import solid from 'vite-plugin-solid'
import { ViteRsw } from 'vite-plugin-rsw';

export default defineConfig({
  plugins: [solid(), ViteRsw(),],
})
