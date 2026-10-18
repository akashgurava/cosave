# CoSave Clean-Slate Implementation Tracker

> **Status**: Living Execution Roadmap  
> **Reference Specification**: [HANDOFF.md](HANDOFF.md)  
> **Golden Rules**: 
> 1. Complete steps strictly in order. Mark tasks `- [x]` only after verifying execution.
> 2. Scaffold crates and dependencies exclusively via Cargo/dev.sh CLI commands without hardcoded versions.
> 3. Zero warning suppressions (no `#[allow(...)]`); resolve all compiler diagnostics at their root cause.
> 4. Default to challenging shortcuts and consult the maintainer if 2 debugging attempts fail.

---

## Phase 1: Workspace & Toolchain Foundation

### 1.1 Toolchain & Workspace Scaffolding
- [ ] Install toolchain prerequisites:
  ```bash
  cargo install cargo-workspaces
  rustup target add wasm32-unknown-unknown
  cargo install trunk
  ```
- [ ] Initialize repository `.gitignore`:
  - Ensure `/target/`, `dist/`, `.scratch/`, `*.db`, `*.db-journal`, `*.db-wal`, `*.db-shm` are ignored.
- [ ] Create root `./dev.sh` orchestrator script with execution permissions (`chmod +x dev.sh`).
- [ ] Implement workspace creation in `./dev.sh`:
  - Command: `./dev.sh ws create`
  - Writes root `Cargo.toml` with:
    ```toml
    [workspace]
    resolver = "3"
    members = ["crates/*"]

    [workspace.package]
    version = "0.1.0"
    edition = "2024"
    authors = ["CoSave Team"]
    license = "MIT OR Apache-2.0"
    ```
- [ ] Implement crate addition in `./dev.sh`:
  - Command: `./dev.sh ws add --lib crates/<name>` and `./dev.sh ws add --bin crates/<name>`
  - Uses `cargo new --lib crates/<name> --name cosave-<name>` or `cargo new --bin crates/<name> --name cosave-<name>`.
  - Ensures each crate's `Cargo.toml` uses `edition.workspace = true` and `version.workspace = true`.
- [ ] Implement target-scoped action routing in `./dev.sh`:
  - Syntax: `./dev.sh <target> <action>`
  - Allowed Targets: `ws` (entire workspace) | `core` | `db` | `api` | `ui`
  - Allowed Actions: `check`, `build`, `fmt`, `test`, `lint`
  - Target Routing Rules:
    - Target `ui`: `./dev.sh ui check` executes `cargo check -p cosave-ui --target wasm32-unknown-unknown`.
    - Target `ws`: `./dev.sh ws check` executes `cargo check --workspace` AND `cargo check -p cosave-ui --target wasm32-unknown-unknown`.
    - Target `ws` test: `./dev.sh ws test` executes `cargo test --workspace --exclude cosave-ui`.
    - Lint action: `./dev.sh <target> lint` executes `cargo clippy -p <pkg> --all-targets -- -D warnings`.
    - Format action: `./dev.sh <target> fmt` executes `cargo fmt -p <pkg>`.
- [ ] Scaffold the 4 workspace crates via `./dev.sh`:
  ```bash
  ./dev.sh ws add --lib crates/core
  ./dev.sh ws add --lib crates/db
  ./dev.sh ws add --bin crates/api
  ./dev.sh ws add --lib crates/ui
  ```
  *Completion Criterion*: Running `cargo metadata --format-version 1` lists all 4 packages (`cosave-core`, `cosave-db`, `cosave-api`, `cosave-ui`) as active workspace members.

### 1.2 Module Documentation & Baseline Verification
- [ ] Author minimal compilable entrypoints:
  - `crates/api/src/main.rs`:
    ```rust
    fn main() {
        println!("cosave v0.1.0");
    }
    ```
  - `crates/core/src/lib.rs`: Minimal empty library file.
  - `crates/db/src/lib.rs`: Minimal empty library file.
  - `crates/ui/src/lib.rs`: Minimal empty library file.
- [ ] Add verbose outer module doc comments (`//!`) to every `lib.rs` and `main.rs` structured with:
  1. `# Architectural Role`: Plain explanation of the crate's responsibility in the system.
  2. `# Boundary Invariants`: Strict rules regarding dependencies and forbidden imports.
  3. `# Exposed Surface`: Summary of what types or interfaces the module provides.
- [ ] Add strict compiler attributes to the top of every `lib.rs` and `main.rs`:
  ```rust
  #![deny(dead_code)]
  #![deny(unused)]
  ```
- [ ] Run initial baseline verification on empty workspace:
  ```bash
  ./dev.sh ws check
  ./dev.sh ws test
  ./dev.sh ws lint
  ```
  *Completion Criterion*: All commands exit code 0 with 0 errors and 0 warnings.

### 1.3 Docker Container Foundation
- [ ] Create multi-stage `Dockerfile` with static musl compilation for `scratch`:
  - Stage 1 (Builder):
    - Base: `rust:bookworm`.
    - Adds musl target: `rustup target add x86_64-unknown-linux-musl`.
    - Compiles static release executable:
      ```bash
      cargo build --release --target x86_64-unknown-linux-musl -p cosave-api
      ```
  - Stage 2 (Runtime):
    - Base: `FROM scratch`.
    - Copies compiled static binary:
      ```dockerfile
      COPY --from=builder /usr/src/cosave/target/x86_64-unknown-linux-musl/release/cosave /cosave
      ENTRYPOINT ["/cosave"]
      ```
- [ ] Implement Docker support in `./dev.sh`:
  - `./dev.sh docker build`: Builds the baseline image tagged `cosave:latest`.
  - `./dev.sh docker test`: Runs `docker run --rm cosave:latest` and verifies it outputs `cosave v0.1.0` and exits code 0.
  *Completion Criterion*: Running `./dev.sh docker test` outputs `cosave v0.1.0` and terminates with exit code 0.

### 1.4 GitHub Actions CI/CD Pipeline
- [ ] Create `.github/workflows/ci.yml`:
  - Trigger: `on: [push, pull_request]`
  - Runner: `ubuntu-latest`
  - Toolchain setup: `dtolnay/rust-toolchain@stable` with components `rustfmt`, `clippy` and targets `wasm32-unknown-unknown`, `x86_64-unknown-linux-musl`.
  - Cache: `Swatinem/rust-cache@v2`.
  - Workflow Steps:
    1. Code formatting check: `cargo fmt --all -- --check`
    2. Linting gate: `cargo clippy --workspace --all-targets -- -D warnings`
    3. Workspace compilation check:
       - `cargo check --workspace`
       - `cargo check -p cosave-ui --target wasm32-unknown-unknown`
    4. Test execution: `cargo test --workspace --exclude cosave-ui`
    5. Docker container build and test:
       - `docker build -t cosave:ci .`
       - `docker run --rm cosave:ci`
  *Completion Criterion*: `.github/workflows/ci.yml` contains zero YAML syntax errors and matches all 5 verification stages.
