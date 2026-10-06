# ADR-0007: Unified Permission Policy

Status: Accepted for MVP  
Date: 2026-10-07

## Decision

Provider固有のsandbox/approvalを直接UIモデルにせず、Core共通policyへ正規化する。

Common permission:
- read
- workspace_write
- command_execute
- network
- external_path
- destructive
- secret_access
- repository_write
- remote_action

## Rule

ProviderがCore policyを安全に表現できない場合、権限を拡大して実行せずpermission incompatibleとする。

## Consequences

Provider mapping matrixとsecurity testが必要。
failover後もpermission escalationを禁止する。
