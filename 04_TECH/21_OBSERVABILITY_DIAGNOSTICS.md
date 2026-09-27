---
title: "Observability and Diagnostics"
doc_id: "FS-TECH-022"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-26"
---

# Observability / Diagnostics

## Goals

A user should be able to report:
> Import failed with operation `abc123`

without sending firmware source.

## Tracing spans

Suggested spans:
- `project.open`
- `artifact.hash`
- `artifact.detect`
- `artifact.parse`
- `artifact.normalize`
- `storage.import`
- `diff.compute`
- `gate.run`
- `release.export`
- `ipc.command`

Fields:
- operation_id
- adapter_id
- artifact_kind
- byte_size
- elapsed_ms
- result_class

Avoid by default:
- full source paths
- symbol names in logs
- firmware bytes
- source code
- secret tokens

## Log levels

ERROR — operation cannot complete  
WARN — capability degraded/review needed  
INFO — high-level lifecycle  
DEBUG — adapter details  
TRACE — disabled by default

## Diagnostic bundle

Future user-triggered bundle may include:
- app version；
- OS；
- parser versions；
- sanitized logs；
- capability summary；
- config with sensitive paths redacted。

Never attach artifacts automatically.

## Performance telemetry

Local timing via tracing is allowed.
Remote telemetry is not enabled in MVP.
