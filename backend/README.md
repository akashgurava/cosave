# CoSave Backend

Rust (Axum 0.8, Tokio 1.53) REST API and static SPA server. Serves `/api/v1` endpoints and hosts compiled static frontend assets with client-side SPA fallback.

## Quickstart

```bash
./dev.sh backend dev      # Start Axum dev server with verbose logging
./dev.sh backend serve    # Run release Axum server
./dev.sh backend full     # Test, check, build, and flint backend
```

Refer to `./dev.sh backend --help` for deeper commands. See [README.md](../README.md) and [AGENTS.md](../AGENTS.md).
