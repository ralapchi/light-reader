import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  // Tauri 需要相对路径
  base: './',
  // 防止 Vite 使用 HMR websocket 而走 Tauri IPC
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
})
