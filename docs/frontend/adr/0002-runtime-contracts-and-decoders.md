# Pure-TypeScript Runtime Schema Decoders and Boundary Validation

All external data crossing the frontend boundary (HTTP responses, LocalStorage, URL parameters, WebSockets) is treated as `unknown` and validated through pure-TypeScript runtime schema decoders, mirroring Rust's `serde` deserialization.

## Context & Decision

In traditional TypeScript frontend codebases, network responses are routinely treated as blindly trusted JSON (`(await res.json()) as User`). If the backend schema changes, a field is renamed, or null is returned unexpectedly, the frontend fails unpredictably deep in the component tree with unhelpful errors like `Cannot read properties of undefined`.

We establish a strict "Parse, Don't Validate" boundary:

1. **Zero-Dependency Pure-TypeScript Decoders**:
   - Every domain entity and API payload defines a dedicated decoder function: `parseX(raw: unknown): T`.
   - Decoders verify type primitives (`typeof raw.id === "string"`), field presence, array item validity, and nullability without external runtime bundle overhead.
   - Corrupted or invalid payloads throw typed `ContractViolationError` immediately at the network boundary, capturing the raw offending payload for diagnostics.

2. **Immutable Domain Output**:
   - Decoders return read-only or frozen structures (`Object.freeze(...)`, `readonly T[]`) to prevent unintentional mutations across components.

3. **Wire DTOs vs Domain Entities**:
   - Separate raw network representations from internal presentation models. Decoders bridge the gap, converting wire timestamps (epoch integers) into formatted domain objects and wire strings into branded nominal types.

4. **Structured Error Contract**:
   - API error responses from the backend wrap structured `ErrorPayload { action: string, message: string }`.
   - The frontend API client unpacks these into `ApiError` instances exposing `.action` and `.message` directly for localized UI alerts and toasts.

## Consequences

- Network contract violations fail immediately at the boundary with clear diagnostics rather than causing mysterious rendering glitches later.
- Eliminates heavy schema validator bundles (like Zod or Joi) in favor of zero-overhead, highly performant TypeScript decoders.
- Provides absolute certainty that any data consumed by Svelte components is 100% structurally valid.
