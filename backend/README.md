# CoSave Server & API

The core application service and API engine for CoSave, managing data persistence, financial logic, and service endpoints.

## Requirements

Verify environment prerequisites from the repository root:

```bash
./dev.sh doctor           # Check all system requirements
```

## Getting Started
 
Start the service locally:

```bash
./dev.sh backend dev      # Start service in development mode (port 5171, API-only, verbose)
```

### CLI Arguments & Configuration Precedence

The backend binary (`cosave`) resolves configuration using strict precedence:

```
cosave [ENV] [OPTIONS]
cosave api [ENV] [OPTIONS]
```

1. **Environment (`ENV`)**:
   - CLI arg (positional `DEV` / `PROD` or `-e, --env <ENV>`) > `COSAVE_ENV` > defaults to `DEV`.
   - Logged immediately on startup: `INFO cosave: Environment resolved to: <ENV>`.
2. **Host (`--host`)**:
   - CLI arg (`-H, --host <HOST>`) > `COSAVE_HOST` > defaults to `0.0.0.0`.
3. **Port (`--port`)**:
   - CLI arg (`-p, --port <PORT>`) > `COSAVE_PORT` > calculated from environment:
     - `DEV` &rarr; `5171`
     - `PROD` &rarr; `5172`
   - Explicit overrides (e.g. `COSAVE_PORT=8768` or `-p 8768`) override defaults.
4. **Static Directory (`--static-dir`)**:
   - CLI arg (`--static-dir <PATH>`) > `COSAVE_STATIC_DIR` > no defaults.
   - **Mandatory** when running in full web server mode. If missing or invalid without `api_only`, process exits with code 1.
5. **API-Only Mode**:
   - CLI subcommand (`api`) or flag (`--api`). When enabled, serving static frontend assets is bypassed.
6. **Verbose Logging**:
   - CLI flag (`-v`, `--verbose`, `--debug`).

### Execution Modes

- **API Mode (Development)**:
  ```bash
  cargo run --manifest-path backend/Cargo.toml -- api --env DEV --port 5171 -v
  ```
- **Production Web Server**:
  ```bash
  cargo run --manifest-path backend/Cargo.toml --release -- --env PROD --port 5172 --static-dir frontend/dist
  ```

### Health Check Endpoint

```http
GET /health
GET /api/v1/health
```

Response envelope (empty data object):
```json
{
  "code": 0,
  "status": "HEALTHY",
  "data": {}
}
```

### Additional Commands

```bash
./dev.sh backend serve    # Run the production-ready service
./dev.sh backend full     # Test, check, build, and format service
```

For more options, run `./dev.sh backend --help`.

## Links & Documentation

- [CoSave Overview](../README.md)
- [Web Interface Guide](../frontend/README.md)
- [Engineering Guidelines](../AGENTS.md)
