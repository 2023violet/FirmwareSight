import react from '@vitejs/plugin-react';
import { defineConfig } from 'vitest/config';

// Tauri expects a fixed dev port, and a silently shifted port looks like a broken shell.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
  build: {
    target: 'es2022',
    outDir: 'dist',
    emptyOutDir: true,
  },
  test: {
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
  },
});
