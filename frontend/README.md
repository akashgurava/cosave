# CoSave Frontend

SvelteKit 2 (Svelte 5) SPA with Tailwind CSS v4 and Vite 8. Communicates via `/api/v1` (proxied to `:3000` in dev). In production, builds static assets into `dist/` served by Axum.

## Quickstart

```bash
./dev.sh ui dev           # Start Vite dev server on :5173
./dev.sh ui serve         # Preview compiled static SPA
./dev.sh ui full          # Test, check, build, and flint frontend
./dev.sh ui shadcn <comp> # Add shadcn-svelte primitive component
```

Refer to `./dev.sh ui --help` for deeper commands. See [README.md](../README.md) and [AGENTS.md](../AGENTS.md).
