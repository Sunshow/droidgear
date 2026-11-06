import path from 'path'
import { defineConfig } from 'vitest/config'
import react, { reactCompilerPreset } from '@vitejs/plugin-react'
import babel from '@rolldown/plugin-babel'

export default defineConfig({
  plugins: [
    react(),
    babel({ presets: [reactCompilerPreset()] }),
  ],
  test: {
    globals: true,
    environment: 'jsdom',
    // Heavy dialogs (ProviderDialog with the model registry) run close to 1s
    // alone and past 5s when the whole suite saturates the machine; the
    // default 5s budget turned that load into intermittent "timed out"
    // failures rather than real assertion failures.
    testTimeout: 15000,
    env: {
      DROIDGEAR_DISABLE_UPDATE_CHECK: '',
    },
    setupFiles: ['./src/test/setup.ts'],
    include: ['src/**/*.{test,spec}.{js,mjs,cjs,ts,mts,cts,jsx,tsx}'],
    exclude: [
      'node_modules',
      'dist',
      'src-tauri',
      '.git',
      '.cache',
      'build',
    ],
    coverage: {
      provider: 'v8',
      thresholds: {
        lines: 60,
        functions: 60,
        branches: 60,
      },
    },
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      'tauri-pty': path.resolve(
        __dirname,
        './node_modules/tauri-pty/dist/index.es.js'
      ),
    },
  },
})
