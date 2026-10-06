# GitHub Development Workflow

Issue: #19  
Status: Active after merge  
Updated: 2026-10-07

## 1. Principle

All non-trivial changes follow:

```text
Issue
  -> ownership / labels / priority
  -> basic design
  -> detailed design
  -> normal branch
  -> implementation or documentation
  -> test / validation
  -> adversarial review
  -> Pull Request
  -> review / fix
  -> merge
```

git worktree is not used.

Security vulnerabilities that could expose users must not be reported in public Issues. Use GitHub private Security Advisories.

## 2. Issue title

Priority prefix is required:

- `[P0]` — blocker, critical path, architecture prerequisite, data-loss/security-adjacent defect
- `[P1]` — high priority
- `[P2]` — normal priority

Optional type prefix follows priority:

- `[BUG]`
- `[ARCH]`
- `[DOCS]`

Examples:

```text
[P0] [ARCH] Define provider transport abstraction
[P1] Add provider status panel
[P0] [BUG] Failover can replay a write operation
[P2] [DOCS] Expand Windows setup guide
```

## 3. Labels

Every Issue must have at least one type label.

Current baseline labels:

- `enhancement` — feature, architecture, refactor, operational improvement
- `bug` — reproducible defect/regression
- `documentation` — docs/README/ADR/contributor workflow

Issue Forms add these labels automatically.

Priority currently uses the title prefix so the workflow does not depend on custom labels. Custom priority labels may be added later without changing the title convention.

Labels may be expanded as the project grows, for example provider/platform/area labels.

## 4. Assignee / Ownership

Assignee means **current implementation owner**, not permanent ownership.

Rules:

1. If a maintainer is actively taking the Issue, assign that maintainer.
2. If an external contributor wants to work on an Issue, they should comment first; after scope/ownership is agreed, assign that contributor.
3. An unclaimed public Issue may remain unassigned.
4. Once implementation starts, the Issue should have an assignee.
5. If work is handed over, update the assignee and leave a short handoff comment when useful.
6. Do not hard-code a particular account in Issue templates.
7. PR ownership should be consistent with Issue ownership, or the handoff must be documented.

This permits community contribution without making `nntokyo` the permanent owner of every Issue.

## 5. Required Issue content

Non-trivial feature/architecture Issue:

- Background
- Objective
- Scope / Non-Scope
- Priority
- Labels
- Ownership / assignee state
- Basic design
- Detailed design
- Acceptance criteria
- Dependencies
- Test / validation plan
- Security impact
- macOS / Windows compatibility
- Provider / CLI compatibility when relevant

Bug Issue additionally requires:

- Environment
- Reproduction
- Expected behavior
- Actual behavior
- Evidence with secrets removed
- Fact vs hypothesis separation
- Regression validation

Documentation Issue requires:

- source of truth
- affected files
- content structure
- link/syntax/consistency validation

## 6. When to split an Issue

Split when:

- implementation can be independently delivered/reviewed
- separate components have different risk profiles
- one Issue would require multiple unrelated PRs
- architecture must land before implementation
- a follow-up is P1/P2 and not required for the current acceptance criteria

Use a parent Epic/design Issue when multiple child Issues share one goal.

## 7. Branches

Use normal Git branches, not git worktree.

Examples:

```text
feat/issue-13-session-identity
fix/issue-4-failure-classifier
docs/issue-19-github-workflow
```

A branch should map to a clear Issue or tightly scoped outcome.

## 8. Pull Requests

PRs must use `.github/pull_request_template.md`.

Required:

- Related Issue
- Summary
- current ownership / handoff state
- basic design
- detailed design
- changes
- test/validation result
- macOS impact
- Windows impact
- provider/CLI compatibility
- security/permission impact
- adversarial review
- merge checklist

Use `Closes #N` when the PR fully satisfies the Issue.

Do not close an Issue from a PR that only partially implements its acceptance criteria.

## 9. Adversarial Review

P0 and behavior-changing PRs require adversarial review.

Source of truth:

- `docs/adversarial-review.md`

At minimum challenge:

- Git optional boundary
- session identity and stale resume
- concurrent writer behavior
- duplicate side effects
- permission escalation
- crash/recovery
- path/process/security risks
- macOS/Windows differences
- provider compatibility

A review comment or PR section must record the result.

## 10. Provider / CLI compatibility

Provider CLIs change frequently.

For implementation PRs:

- verify current official documentation
- verify actual capability where possible
- prefer capability probe over version-only assumptions
- add/update parser fixtures
- record unsupported behavior
- do not rely on undocumented/private interfaces as stable contracts

## 11. Triage

When an Issue is opened:

1. confirm title priority
2. confirm type label
3. check duplicate/parent dependency
4. confirm security-sensitive content is appropriate for public Issue
5. clarify scope and acceptance criteria
6. assign current implementation owner if work is starting
7. leave unassigned if openly available for contributors

## 12. Merge policy

Before merge:

- no unresolved blocker
- Issue relationship is correct
- ownership/handoff is clear
- tests/validation passed
- adversarial review completed where required
- documentation updated
- compatibility/security impact recorded

Squash merge is preferred for focused Issue-driven PRs unless preserving commit history is materially useful.
