# ADR-0003: Native Resume First

Status: Accepted for MVP  
Date: 2026-10-07

## Decision

Session identityをWorkspace単位ではなくTask単位にする。

```text
Workspace -> Task -> Provider -> ProviderSession[]
```

同一Providerではnative resume/continueを優先。
Provider変更時のみPortable Context Envelopeを生成する。

## Validation

resume前に:
- native session existence
- cwd
- workspace fingerprint
- optional branch/HEAD
- capability
- permission

を再検証する。

## Rationale

不要な再要約による情報損失・token消費を避けつつ、古いcontextの誤利用を防ぐ。
