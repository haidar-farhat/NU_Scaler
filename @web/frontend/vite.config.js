import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

const SERVER_URL = 'http://15.237.190.24:8000'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    host: true,
    port: 5173,
    proxy: {
      '/api': {
        target: SERVER_URL,
        changeOrigin: true,
        secure: false,
        ws: true,
      },
      '/sanctum': {
        target: SERVER_URL,
        changeOrigin: true,
        secure: false,
      }
    }
  }
})
