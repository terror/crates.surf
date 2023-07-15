import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  root: './client',
  plugins: [svelte()],
  server: {
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:8000/api',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api/, ''),
      },
    },
  },
});
