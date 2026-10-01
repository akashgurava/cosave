# Dual-Tier Rustdoc Architecture, Living Contract Specifications, and Structured Ingress/Egress Sections

Rust documentation comments are treated as authoritative, living executable contracts: every module begins with an outer doc comment (`//!`) establishing architecture and capability bullets, every item carries an inner doc comment (`///`) with structured markdown sections (`# Ingress`, `# Returns`, `# Errors`, `# Security & Access Control`, `# Invariants`), and documentation must stay strictly synchronized with implementation refactorings.

## Context & Decision

Ad-hoc, missing, or stale code documentation leads to cognitive drift, obscured endpoint requirements, undocumented error contracts, and invisible architecture invariants. When route prefixes change (e.g. `/categories` to `/config`), transactions collapse into single-shot queries, or endpoints adopt Command-Query Separation (CQS), outdated doc comments actively mislead engineers and AI agents.

We established four architectural standards:

1. **Dual-Tier Documentation Coverage (`//!` and `///`)**:
   - **Outer Module Comments (`//!`)**: Every Rust source file begins with an outer module-level doc comment. The first line is an active 1-sentence summary, followed by architectural context and bulleted highlights detailing design decisions, CQS invariants, and capability boundaries.
   - **Inner Item Comments (`///`)**: Every public, `pub(crate)`, `pub(super)`, and non-trivial private item (functions, structs, enums, traits, methods, type aliases, constants) carries a structured doc comment starting with a concise third-person present-tense summary.
2. **Standardized Sectional Hierarchy**:
   - **HTTP Route Handlers**: Must declare canonical routes and aliases, security/role/scoping requirements (`# Security & Access Control`), input extractors (`# Ingress`), wire envelopes and CQS guarantees (`# Returns`), and failure status codes mapped to exact error variants (`# Errors`).
   - **Database Routines**: Must document the query execution model (single-shot atomic query with inline subqueries vs multi-statement transaction), rows-affected validation, SQLite engine constraint classification (`is_unique_violation`, `is_foreign_key_violation`), `# Ingress`, `# Returns`, and `# Errors`.
   - **Schema Migrations**: Must document `# Database Objects Created` (tables, columns, constraints, foreign key cascades `ON DELETE CASCADE|RESTRICT`, indexes, views), `# Invariants` (transaction lifecycle, idempotency, dedicated action tokens), and `# Errors`.
   - **Models & Value Objects**: Must document model tiers (Wire DTO vs Presentation Entity vs DB Row), canonical view column projections, and Parse-Don't-Validate invariants (`try_new`).
   - **Error Enums**: Must document failure domains, screaming status codes, compile-time action tokens, and exact trigger conditions per variant.
3. **Living Documentation & Zero Stale Contracts**:
   - Doc comments are authoritative contract specifications. Modifying code behavior, route hierarchies, or database execution models without updating doc comments in the same changeset is prohibited. Stale doc comments are treated with the same severity as compiler type errors.
4. **Rustdoc Intra-Doc Cross Linking**:
   - Types, functions, and error variants referenced in documentation must use intra-doc markdown links (`[`TypeName`]`) to ensure compiler-verified cross-referencing across subsystem boundaries.

## Consequences

- API contracts, route aliases, and authentication requirements are readable directly from handler source files without guessing.
- Database execution semantics (single-shot subquery vs open transaction) are explicitly visible, preventing accidental transaction creep.
- Error handling is transparent: client consumers and black-box tests have an authoritative specification of HTTP status codes and domain error variants.
- Intra-doc links are verified during documentation and test builds, preventing dangling type references.
- Agent and developer navigation is dramatically sharpened by consistent module overviews and structured section headers.
