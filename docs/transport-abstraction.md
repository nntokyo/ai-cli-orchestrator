# Transport Abstraction

Issue: #15  
Status: Implemented after merge  
Updated: 2026-10-07

## Goal

Provider-specific CLI behavior must not leak into Router/Core process handling.

Transport selection prefers stable machine-readable surfaces:

1. official native/app/agent protocol
2. persistent structured stdin/stdout
3. one-shot headless structured process
4. PTY fallback

PTY is a compatibility fallback, not the primary integration model.

## Current provider baseline

Checked against official documentation on 2026-10-07:

- Codex: App Server is an official application integration surface.
- Claude Code: Provider adapter must re-check current structured/headless capabilities at implementation time.
- Grok: ACP over stdio is documented for IDE/tool integrations; structured headless JSON is available.
- Antigravity: `stream-json` input/output supports multiple turns in one long-lived process; JSON/headless modes are also available.

Provider-specific command lines and protocol schemas belong to #5.

## Core transport kinds

- `NativeProtocol`
- `PersistentStructured`
- `HeadlessStructured`
- `PtyFallback`

The enum encodes preference only. Availability is determined by Provider capability probing.

## Command safety

`CommandSpec` stores:

- executable path/name
- argument vector
- explicit cwd
- explicit environment policy

Arguments are never concatenated into a shell command string.

### Environment policy

Adapters must choose one explicitly:

- `Inherit`: inherit parent environment, then set/remove selected values
- `Clear`: start from an empty environment and set only selected values

No implicit environment-policy default is provided.

This matters because Provider CLIs may rely on HOME/PATH/official credential stores, while native integrations may require tighter environment control.

## Stdio framing

The generic stdio transport is line-oriented and byte-preserving.

- stdout and stderr remain separate
- CRLF is normalized at the framing boundary
- partial reads are reassembled
- a final unterminated frame is emitted at EOF
- frames have a configurable maximum
- oversized frames are reported and skipped until the next newline

The transport does not parse Provider JSON. Provider Adapter owns schema parsing.

## Backpressure

The event channel is bounded.

Reader workers use bounded retry rather than unbounded buffering. If the consumer stops reading, the pipe eventually backpressures the child process instead of allowing memory growth.

During shutdown, reader workers can be stopped so teardown cannot deadlock forever on a full event channel.

## Process lifecycle

The stdio transport supports:

- persistent writes to child stdin
- stdout/stderr events
- process ID
- non-blocking exit polling
- closing stdin
- graceful shutdown with timeout
- forced termination after timeout
- best-effort termination on Drop

Platform-specific graceful interrupt signals and process-tree semantics belong to #9.

## Cancellation semantics

Issue #15 provides portable lifecycle primitives.

- normal completion: consume terminal Provider event, then shutdown
- graceful cancellation: close stdin and wait for a bounded period
- timeout: terminate child process

Provider-specific interrupt/control messages may be added by #5 when the official protocol supports them.

Windows/macOS process-group escalation belongs to #9.

## Security boundaries

- no shell concatenation
- cwd is explicit and validated as a directory
- environment inheritance is explicit
- stdout/stderr remain separate
- output memory is bounded by frame/channel limits
- Drop prevents obvious orphan child processes
- Provider credentials are not interpreted or persisted here
- transport events are raw bytes; no lossy Unicode conversion is required

## Explicit non-goals

Issue #15 does not implement:

- Codex App Server JSON-RPC schema
- Claude Code event schema
- Grok ACP schema
- Antigravity stream-json schema
- Provider auth
- Provider quota parsing
- PTY implementation
- Windows ConPTY/process-tree management
- OS-specific Ctrl-C/SIGINT delivery

Those belong to #5/#9.

## Adversarial invariants

- an argument containing spaces or shell metacharacters remains one argument
- invalid cwd fails before child execution
- output cannot grow an unbounded in-memory line buffer
- event queues are bounded
- stdout and stderr cannot be silently merged
- shutdown cannot wait forever before escalation
- transport teardown attempts to prevent orphan processes
- Provider-specific data cannot change Core transport semantics
