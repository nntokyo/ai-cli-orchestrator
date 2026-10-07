# ADR-0009: Typed Frontend/Core Protocol

Status: Accepted for bootstrap phase  
Date: 2026-10-07  
Issue: #30

## Decision

Define the frontend/Core data contract in a dedicated Rust crate:

```text
crates/orchestrator-protocol
```

Rust types are the source of truth. TypeScript bindings are generated with `ts-rs` and committed under:

```text
src/generated/protocol/
```

## Why

The frontend and Rust core will exchange Task, Provider, Session, routing, permission, recovery, and repository-host messages.

Maintaining equivalent Rust and TypeScript interfaces by hand would allow silent drift. A single Rust source of truth makes serialization behavior and frontend types reviewable together.

## Initial contract

The bootstrap contract contains:

- protocol version
- request ID
- `CoreRequest`
- `CoreEvent`
- protocol error shape

The contract is intentionally small. Runtime Provider/Task messages are added by their owning Issues.

## Generation

`ts-rs` exports TypeScript during Rust tests.

Rules:

- edit Rust protocol types, not generated TypeScript;
- generated TypeScript is committed;
- CI regenerates bindings and fails on drift;
- generated files are excluded from Biome formatting/linting;
- TypeScript still type-checks against generated files when they are imported.

## Permission boundary

A protocol type is **not** a runtime capability.

Defining a request such as `GetBootstrapStatus` does not make it callable from the webview.

Tauri IPC remains disabled in the bootstrap until an owning Issue explicitly introduces:

1. a command/event implementation;
2. a least-privilege capability;
3. permission mapping;
4. validation and adversarial tests.

This keeps ADR-0007 (Unified Permission Policy) authoritative.

## Versioning

`PROTOCOL_VERSION` starts at `1`.

Compatibility rules will be expanded when the first runtime IPC surface lands. Until then:

- additive type work may remain protocol version 1 when wire compatibility is unchanged;
- incompatible serialization changes require an explicit versioning decision;
- Provider-native session IDs and secrets must not be exposed merely because a frontend type exists.

## Tooling

Bootstrap versions verified on 2026-10-07:

- `ts-rs` 12.0.1
- `serde` 1.0.229
- `@biomejs/biome` 2.5.15

Biome owns frontend formatting/linting. Rust formatting/linting remains rustfmt/clippy.

## Consequences

Benefits:

- no hand-maintained duplicate TypeScript contract;
- serialization and type changes reviewed together;
- generated drift becomes a CI failure;
- permission remains independent from type availability.

Costs:

- generated files are committed;
- protocol changes update lockfiles/bindings;
- ts-rs limitations must be considered when choosing wire shapes.

## Alternatives considered

### Handwritten TypeScript interfaces

Rejected because the Rust and TypeScript shapes can drift silently.

### Generate Rust from TypeScript

Rejected because the execution/persistence/domain core is Rust-first and serde serialization is authoritative.

### Generate bindings only during frontend build

Rejected because committed generated files make public PR diffs inspectable and let CI detect stale generation explicitly.

## Security

- no generated type grants IPC permission;
- secrets/tokens must not be included in public frontend contracts without an explicit security review;
- generated files are never trusted as manually edited source;
- protocol input validation remains mandatory when runtime IPC is introduced.

## Licensing

- ts-rs is MIT licensed.
- Biome is MIT OR Apache-2.0.
- Project-generated bindings remain part of this Apache-2.0 project.

## References checked 2026-10-07

- ts-rs: https://github.com/Aleph-Alpha/ts-rs
- ts-rs docs: https://docs.rs/ts-rs/
- Biome: https://biomejs.dev/
- Biome CLI: https://biomejs.dev/reference/cli/
