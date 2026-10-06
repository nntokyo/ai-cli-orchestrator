# ADR-0006: Transport Abstraction

Status: Accepted for MVP  
Date: 2026-10-07

## Decision

Provider AdapterとCLI transportを分離する。

優先順位:
1. native/app protocol
2. persistent structured stream
3. headless JSON process
4. PTY fallback

## Rationale

UI text scrapingは壊れやすいため、機械可読interfaceがある場合は優先する。

## Consequences

transport capability probeが必要。
persistent processではbackpressure、interrupt、crash recoveryが必要。
