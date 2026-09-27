---
title: "V0 Clickable Prototype"
doc_id: "FS-V0-004"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Product / Design"
last_updated: "2026-09-27"
---

# V0 Clickable Prototype

## Run

No dependency install is required.

Open `index.html` directly in a modern desktop browser, or serve this directory with a static local server:

```bash
python -m http.server 8765
```

Then open:
`http://127.0.0.1:8765/`

## Reset

Reload the page or press:
`Alt + Shift + R`

## Scope

This is not production frontend code.

It intentionally contains:
- static fixture data;
- a deterministic in-memory state machine;
- no Rust;
- no Tauri;
- no SQLite;
- no real filesystem parser;
- no real export.

## Required test story

Start in STATE A with no MAP.
Add MAP in Analyze to enter STATE B.
Use Dependencies/Gate/Compare.
Attempt invalid ELF to enter STATE C.
Recover and use Bundle/History as STATE D.
