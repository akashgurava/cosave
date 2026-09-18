# CoSave Web Interface

The user interface and interactive dashboard for CoSave, featuring financial tracking, category visualization, and account settings.

## Requirements

Verify environment prerequisites from the repository root:

```bash
./dev.sh doctor           # Check all system requirements
```

## Getting Started

Start the web interface locally:

```bash
./dev.sh ui dev           # Start the web interface on http://localhost:5172
```

In development mode, Vite serves the application on port `5172` and transparently proxies API calls (`/api/*`) to the backend listening on port `5171`. In production, the backend serves both the static UI bundle and API on port `5172`.

### Additional Commands

```bash
./dev.sh ui serve         # Preview compiled static web interface
./dev.sh ui full          # Test, check, build, and format web interface
```

For more options, run `./dev.sh ui --help`.

## Links & Documentation

- [CoSave Overview](../README.md)
- [Server & API Guide](../backend/README.md)
- [Engineering Guidelines](../AGENTS.md)
