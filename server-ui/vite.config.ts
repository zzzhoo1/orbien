import {defineConfig} from 'vitest/config'
import vue from '@vitejs/plugin-vue'
import {fileURLToPath, URL} from 'node:url'
import path from 'node:path'

const srcRoot = fileURLToPath(new URL('./src', import.meta.url))
const rawSvgMock = path.resolve(srcRoot, 'test/mocks/rawSvgMock.ts')

export default defineConfig({
    plugins: [vue()],
    base: '/',
    resolve: {
        alias: {
            '@': srcRoot,
            '@/assets/icon/search.svg?raw': rawSvgMock,
            '@/assets/icon/signal.svg?raw': rawSvgMock,
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
