# Database Lifecycle, Transaction Boundaries, Uniform Identifiers, and Domain Value Objects

Database schemas enforce idempotent DDL with explicit foreign key cascades, multi-statement workflows execute inside explicit transaction blocks, entities use standardized prefixed identifiers with UTC epoch seconds timestamps, and domain validation eliminates primitive obsession via constructor-enforced Value Objects ("Parse, Don't Validate").

## Context & Decision

Previous backend features allowed raw primitive types (`String`, `i64`) to represent unvalidated domain concepts across handlers and database queries, risking invalid internal state. Database migrations, transactions, primary key schemes, and timestamp formats lacked uniform architectural constraints, leading to ad-hoc timestamp generators, implicit foreign key behaviors, and unstructured database updates.

We established five architectural standards for data and domain modeling:

1. **Idempotent DDL & Explicit Referential Actions**: All tables, indexes, and views continue to be initialized through `crate::core::create_db_object(action, table, pool, sql)` with compile-time action tokens. Every table and index must be created idempotently (`CREATE TABLE IF NOT EXISTS`, `CREATE INDEX IF NOT EXISTS`). Every foreign key constraint must explicitly declare referential actions (`ON DELETE CASCADE`, `ON DELETE RESTRICT`, or `ON DELETE SET NULL`); implicit SQLite defaults are forbidden. Indexes follow the canonical naming convention `idx_<table_name>_<column1>[_<column2>]`.
2. **Explicit Transaction Boundaries & Atomic Operations**: Any workflow combining multiple write statements (INSERT, UPDATE, DELETE) or executing a state check followed by a mutation must execute within an explicit `pool.begin().await` transaction block. Sub-operations participating in transactions accept `&mut sqlx::Transaction<'_, sqlx::Sqlite>`. External side effects (password hashing, disk I/O, network requests) are strictly banned inside transaction blocks. Each transaction step (`BEGIN`, individual queries, and `COMMIT`) carries a dedicated compile-time action token (`FEATURE.WORKFLOW.TX_BEGIN`, `FEATURE.WORKFLOW.STEP.QUERY`, `FEATURE.WORKFLOW.TX_COMMIT`).
3. **Prefixed Collision-Free Identifiers**: Domain entities use typed, prefixed string identifiers (`usr_<id>`, `cat_<id>`, `sub_<id>`, `tx_<id>`) using collision-free, time-sortable algorithms (UUIDv7 or nanoid). Numeric auto-increment (`INTEGER PRIMARY KEY AUTOINCREMENT`) is reserved strictly for local, non-replicated lookup tables (such as `colors`).
4. **Timestamp SSOT**: All database timestamps are stored as `INTEGER NOT NULL` representing UTC epoch seconds. Timestamps must be sourced exclusively from a centralized helper (`crate::core::time::now_epoch_secs()` or equivalent in `core`). Feature-level ad-hoc timestamp generators or formatting timestamps as ISO strings or floats in SQLite are forbidden.
5. **Domain Invariants & Value Objects ("Parse, Don't Validate")**: Core domain concepts with syntax, format, or length rules (e.g. `CategoryName`, `Username`, `HexColor`, `AmountCents`) must be implemented as private Rust newtypes rather than bare primitives. Value Objects are instantiated exclusively through fallible constructors (`try_new(raw, action) -> Result<Self, FeatureError>`). Construction validates and normalizes (trims) data; once instantiated, invalid domain state is unrepresentable. Financial values must always be represented as integer minor units (`AmountCents(i64)`); floating-point types (`f32`, `f64`) for currency are forbidden.

## Consequences

- Domain models guarantee data correctness at compile and construction time, eliminating repetitive validation checks inside database helpers.
- Database write operations are atomic, eliminating partial writes and database corruption from aborted multi-statement queries.
- Primary keys and foreign keys are predictable, traceable, and collision-free across environments and client transports.
- Timestamp representations are uniform across all SQLite tables, queries, and serialized payloads.
- Long-running transactions and SQLite write locks are prevented by isolating non-database computations outside transaction boundaries.
