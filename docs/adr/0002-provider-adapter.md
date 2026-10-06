# ADR-0002: Provider Adapter Boundary

Status: Accepted for MVP  
Date: 2026-10-07

## Decision

Codex / Claude Code / Grok / Antigravity固有処理をProvider Adapterへ隔離する。

CoreはCLI command名、flag、vendor JSON schemaを知らない。

## Adapter responsibilities

- binary/version/auth/capability probe
- session create/resume
- model/effort mapping
- quota probe
- vendor failure hint
- provider event -> normalized event

Transportは別interfaceとする。

## Consequences

新Provider追加時にCore変更を最小化できる。
CLI仕様変更はfixture testとAdapter修正に閉じ込める。
