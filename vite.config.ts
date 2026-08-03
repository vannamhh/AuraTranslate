import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// Four settings below are required by Tauri, not by taste:
//   - server.port + strictPort: tauri.conf.json `devUrl` points at 1420. If Vite
//     silently hops to another port, `tauri dev` loads a dead URL.
//   - clearScreen false: keeps Rust compiler errors visible in the terminal.
//   - envPrefix: Tauri injects TAURI_* vars the frontend is allowed to read.
export default defineConfig({
  plugins: [vue()],
  clearScreen: false,
  envPrefix: ['VITE_', 'TAURI_'],
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
})
