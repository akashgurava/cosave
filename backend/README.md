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
./dev.sh backend dev      # Start service in development mode
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
