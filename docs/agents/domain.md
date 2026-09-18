# Domain Docs

Guidelines for how autonomous agents consume domain documentation and Architecture Decision Records (ADRs) when exploring the CoSave codebase.

## Sources to Read Before Exploration

- **[`CONTEXT.md`](../../CONTEXT.md)**: Authoritative project domain glossary and ubiquitous language.
- **`docs/adr/`**: Architectural decision records for active and past architectural decisions.

If either is absent or missing a concept, proceed silently. The `/domain-modeling` skill resolves new terms and records ADRs when decisions land.

## Ubiquitous Language

- Always use the domain terms defined in [`CONTEXT.md`](../../CONTEXT.md).
- Avoid listed forbidden synonyms (e.g., use `Family`, never `Group` or `Team`; use `Member`, never `Profile`).
- If a required concept is not in the glossary, note the candidate term for `/domain-modeling`.

## Flagging ADR Conflicts

If proposed implementation details contradict an existing ADR in `docs/adr/`, flag it explicitly:
> _Contradicts ADR-XXXX (<title>), but reopening because: <rationale>_
