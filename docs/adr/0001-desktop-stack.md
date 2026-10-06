# ADR-0001: Desktop Stack

Status: Accepted for MVP  
Date: 2026-10-07

## Decision

- Tauri 2
- React
- TypeScript
- Rust core
- Monaco Editor
- xterm.js
- SQLite

## Rationale

複数CLIのprocess/PTY制御、macOS/Windows配布、ローカルファイルアクセス、安全なCore分離を両立するため。

Electronは候補だが、MVPではRust coreとの統合と配布サイズを重視しTauriを採用する。

## Consequences

- Rust/TypeScript双方の境界設計が必要
- Windows/macOS CIが必須
- Tauri plugin権限設定をsecurity review対象にする
- 実装開始時に各依存の最新安定版を再確認しlockfile固定する
