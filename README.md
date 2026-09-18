# CoSave

Personal and family financial tracking with a fast Rust backend and a modern Svelte 5 UI.

## Quick Start

```bash
./dev.sh doctor   # Verify environment prerequisites (Rust, Node, pnpm, Docker)
./dev.sh dev      # Start dev server with live reload
```

Open [http://localhost:5172](http://localhost:5172) in your browser.

## Ports

| Component | Dev Port | Prod Port | Notes |
| :--- | :--- | :--- | :--- |
| **Frontend** | `5172` | `5172` | Vite dev server; proxies `/api` to `5171` in dev |
| **Backend** | `5171` | `5172` | Axum API server in dev; unified static SPA + API in prod |

## Common Workflows

Always prefer `./dev.sh` over raw `cargo` or `pnpm` commands:

```bash
./dev.sh fbuild          # Fast check, lint, and build
./dev.sh full            # Full test and verification pipeline
./dev.sh flint           # Format and lint both backend and frontend
./dev.sh test            # Run automated unit and container tests
./dev.sh serve           # Run local production server
```

Run `./dev.sh --help` for the full command reference.

## Architecture & Agent Rules

- [Architecture & Standards (AGENTS.md)](AGENTS.md) — Two-phase feature lifecycle, coding standards, directory rules, and agent guidelines.
- [Domain Concepts (CONTEXT.md)](CONTEXT.md) — Product glossary, system architecture, and domain invariants.
