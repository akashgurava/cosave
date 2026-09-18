# CoSave

[![CI/CD](https://github.com/akashgurava/cosave/actions/workflows/ci.yml/badge.svg)](https://github.com/akashgurava/cosave/actions/workflows/ci.yml)

Personal and family financial tracking with a fast Rust backend and a modern Svelte 5 UI.

## Quick Start

### Option A: Standalone Docker Compose (Recommended for Users)

Run CoSave without cloning the entire source repository:

```bash
# 1. Create a directory and download docker-compose.yml
mkdir cosave && cd cosave
curl -sSL -O https://raw.githubusercontent.com/akashgurava/cosave/main/docker-compose.yml

# 2. (Optional) Download environment template if you need custom ports or user IDs
curl -sSL -O https://raw.githubusercontent.com/akashgurava/cosave/main/.env.example && cp .env.example .env

# 3. Start the application
docker compose up -d
```

### Option B: From Cloned Repository

```bash
# 1. Clone repository and navigate inside
git clone https://github.com/akashgurava/cosave.git
cd cosave

# 2. Start prebuilt image
docker compose up -d

# (Or compile and run directly from local source)
docker compose up -d --build
```

Open [http://localhost:5172](http://localhost:5172) in your browser. All data and SQLite database files are persisted in `./data/`.

> **Manage Container**:
> - View logs: `docker compose logs -f`
> - Stop container: `docker compose down`
> - Update image: `docker compose pull && docker compose up -d`

## Local Development & Building

For development, compiling, and testing, use the `./dev.sh` workflow helper:

### Prerequisites
```bash
./dev.sh doctor          # Verify environment prerequisites (Rust, Node, pnpm, Docker)
```

### Development Servers (Live Reload)
```bash
./dev.sh dev             # Start both backend (:5171) and frontend (:5172) with live reload
./dev.sh backend dev     # Start backend dev server only
./dev.sh ui dev          # Start frontend dev server only
```

### Docker Workflows via `./dev.sh`
```bash
./dev.sh build docker    # Build multi-stage Docker image locally (cosave:latest)
./dev.sh serve docker    # Run prebuilt production Docker container (:5172)
./dev.sh test docker     # Run container integration smoke tests
./dev.sh clean docker    # Clean stale Docker test containers
```

### Build & Verification Pipelines
Always prefer `./dev.sh` over raw `cargo` or `pnpm` commands:

```bash
./dev.sh fbuild          # Fast build: check -> flint (auto-fixes) -> build (release)
./dev.sh full            # Full verification: test -> check -> build -> flint
./dev.sh flint           # Auto-format and lint both backend and frontend
./dev.sh check           # Run cargo check and svelte-check
./dev.sh test            # Run unit and integration tests
./dev.sh serve           # Run local production server from source
```

Run `./dev.sh --help` for the full command reference.

## Ports

| Component | Dev Port | Prod Port | Notes |
| :--- | :--- | :--- | :--- |
| **Frontend** | `5172` | `5172` | Vite dev server; proxies `/api` to `5171` in dev |
| **Backend** | `5171` | `5172` | Axum API server in dev; unified static SPA + API in prod |

## Architecture & Agent Rules

- [Architecture & Standards (AGENTS.md)](AGENTS.md) — Two-phase feature lifecycle, coding standards, directory rules, and agent guidelines.
- [Domain Concepts (CONTEXT.md)](CONTEXT.md) — Product glossary, system architecture, and domain invariants.
