# CoSave

Rust (Axum) + SvelteKit 2 SPA financial app. In development, Vite (`:5173`) proxies `/api` to Axum (`:3000`). In production, Axum serves the compiled SPA with client fallback.

## Quickstart

```bash
./dev.sh dev              # Start dev servers with live-reload
./dev.sh serve            # Run compiled production server
./dev.sh ui full          # Test, check, build, and flint frontend
./dev.sh backend full     # Test, check, build, and flint backend
```

Refer to `./dev.sh --help` for deeper commands. Modules: [backend/README.md](backend/README.md) | [frontend/README.md](frontend/README.md).
