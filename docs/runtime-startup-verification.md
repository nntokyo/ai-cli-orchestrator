# Runtime Startup Verification

Issue: #37  
Parent: #30

## Purpose

The bootstrap CI already verifies that the Tauri application compiles on macOS and Windows.
Runtime startup smoke verification adds a second property: the built desktop executable must actually start and remain alive through a short startup window instead of panicking or exiting immediately.

This is a process-liveness smoke test, not a visual UI test.

## Developer launch

From the repository root:

```bash
pnpm install --frozen-lockfile
pnpm tauri dev
```

Expected bootstrap behavior:

- an `AI CLI Orchestrator` desktop window opens;
- the bootstrap React shell renders;
- the process remains running until the window/application is closed;
- no Provider CLI, API key, network connection, Git repository, or Tauri IPC capability is required.

## CI runtime smoke

After:

```bash
pnpm tauri build --no-bundle
```

the CI checks the exact executable produced by Tauri.

### macOS

```text
target/release/ai-cli-orchestrator
```

The process is started in the background. After a five-second grace period, CI requires the PID to still be alive. The process is then terminated by a cleanup trap.

### Windows

```text
target\release\ai-cli-orchestrator.exe
```

PowerShell starts the process with `Start-Process -PassThru`. After a five-second grace period, CI requires `HasExited` to remain false. A `finally` block stops the process.

## Failure evidence

If the application exits during startup, the smoke step:

1. fails the workflow;
2. reports the exit status/code when available;
3. prints captured stdout/stderr;
4. still performs process cleanup.

The step has its own timeout so a broken cleanup path cannot leave the workflow running indefinitely.

## What this does not prove

The runtime smoke does not prove:

- exact visual rendering;
- user interaction correctness;
- accessibility behavior;
- code signing/notarization;
- packaged installer behavior;
- Provider CLI behavior;
- runtime IPC command behavior.

Those belong to their owning Issues.

## Security boundary

The smoke test must not broaden application permissions. It does not add:

- Tauri IPC capability;
- shell/filesystem/network permission;
- Provider credentials;
- Provider binaries;
- auto-approval.

It only starts the already-built bootstrap executable and checks process liveness.
