import {defineConfig} from 'vitest/config'
import vue from '@vitejs/plugin-vue'
import {fileURLToPath, URL} from 'node:url'

export default defineConfig({
    plugins: [vue()],
    base: '/',
    resolve: {
        alias: {
            '@': fileURLToPath(new URL('./src', import.meta.url)),
            '@/assets/icon/search.svg?raw': fileURLToPath(new URL('./src/test/mocks/rawSvgMock.ts', import.meta.url)),
            '@/assets/icon/signal.svg?raw': fileURLToPath(new URL('./src/test/mocks/rawSvgMock.ts', import.meta.url)),
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
