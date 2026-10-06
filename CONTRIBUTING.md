# Contributing

## Development policy

このrepositoryでは原則として以下をワンセットとする。

1. Issue
2. 基本設計
3. 詳細設計
4. 通常branch
5. 実装
6. test / lint / build
7. 敵対レビュー
8. Pull Request
9. review / fix
10. merge

git worktreeは使用しない。

## Branch examples

- `feat/issue-13-session-identity`
- `fix/issue-4-router-classifier`
- `docs/issue-2-architecture`

## Pull Request requirements

- Issueをlink
- 変更理由
- 設計影響
- test結果
- macOS/Windows影響
- security影響
- Provider CLI仕様根拠
- 敵対レビュー結果

## Provider changes

CLI仕様は変化しやすいため、実装時に公式documentationを再確認する。

- undocumented/private interfaceを前提にしない
- capability probeを優先
- parser fixtureを追加
- unsupported versionを明示
- 最新安定版でcompatibilityを確認

## Safety

- API key/tokenをcommitしない
- shell command文字列連結を避ける
- permissionをProvider切替で拡大しない
- write failoverは副作用確認後
- Gitなしworkspaceでもrecovery可能にする

## Review

P0変更では `docs/adversarial-review.md` に沿った敵対レビューを必須とする。
