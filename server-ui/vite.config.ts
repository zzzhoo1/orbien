import {defineConfig} from 'vitest/config'
import vue from '@vitejs/plugin-vue'
import {fileURLToPath, URL} from 'node:url'
import type {Plugin} from 'vite'

// Intercept *.svg?raw imports in Vitest (jsdom cannot parse real SVG files).
// resolve.alias does not support query-string keys, so we use a plugin instead.
function rawSvgStubPlugin(): Plugin {
  return {
    name: 'raw-svg-stub',
    enforce: 'pre',
    load(id) {
      if (id.endsWith('.svg?raw') || (id.includes('.svg?') && id.includes('raw'))) {
        return `export default '<svg></svg>'`
      }
    },
  }
}

export default defineConfig({
    plugins: [vue(), rawSvgStubPlugin()],
    base: '/',
    resolve: {
        alias: {
            '@': fileURLToPath(new URL('./src', import.meta.url)),
        },
    },
    server: {
        port: 5173,
        proxy: {
            '/api': {
                target: 'http://127.0.0.1:8020',
                changeOrigin: true,
            },
            '/healthz': {
                target: 'http://127.0.0.1:8020',
                changeOrigin: true,
            },
        },
    },
    build: {
        outDir: '../server/assets',
        emptyOutDir: true,
        assetsDir: 'assets',
    },
    test: {
        environment: 'jsdom',
        globals: true,
        include: ['src/**/*.{spec,test}.ts'],
    },
})
