# CoSave Backend

Axum-based REST API and static asset server for CoSave, backed by SQLite with SQLx.

## Quick Start

Always run commands from the repository root using `./dev.sh`:

```bash
./dev.sh backend dev      # Start backend API server on :5171 (live reload)
```

## Architecture

The backend is organized into **Feature-First** domain modules:

```
backend/src/
├── core/                 # Shared utilities (DbPool, AppState, ApiResponse, Error)
├── features/             # Self-contained domain features
│   ├── auth/             # Authentication, sessions, credentials
│   └── family/           # Family members and accounts
│       ├── mod.rs        # Router export
│       ├── db.rs         # PURE SQLx queries (Only place where SQL lives)
│       ├── models.rs     # Serde structs mirroring frontend types.ts
│       └── routes.rs     # Thin Axum handlers (Zero SQL)
└── main.rs               # Server bootstrap and router assembly
```

## Common Workflows

```bash
./dev.sh backend check    # Run cargo check and clippy
./dev.sh backend test     # Run cargo unit tests
./dev.sh backend flint    # Format and lint backend code
./dev.sh backend add <crate> # Add a new cargo dependency
```

Run `./dev.sh backend --help` for all options.
