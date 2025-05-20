import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    host: true,
    port: 5173,
    proxy: {
      '/api': {
        target: 'http://15.237.190.24:8000',
        changeOrigin: true,
        secure: false,
        ws: true,
      },
      '/sanctum': {
        target: 'http://15.237.190.24:8000',
        changeOrigin: true,
        secure: false,
      }
    }
  }
})
