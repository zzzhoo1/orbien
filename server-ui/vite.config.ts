import {defineConfig} from 'vitest/config'
import vue from '@vitejs/plugin-vue'
import {fileURLToPath, URL} from 'node:url'

const srcRoot = fileURLToPath(new URL('./src', import.meta.url))
const rawSvgMock = fileURLToPath(new URL('./src/test/mocks/rawSvgMock.ts', import.meta.url))

export default defineConfig({
    plugins: [vue()],
    base: '/',
    resolve: {
        alias: [
            {find: /^@$/, replacement: srcRoot},
            {find: /^@\//, replacement: `${srcRoot}/`},
            // Stub all ?raw SVG imports so jsdom tests don't choke on SVG content
            {find: '@/assets/icon/search.svg?raw', replacement: rawSvgMock},
            {find: '@/assets/icon/signal.svg?raw', replacement: rawSvgMock},
        ],
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
