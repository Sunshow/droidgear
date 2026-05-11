import { defineConfig, type UserConfig } from 'vite'
import react, { reactCompilerPreset } from '@vitejs/plugin-react'
import babel from '@rolldown/plugin-babel'
import tailwindcss from '@tailwindcss/vite'
import path, { resolve } from 'path'
import packageJson from './package.json'

const host = process.env.TAURI_DEV_HOST
const port = parseInt(process.env.TAURI_DEV_PORT || '1420', 10)

// https://vitejs.dev/config/
export default defineConfig(async () => {
  const babelPlugin = await babel({ presets: [reactCompilerPreset()] })
  return {
    define: {
      __APP_VERSION__: JSON.stringify(packageJson.version),
    },
    plugins: [
      ...react(),
      babelPlugin,
      tailwindcss(),
    ],
    resolve: {
      alias: {
        '@': path.resolve(__dirname, './src'),
        'tauri-pty': path.resolve(
          __dirname,
          './node_modules/tauri-pty/dist/index.es.js'
        ),
      },
    },
    build: {
      chunkSizeWarningLimit: 600, // Prevent warnings for template's bundled components
      cssMinify: 'lightningcss',
      rollupOptions: {
        input: {
          main: resolve(__dirname, 'index.html'),
        },
      },
    },
    // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
    //
    // 1. prevent vite from obscuring rust errors
    clearScreen: false,
    // 2. tauri expects a fixed port; use TAURI_DEV_PORT env var to customize
    server: {
      port,
      strictPort: true,
      host: host || false,
      hmr: host
        ? {
            protocol: 'ws',
            host,
            port: port + 1,
          }
        : undefined,
      watch: {
        // 3. tell vite to ignore watching `src-tauri`
        ignored: ['**/src-tauri/**'],
      },
    },
  } satisfies UserConfig
})
