# Session Identity and Resume Validation

Issue: #13  
Status: Implemented after merge  
Updated: 2026-10-07

## Identity model

```text
Workspace
 └─ Task
     └─ Provider
         └─ ProviderSession[]
```

A provider-native session is never selected by provider/workspace alone.

The minimum identity boundary is:

- Workspace ID
- Task ID
- Provider ID
- Provider Session ID
- native Provider session ID
- canonical working directory

Identity values are non-empty by construction. Empty or whitespace-only IDs are rejected both by constructors and serde deserialization.

## Workspace path policy

The Core resolves an existing workspace with filesystem canonicalization.

Safety rules:

- symlink aliases resolve to the same target identity;
- path equivalence relies on the OS filesystem canonicalization result rather than custom Unicode case folding;
- Windows CI verifies that case aliases canonicalize to the same workspace;
- macOS/Unix paths are not blindly lowercased because case sensitivity is volume-dependent;
- a moved or renamed workspace is treated as a **new location** until a future persistence/UI flow performs an explicit relink;
- Git is not required for workspace identity.

Treating a moved workspace as new is intentionally conservative. It avoids silently resuming a provider session against a different path.

## Resume decisions

The Core returns one of four outcomes.

### Resume

All identity and current-state checks match.

### ResumeWithDrift

The native session may still be resumed, but the caller must inject/reconcile the detected drift.

Current drift reasons:

- session previously marked stale
- workspace fingerprint changed
- Git context changed
- permission policy revision changed
- Provider capability/version requires revalidation

### StartNew

The selected session belongs to the correct logical identity, but native resume is no longer safe/possible.

Reasons:

- workspace location changed
- native session was deleted
- Provider resume is unsupported
- stored session is missing/closed
- Provider compatibility is explicitly incompatible

### Reject

The selected metadata belongs to a different logical identity.

Reasons:

- Workspace mismatch
- Task mismatch
- Provider mismatch

A Reject is deliberately different from StartNew. The caller must not silently consume or reinterpret the mismatched session.

## Continue-most-recent rule

Provider-native commands such as "continue most recent" are not trusted as the Task identity source.

The Orchestrator must first select the expected Task/Provider session record and then validate the Provider-native result against that identity.

This prevents two Tasks in the same directory from resuming each other's conversation.

## Provider version policy

Issue #13 does not parse Provider-specific semantic versions.

Instead, Provider integration supplies one of:

- Compatible
- RequiresRevalidation
- Incompatible

Provider-specific interpretation belongs to #5.

This keeps Core independent from vendor version syntax.

## Fingerprint policy

`WorkspaceFingerprint` is opaque at this layer.

The actual fingerprint algorithm is owned by later filesystem/persistence work. Resume validation only compares snapshots.

This avoids hard-coding an expensive whole-workspace hash into the identity contract.

## Git policy

Git context is optional.

When present, branch/HEAD drift is reported as ResumeWithDrift.

When absent in both stored/current state, Git has no effect on resume eligibility.

## Permission policy integration

Issue #13 stores only an opaque permission-policy revision.

The actual permission model and mapping are owned by #17.

A permission revision change causes ResumeWithDrift rather than automatic resume without notice.

## Explicit non-goals

Issue #13 does not implement:

- SQLite persistence (#8)
- Provider CLI session discovery (#5)
- durable session registry UI (#3)
- Provider-specific resume commands (#5/#3)
- permission policy semantics (#17)
- workspace fingerprint generation algorithm

## Adversarial invariants

- a session from Task A can never validate for Task B;
- provider-native "most recent" does not bypass Task identity;
- a moved workspace does not silently resume at a new path;
- deleted native sessions fall back to StartNew;
- Git absence never blocks Local Workspace Mode;
- Provider version drift is explicit;
- permission drift is explicit;
- no Provider-specific command or credential exists in this Core module.
