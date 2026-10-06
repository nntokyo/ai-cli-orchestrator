# ADR-0005: Single Writer Per Workspace

Status: Accepted for MVP  
Date: 2026-10-07

## Decision

git worktreeを使わないため、1 workspaceにつきWRITE leaseは原則1つとする。

READ ONLY agent / review agentは並列可能。

## Failover

Provider切替前に旧writerの停止とlease releaseを確認する。
crash時はheartbeat + process existenceでstale lockを判断する。

## Consequences

同一workspaceで複数Agentによる独立実装を完全並列にはしない。
一方で実装 + 敵対レビュー等のread-only並列は可能。
