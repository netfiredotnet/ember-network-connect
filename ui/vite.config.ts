import tailwindcss from '@tailwindcss/vite'
import react from '@vitejs/plugin-react'
import path from "path"
import { defineConfig, type PluginOption } from 'vite'

// Mock API plugin for local development without Rust backend
function mockApiPlugin(): PluginOption {
  const mockTimer = 300 // 5 minutes

  return {
    name: 'mock-api',
    configureServer(server) {
      server.middlewares.use('/get_timer', (_req, res) => {
        console.log('[Mock API] GET /get_timer -> 300')
        res.setHeader('Content-Type', 'text/plain')
        res.end(String(mockTimer))
      })

      server.middlewares.use('/reset_dhcp', (req, res) => {
        if (req.method === 'POST') {
          console.log('[Mock API] POST /reset_dhcp -> success')
          res.setHeader('Content-Type', 'application/json')
          res.end(JSON.stringify({ success: true }))
        } else {
          res.statusCode = 405
          res.end('Method not allowed')
        }
      })
    },
  }
}

export default defineConfig(({ mode }) => {
  // Set VITE_BACKEND_URL to proxy to a real backend instead of mocks
  // Example: VITE_BACKEND_URL=http://192.168.1.100:80 pnpm dev
  const backendUrl = process.env.VITE_BACKEND_URL

  return {
    plugins: [
      tailwindcss(),
      react(),
      // Use mock API unless a real backend URL is specified
      mode === 'development' && !backendUrl && mockApiPlugin(),
    ].filter(Boolean),
    build: {
      outDir: 'build',
      sourcemap: false,
    },
    server: {
      port: 3000,
      proxy: backendUrl
        ? {
            '/reset_dhcp': backendUrl,
            '/get_timer': backendUrl,
          }
        : undefined,
    },
    resolve: {
      alias: {
        "@": path.resolve(__dirname, "./src"),
      },
    },
  }
})
