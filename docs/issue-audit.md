# MVP Issue Consistency Audit

Issue: #25  
Status: Audit complete  
Audit date: 2026-10-07

## Purpose

This audit checks the open MVP Issues against the accepted architecture, detailed design, ADRs, GitHub workflow, security model, and Apache-2.0 licensing policy before implementation begins.

## Sources of truth

- `docs/basic-design.md`
- `docs/detailed-design.md`
- `docs/adr/0001-0007`
- `docs/adversarial-review.md`
- `docs/development-workflow.md`
- `docs/licensing.md`
- Epic #1

## Resolved inconsistencies

### Session identity

Issue #3 still described `workspace + provider -> session`, while ADR-0003 and the detailed design use:

```text
Workspace -> Task -> Provider -> ProviderSession[]
```

#3 was corrected and now depends on #13 for identity validation.

### Persistence

Issue #8 did not include entities introduced by the detailed design.

It now covers:
- task
- execution_run
- checkpoint
- workspace_lease
- permission_policy
- repository_host_link
- schema migrations

Privacy/retention ownership is separated into #27.

### Transport

Initial documents over-emphasized subprocess/PTY.

The current priority is:

1. official native/app/agent protocol;
2. persistent structured stdin/stdout;
3. headless structured process;
4. PTY fallback.

Provider baseline:
- Codex: official App Server / SDK for product integration where supported;
- Claude Code: structured stream-json integration;
- Grok: ACP for IDE/tool integration, structured headless fallback;
- Antigravity: persistent stream-json input/output, structured headless fallback.

Every implementation PR must revalidate current official provider documentation and installed CLI capabilities.

### Concurrency

No git worktree is used.

#14 owns Single Writer semantics:
- one WRITE lease per workspace by default;
- multiple effective READ-only reviewers may run concurrently;
- provider failover must stop the old writer before transferring the lease;
- external editor/terminal changes produce conflict/stale-state signals.

### Execution lifecycle

The detailed design had a run state machine but no implementation owner.

#26 now owns:
- Task/Run lifecycle;
- pause/cancel/interrupt;
- retry/reroute;
- idempotency and duplicate-side-effect guard;
- restart reconciliation.

### Local privacy

Not storing API keys is insufficient because prompts, code, command output, checkpoints, and logs may contain sensitive information.

#27 now owns:
- retention;
- redaction;
- local deletion;
- raw event policy;
- database/temp/checkpoint permissions.

### GitHub integration

#7 previously mixed local Git and GitHub automation without defining the host boundary.

#28 now owns:
- RepositoryHostAdapter;
- authentication capability;
- fork/upstream identity;
- remote mutation permission;
- API/rate-limit/offline failure behavior.

Local development must continue when GitHub is unavailable.

### Bootstrap

The stack ADR existed but there was no owner for creating the actual Tauri/React/Rust project skeleton.

#30 now owns:
- workspace layout;
- package manager;
- lockfiles;
- toolchain baseline;
- minimal app;
- typed UI/Core boundary;
- smoke CI;
- Apache-2.0 package metadata.

### Release distribution

#9 is developer build/CI scope.

#29 separately owns production distribution:
- signing/notarization;
- secure updater;
- release provenance;
- SBOM;
- third-party license inventory/NOTICE.

## Adversarial findings incorporated

The following failure modes are now explicit requirements:

- a test/build failure must not be classified as a provider outage;
- a provider can soft-deny a tool without a non-zero process exit, so exit code alone is not success;
- `continue most recent` must not cross Task identity;
- provider/model listing is not necessarily proof of entitlement;
- failover must not repeat confirmed side effects;
- provider switching must not widen permissions;
- a previous provider process must not retain write access after lease transfer;
- GitHub failure must not block Local Workspace Mode;
- raw provider events/logs/checkpoints can contain secrets even when tokens are not stored;
- provider binaries are not bundled by default and remain under vendor terms;
- dependency licenses/NOTICE/SBOM become release gates.

## Ownership and labels

- Active critical-path Issues use the current implementation owner.
- Assignee is not permanent ownership; it can change on contributor handoff.
- Unclaimed future work may remain unassigned.
- Type labels use the repository baseline labels until more granular labels are introduced.

#29 is intentionally allowed to remain unassigned while it is a later release-phase P1 item.

## Implementation order

The order is dependency-oriented rather than strictly numeric.

### Phase 0: repository/application baseline

- #30 Application Bootstrap

### Phase 1: identity and integration boundaries

- #13 Workspace / Task / Session Identity
- #15 Transport Abstraction
- #17 Permission / Sandbox Policy

These define the contracts required by providers and execution.

### Phase 2: execution safety

- #26 Task Execution Engine
- #14 Single Writer
- #16 Non-Git Checkpoint / Crash Recovery

### Phase 3: routing and providers

- #4 Router / Quota / Failure Classifier
- #5 Provider Adapters
- #3 Native Resume Registry

Some work may proceed in parallel once Phase 1 interfaces are stable.

### Phase 4: persistence and privacy

- #8 SQLite / Portable Context
- #27 Local Data Privacy

Persistence schema should not be considered complete until retention/deletion behavior is included.

### Phase 5: repository integration and UI

- #28 GitHub Repository Host Adapter
- #7 Local/Git/GitHub Workflow
- #6 Desktop UI

### Phase 6: platform CI and security closure

- #9 macOS / Windows Build & CI
- #10 Security / Test / Adversarial Review

Security review is continuous, but #10 closes only after implemented boundaries have tests.

### Phase 7: distribution

- #29 Release Distribution / Signing / Updater / SBOM

This is P1 during development but becomes a blocker before public production binaries.

## Explicitly deferred from MVP

The following remain non-goals unless promoted by a later Issue:

- Linux formal support;
- VS Code extension compatibility;
- multi-root workspace;
- GitLab/Bitbucket implementation;
- cloud sync of local history;
- provider CLI bundling;
- mandatory CLA/DCO;
- git worktree-based parallel writers.

## Official capability references checked

Checked on 2026-10-07:

- OpenAI Codex App Server / platform guidance: https://developers.openai.com/blog/codex-as-a-platform
- OpenAI Codex App Server auth/integration reference: https://developers.openai.com/siwc/token-sharing-open-source/codex-app-server
- Anthropic Claude Code CLI reference: https://docs.anthropic.com/en/docs/claude-code/cli-usage
- xAI Grok CLI reference: https://docs.x.ai/build/cli/reference
- xAI Grok headless/ACP: https://docs.x.ai/build/cli/headless-scripting
- xAI Grok permissions: https://docs.x.ai/build/features/permissions
- Google Antigravity headless: https://antigravity.google/docs/cli/headless/
- Google Antigravity resume: https://antigravity.google/docs/cli/commands/resume/

Provider specifications are fast-moving. This audit records architecture assumptions; implementation PRs must re-check the official docs and actual installed CLI.


## Dependency graph correction — 2026-10-07

A second audit found circular blocking dependencies in the initial Issue relationships:

- #5 <-> #17
- #4 <-> #26
- #14 <-> #16
- #16 <-> #27
- #8 <-> #13

The resolution is documented in `docs/dependency-graph.md`.

Dependency semantics are now split into:

- **Prerequisites** — blocking DAG edges
- **Integration follow-ups** — later wiring/consumption, not blocking edges

#8 was promoted from P1 to P0 because Native Resume #3 requires durable session persistence and #8 is on that critical path.

The current implementation begins with provider-independent contracts #13 / #15 / #17 after the completed #30 bootstrap.
