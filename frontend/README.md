# CoSave Frontend

SvelteKit 2 + Svelte 5 (Runes) web interface for CoSave, styled with Tailwind CSS v4 and shadcn-svelte.

## Quick Start

Always run commands from the repository root using `./dev.sh`:

```bash
./dev.sh ui dev           # Start Vite dev server on http://localhost:5172
```

In development, Vite serves on port `5172` and transparently proxies `/api/*` requests to the backend on port `5171`.

## Architecture

The frontend mirrors the backend's **Feature-First** structure:

```
frontend/src/lib/
├── components/
│   └── ui/               # Upstream shadcn-svelte primitives (CLI-only, IMMUTABLE)
└── features/             # Self-contained domain features
    └── family/           # Family & accounts feature
        ├── components/   # Feature-specific UI components
        ├── api.ts        # Typed apiFetch calls
        ├── types.ts      # TypeScript interfaces matching backend models.rs
        └── mock.ts       # Prototype mock data for Phase 1
```

## Common Workflows

```bash
./dev.sh ui check         # Run svelte-check and TypeScript diagnostics
./dev.sh ui test          # Run Vitest unit tests
./dev.sh ui flint         # Format and lint with Prettier/ESLint
./dev.sh ui shadcn <comp> # Install official shadcn-svelte component
./dev.sh ui add <pkg>     # Add a new pnpm dependency
```

Run `./dev.sh ui --help` for all options.
