# CoSave

A personal and family financial management application built to bring clarity to spending patterns, simplify budget tracking, and grow savings together.

## Requirements

Verify your environment prerequisites:

```bash
./dev.sh doctor           # Check all system requirements
```

## Getting Started

Start the application locally:

```bash
./dev.sh dev              # Start the application with live reload
```

Once started, open your browser to:
- Web App: [http://localhost:5172](http://localhost:5172) (development mode proxies API calls to the backend on port `5171`)

### Port & Configuration Conventions

| Component | Dev Port | Prod / Container Port | Notes |
| :--- | :--- | :--- | :--- |
| **Frontend** | `5172` | `5172` | Always accessed on `5172`; proxies `/api` to `5171` in dev |
| **Backend** | `5171` | `5172` | API server in dev; serves both API and static SPA in prod |

The backend supports CLI arguments and environment variables with strict precedence:
1. **Environment**: Positional `DEV`/`PROD` or `-e, --env` > `COSAVE_ENV` > defaults to `DEV`
2. **Host**: `-H, --host` > `COSAVE_HOST` > defaults to `0.0.0.0`
3. **Port**: `-p, --port` > `COSAVE_PORT` > calculated from environment (`DEV` -> `5171`, `PROD` -> `5172`)
4. **Static Directory**: `--static-dir` > `COSAVE_STATIC_DIR` (mandatory in non-API mode)
5. **API-Only Mode**: `cosave api` or `--api` (disables static file requirement)

### Additional Commands

```bash
./dev.sh serve            # Run the production-ready application (port 5172)
./dev.sh test             # Run automated tests
./dev.sh full             # Run full verification pipeline
```

For a complete list of commands, run `./dev.sh --help`.

## Links & Documentation

- [Web Interface Guide](frontend/README.md)
- [Server & API Guide](backend/README.md)
- [Architecture & Standards](AGENTS.md)
- [Project Overview](CONTEXT.md)
