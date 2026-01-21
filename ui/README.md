# Ember Network Connect UI

## Stack

- React 19
- TypeScript 5.9
- Vite 7
- Tailwind CSS 4
- shadcn/ui components

## Development

```bash
# Install dependencies
pnpm install

# Start dev server with mock API (http://localhost:3000)
pnpm dev

# Start dev server connected to a real backend
VITE_BACKEND_URL=http://192.168.1.100:80 pnpm dev

# Build for production
pnpm build

# Preview production build
pnpm preview

# Lint
pnpm lint
```

### Mock API

When running `pnpm dev` without `VITE_BACKEND_URL`, a mock API is used:

- `GET /get_timer` - Returns `300` (5 minutes)
- `POST /reset_dhcp` - Logs to console, returns success

This allows frontend development on macOS/Windows without the Rust backend.

### Testing Failure States

Configure mock behavior by visiting these URLs:

```bash
# Make reset button fail (500 error)
http://localhost:3000/__mock?failReset=true

# Disable reset failure
http://localhost:3000/__mock?failReset=false

# Make timer fetch fail (page won't load properly)
http://localhost:3000/__mock?failTimer=true

# Set timer to 10 seconds (test countdown)
http://localhost:3000/__mock?timer=10

# Add 2 second delay to responses
http://localhost:3000/__mock?delay=2000

# Combine options
http://localhost:3000/__mock?failReset=true&delay=1000
```

Settings persist until the dev server restarts. Visit `/__mock` to see current settings.

## Structure

```
src/
├── main.tsx              # Entry point
├── index.css             # Tailwind imports + theme
├── components/
│   ├── App.tsx           # Main app component
│   └── ui/               # shadcn/ui components
│       └── button.tsx
├── lib/
│   └── utils.ts          # cn() utility
└── img/
    └── logo.svg
```

## API Endpoints

The UI expects these endpoints from the backend:

- `GET /get_timer` - Returns remaining timeout in seconds
- `POST /reset_dhcp` - Triggers DHCP reset
