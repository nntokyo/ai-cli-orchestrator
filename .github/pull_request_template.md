## Related Issue

Refs #

<!--
Default to "Refs #<issue>".
Only when this PR fully satisfies the Issue acceptance criteria should you replace "Refs" with a GitHub closing keyword.
For partial PRs, never place a closing keyword immediately before the real Issue reference, even in a negated sentence.
-->

## Summary

<!-- What does this PR change and why? Keep this outcome-oriented. -->

## Ownership

- Issue assignee / current implementation owner:
- PR author / owner:
- Handoff notes, if ownership changed:

## Basic Design

<!-- Components, boundaries, workflow, and important decisions. Link the issue/docs/ADR where possible. -->

## Detailed Design

<!-- Interfaces, state, persistence, failure handling, concurrency, permissions, OS/provider differences. -->

## Changes

- 
- 

## Test / Validation Result

<!-- Provide commands/results. For docs-only changes, describe syntax/link/consistency validation. -->

- [ ] Unit tests
- [ ] Integration tests
- [ ] E2E / manual verification
- [ ] Docs-only validation
- [ ] N/A explained below

Result / evidence:

## Platform Compatibility

### macOS

<!-- Apple Silicon / Intel, shell/PATH/process behavior, or N/A. -->

### Windows

<!-- x64/arm64, PowerShell/cmd/ConPTY/path/process behavior, or N/A. -->

## Provider / CLI Compatibility

<!-- Codex / Claude Code / Grok / Antigravity impact. Include verified versions/capabilities when relevant. -->

- Codex:
- Claude Code:
- Grok:
- Antigravity:
- Other / N/A:

## Licensing / Third-Party Impact

<!-- New dependencies/assets, license compatibility, attribution/NOTICE, provider redistribution terms, or N/A with reason. -->

- [ ] No new third-party dependency/asset, or license obligations were reviewed
- [ ] Required attribution / NOTICE impact was reviewed
- [ ] Provider CLI/API redistribution or trademark terms were reviewed when relevant

Details:

## Security / Permission Impact


<!-- Secrets, subprocess, filesystem, network, sandbox, permission mapping, remote actions, or N/A with reason. -->

## Adversarial Review

<!-- Try to break the change. Refer to docs/adversarial-review.md. -->

Checked:
- [ ] Git remains optional; no accidental git worktree dependency
- [ ] Session/task identity cannot cross-contaminate work
- [ ] No unsafe duplicate side effect on retry/failover
- [ ] Permission does not expand across provider switch
- [ ] macOS/Windows differences considered
- [ ] Secret/log/path/process risks considered
- [ ] Public documentation does not claim unimplemented behavior as shipped

Findings / fixes:

## Merge Checklist

- [ ] Related Issue is linked; complete PRs may use a closing keyword, partial PRs use `Refs #...` only
- [ ] Issue labels are set
- [ ] Issue assignee represents the current implementation owner, or handoff is documented
- [ ] Basic design is complete
- [ ] Detailed design is complete
- [ ] Tests/validation are complete
- [ ] Adversarial review is recorded
- [ ] Latest relevant provider/CLI compatibility was checked for code changes
- [ ] Licensing / third-party obligations were reviewed
- [ ] No unresolved blocker remains
