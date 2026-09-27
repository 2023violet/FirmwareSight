---
title: "ADR-0022 Portable Schema Strictness"
doc_id: "ADR-0022"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Architecture"
last_updated: "2026-09-27"
---

# ADR-0022 — Portable Schema Strictness

## Status
Accepted.

## Decision
Release Bundle portable schemas：
- stable objects default `additionalProperties:false`
- explicit `extensions`
- stable URN `$id`
- required schema_version
- semantic breaking change increments schema major

## Consequences
契约更严格，可审计；扩展必须有明确 extension point。

## Revisit trigger
公开 schema hosting/domain 确定后可增加 URL mirror，但 URN 不变。
