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

## Structure

```
src/
├── main.tsx              # Entry point
├── index.css             # Tailwind imports + theme
├── components/
│   ├── App.tsx           # Main app component
│   ├── Notifications.tsx # Alert notifications
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
