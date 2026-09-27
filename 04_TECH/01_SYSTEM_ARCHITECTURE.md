---
title: "System Architecture v0.2"
doc_id: "FS-TECH-002"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-26"
---

# System Architecture

## 1. Architectural style

**Core-first + Hexagonal/Ports & Adapters + Product-specific Shell**

不是为了套理论，而是为了保证：
- Desktop 与 CLI 共用事实；
- parser 可以替换；
- storage 可以迁移；
- UI 不成为业务逻辑宿主；
- future CI/headless 不需要重写。

## 2. Layer model

```text
┌─────────────────────────────────────────────┐
│ Interface                                   │
│ React Desktop            fwsight CLI        │
└──────────────────┬───────────────┬──────────┘
                   │               │
                   ▼               ▼
┌─────────────────────────────────────────────┐
│ Application                                 │
│ use cases / jobs / cancellation / mapping   │
│ open project / import / compare / gate ...  │
└──────────────────┬──────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────┐
│ Pure Rust Core                              │
│ Domain / Normalize / Diff / Gate / Release  │
│ Evidence / Policy / Validation              │
└──────────────────┬──────────────────────────┘
                   │ ports
       ┌───────────┼────────────┬─────────────┐
       ▼           ▼            ▼             ▼
   Artifact     Storage     Provenance      Report
   Adapter      Adapter      Adapter         Adapter
       │           │            │             │
 object/gimli   rusqlite      git CLI       JSON/HTML
 MAP parsers     SQLite
```

## 3. Dependency rule

Allowed:
- Interface → Application
- Application → Core + Ports
- Adapter → Core contracts / ports

Forbidden:
- Core → Tauri
- Core → SQLite
- Core → Tokio
- Core → WebView/React
- Parser → UI
- UI → SQLite directly
- UI → filesystem directly

## 4. Initial workspace

```text
firmwaresight/
├── Cargo.toml
├── rust-toolchain.toml
├── package.json
├── pnpm-lock.yaml
├── pnpm-workspace.yaml
├── apps/
│   ├── cli/
│   └── desktop/
│       ├── ui/
│       └── src-tauri/
├── crates/
│   ├── firmwaresight-core/
│   ├── firmwaresight-artifact/
│   ├── firmwaresight-storage/
│   └── firmwaresight-report/
├── fixtures/
├── schemas/
├── docs/
└── .ai/
```

只有 4 个 library crates 起步。

### core
domain + normalize + diff + gate + release models.

### artifact
ELF/MAP/BIN/HEX adapters.

### storage
SQLite repository implementations.

### report
portable JSON/HTML rendering.

不要在 Phase 0 建十几个空 crate。

## 5. Application layer placement

MVP application orchestration initially lives in each binary/app with a small shared module if needed.

只有 desktop 与 CLI 出现明显重复 use-case code 后，才抽出 `firmwaresight-app` crate。

## 6. Data flow

```text
User-selected file
   ↓
fingerprint + format detect
   ↓
artifact adapter
   ↓
NormalizedArtifact
   ↓
Core builds BuildSnapshot
   ↓
transactional persistence
   ↓
query DTO
   ↓
Desktop / CLI
```

Diff 和 Gate 都只操作 normalized snapshot/policy，不重新解析 raw file。

## 7. Failure model

Adapters fail with typed diagnostic.
Core never fabricates missing facts.
Application maps technical errors to stable error codes.
UI renders error + evidence/capability state.
