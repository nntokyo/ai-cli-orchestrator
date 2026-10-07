# 敵対レビュー基準

すべての重要PRで、通常レビューとは別に「失敗させる前提」で確認する。

## Architecture

- Provider固有仕様がCoreへ漏れていないか
- Git必須になっていないか
- git worktree前提が混入していないか
- macOSだけで成立する実装になっていないか
- Windows process/path semanticsを壊していないか

## Session

- 別Taskのsessionを誤resumeしないか
- cwd変更後に古いsessionを無条件resumeしないか
- branch/HEAD変更を見逃さないか
- Provider切替で元sessionを破壊しないか
- stale contextをavailableとして扱わないか

## Concurrency

- 同一workspaceでwriterが2つ動けないか
- crashで永続lockしないか
- failover時に旧processが書き続けないか
- external editor変更を上書きしないか
- lease移譲後も旧Provider processが書き続けないか
- terminal/manual editsをAgent自身の変更と誤認しないか

## Failover

- test failureをProvider failureと誤認しないか
- write副作用後に同じpromptを無条件再実行しないか
- unknown errorで無限Provider切替しないか
- quota unknownをavailable扱いしていないか
- exit code 0だけでsoft-denied toolを成功扱いしていないか
- all targets unavailable時に無限retryせずwaiting/blockedへ遷移するか

## Permission

- Provider切替で権限が拡大しないか
- unsupported permissionを許可側へ丸めていないか
- network / external path / destructive actionを混同していないか
- auto approveが既定で有効になっていないか

## Recovery

- Gitなしworkspaceで戻せるか
- partial writeを検知できるか
- orphan processを回収できるか
- binary/large fileで容量爆発しないか

## Repository Host

- fork/upstreamを取り違えてremote mutationしないか
- GitHub認証/API障害でLocal Workspace Modeを止めていないか
- remote actionをretryしてIssue/PR/commentを重複作成しないか
- token/passwordをapp DBへ複製していないか

## Local Data / Privacy

- prompt/code/command output/raw event/checkpointの保存期間はboundedか
- delete workspace historyで関連データが残存しないか
- secret redaction前のraw logを無制限保存していないか
- SQLite/temp/checkpoint file permissionが広すぎないか

## Security

- shell injection
- argument injection
- path traversal
- symlink escape
- secret logging
- malicious workspace prompt
- credential copying
- remote action duplication
- unbounded child process
- insecure temporary files
- untrusted workspace trust bypass
- supply-chain / dependency substitution

## Public OSS

- 未実装を実装済みと書いていないか
- 非公開endpoint依存を公式仕様のように書いていないか
- ライセンス違反/CLI再配布条件に問題がないか
- trademark/brandingを公式製品と誤認させないか
- dependency/assetのlicense/NOTICE要件を落としていないか
- Provider binaryを再配布可能と無根拠に仮定していないか
- production artifactがsigning/SBOM/release gateを迂回していないか

## Merge gate

P0設計/実装PRは最低限:
- Issue link
- basic/detailed design該当箇所
- testsまたは「docs only」の明示
- 敵対レビュー記録
- blockerなし
を満たす。
