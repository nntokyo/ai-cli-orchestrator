# Contributing

## Development policy

このrepositoryでは原則として以下をワンセットとします。

1. Issue
2. priority / labels / ownership確認
3. 基本設計
4. 詳細設計
5. 通常branch
6. 実装または資料更新
7. test / lint / build / validation
8. 敵対レビュー
9. Pull Request
10. review / fix
11. merge

git worktreeは使用しません。

詳細な運用ルールは [GitHub Development Workflow](docs/development-workflow.md) を参照してください。

## Issue requirements

Issue Formsを使用してください。

最低限:
- priority
- type label
- background / objective
- scope / non-scope
- basic design
- detailed design
- acceptance criteria
- validation plan
- security / compatibility impact

### Assignee / ownership

assigneeは恒久的な所有者ではなく、**現在の実装オーナー**を示します。

- maintainerが着手する場合はそのmaintainerを設定
- 外部Contributorが着手する場合は、scope合意後にそのContributorへ変更
- 未着手の公開Issueはunassignedでもよい
- 引き継ぎ時はassigneeを更新する
- templateへ特定アカウントをhard-codeしない

## Toolchain baseline

Use the versions pinned by the repository:

- Node.js: `.node-version`
- Rust: `rust-toolchain.toml`
- Package manager: pnpm (exact version will be pinned in `package.json` when the scaffold lands)

Do not manually edit generated dependency lockfiles. Regenerate them with the corresponding package manager and review the diff.

See [ADR-0008](docs/adr/0008-project-layout-toolchain.md).

## Branch examples

- `feat/issue-13-session-identity`
- `fix/issue-4-router-classifier`
- `docs/issue-19-github-workflow`

## Pull Request requirements

PR templateを使用してください。

必須:
- Related Issue / `Closes #...`
- ownership / handoff
- basic design
- detailed design
- changes
- test/validation result
- macOS/Windows impact
- Provider CLI compatibility
- security/permission impact
- adversarial review result

## Provider changes

CLI仕様は変化しやすいため、実装時に公式documentationを再確認します。

- undocumented/private interfaceを安定contractとして扱わない
- capability probeを優先
- parser fixtureを追加/更新
- unsupported versionを明示
- 最新安定版でcompatibilityを確認

## Safety

- API key/tokenをcommitしない
- shell command文字列連結を避ける
- permissionをProvider切替で拡大しない
- write failoverは副作用確認後
- Gitなしworkspaceでもrecovery可能にする
- exploitable security vulnerabilityは公開Issueへ投稿しない

## Review

P0およびbehavior-changing変更では [敵対レビュー基準](docs/adversarial-review.md) に沿った敵対レビューを必須とします。


## Contribution license

This project is licensed under the Apache License, Version 2.0 (`Apache-2.0`).

Unless you explicitly state otherwise in a manner accepted by the project, a contribution intentionally submitted for inclusion in this repository is provided under the Apache-2.0 terms, consistent with Section 5 of the license.

By opening a Pull Request, ensure that:

- you have the right to submit the contributed material;
- newly added third-party code/assets are identified with their original license and attribution requirements;
- provider proprietary code or other material that cannot be relicensed is not copied into this repository;
- required NOTICE/attribution information is preserved;
- generated code or assets are reviewed for licensing provenance before inclusion.

A separate CLA is not currently required. See [Licensing Policy](docs/licensing.md) for the current project policy.
