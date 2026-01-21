import tailwindcss from '@tailwindcss/vite'
import react from '@vitejs/plugin-react'
import path from "path"
import { defineConfig, type PluginOption } from 'vite'

// Mock API plugin for local development without Rust backend
// Configure via /__mock endpoint (see README)
function mockApiPlugin(): PluginOption {
  let failReset = false   // Only fail /reset_dhcp
  let failTimer = false   // Fail /get_timer (page won't load)
  let timerValue = 300
  let delayMs = 0

  return {
    name: 'mock-api',
    configureServer(server) {
      // Settings endpoint to configure mock behavior
      server.middlewares.use('/__mock', (req, res) => {
        const url = new URL(req.url || '/', 'http://localhost')

        if (url.searchParams.has('failReset')) {
          failReset = url.searchParams.get('failReset') === 'true'
        }
        if (url.searchParams.has('failTimer')) {
          failTimer = url.searchParams.get('failTimer') === 'true'
        }
        if (url.searchParams.has('timer')) {
          timerValue = parseInt(url.searchParams.get('timer') || '300')
        }
        if (url.searchParams.has('delay')) {
          delayMs = parseInt(url.searchParams.get('delay') || '0')
        }

        const settings = { failReset, failTimer, timerValue, delayMs }
        console.log(`[Mock API] Settings:`, settings)

        // Return HTML page with current settings and redirect link
        res.setHeader('Content-Type', 'text/html')
        res.end(`
          <!DOCTYPE html>
          <html>
          <head><title>Mock API Settings</title></head>
          <body style="font-family: system-ui; padding: 2rem;">
            <h1>Mock API Settings Updated</h1>
            <pre>${JSON.stringify(settings, null, 2)}</pre>
            <p><a href="/">Back to app</a></p>
            <hr/>
            <h3>Quick links:</h3>
            <ul>
              <li><a href="/__mock?failReset=true">Enable reset failure</a></li>
              <li><a href="/__mock?failReset=false">Disable reset failure</a></li>
              <li><a href="/__mock?timer=10">Set timer to 10s</a></li>
              <li><a href="/__mock?timer=300">Set timer to 300s</a></li>
              <li><a href="/__mock?delay=2000">Add 2s delay</a></li>
              <li><a href="/__mock?delay=0">Remove delay</a></li>
            </ul>
          </body>
          </html>
        `)
      })

      server.middlewares.use('/get_timer', (_req, res) => {
        setTimeout(() => {
          if (failTimer) {
            console.log('[Mock API] GET /get_timer -> 500 (fail mode)')
            res.statusCode = 500
            res.end('Internal Server Error')
          } else {
            console.log(`[Mock API] GET /get_timer -> ${timerValue}`)
            res.setHeader('Content-Type', 'text/plain')
            res.end(String(timerValue))
          }
        }, delayMs)
      })

      server.middlewares.use('/reset_dhcp', (req, res) => {
        if (req.method === 'POST') {
          setTimeout(() => {
            if (failReset) {
              console.log('[Mock API] POST /reset_dhcp -> 500 (fail mode)')
              res.statusCode = 500
              res.end('Internal Server Error')
            } else {
              console.log('[Mock API] POST /reset_dhcp -> success')
              res.setHeader('Content-Type', 'application/json')
              res.end(JSON.stringify({ success: true }))
            }
          }, delayMs)
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
