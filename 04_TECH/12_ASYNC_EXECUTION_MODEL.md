---
title: "Async and Execution Model"
doc_id: "FS-TECH-013"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-26"
---

# Async & Execution Model

## 1. Core principle

**Async is an execution concern, not a domain property.**

Core APIs should look like:

```rust
fn compare(base: &BuildSnapshot, target: &BuildSnapshot) -> Result<BuildDiff, DiffError>;
```

not:

```rust
async fn compare(...)
```

unless the operation itself fundamentally waits on an external async resource.

## 2. Work classes

### CPU / parsing
Examples:
- ELF parse；
- symbol normalization；
- diff；
- report rendering。

Execution:
- synchronous functions；
- Desktop invokes on blocking worker thread；
- CLI invokes directly。

### Blocking IO
Examples:
- file read；
- SQLite transaction；
- Git process。

Execution:
- never on UI thread；
- Desktop runs in background/blocking task；
- Core does not know executor.

### Async IO
MVP minimal.
Future:
- update metadata；
- CVE feed；
- license API。

Execution:
- Application/Network adapter may use Tokio.

## 3. Tokio policy

Desktop may depend on Tokio explicitly only if direct APIs are required.
Prefer Tauri runtime APIs where sufficient.

Tokio types must not cross:
- core public API；
- artifact contracts；
- storage ports；
- release schema。

CLI remains synchronous until a real async command arrives.

## 4. Cancellation

Long-running import job gets:
- operation id；
- progress state；
- cooperative cancellation flag at application level。

Parser loops should expose natural checkpoints where practical.

Cancellation never leaves partial Build visible as complete.

## 5. Concurrency

MVP:
- one active import per project；
- reads can occur while no write transaction is active；
- SQLite writes serialized through storage boundary；
- parallel parsing only after profiler evidence。

Do not introduce Rayon by default.

## 6. UI responsiveness

Target:
operations >100 ms must not block WebView event loop.
operations >500 ms should expose busy/progress state.
operations >2 s should expose cancellation where safe.
