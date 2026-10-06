# ADR-0004: Git Optional Workspace

Status: Accepted  
Date: 2026-10-07

## Decision

Gitはoptional capabilityとし、Core要件にしない。
git worktreeは使用しない。

## Modes

Local Workspace:
- Gitなし
- checkpoint/recoveryはOrchestratorが提供

Repository:
- Git status/diff/branchを追加

GitHub Repository:
- Issue/PR workflowを追加

## Consequences

Gitなしでも安全なrollback/recovery機構が必要。
RepositoryHost integrationはCoreから分離する。
