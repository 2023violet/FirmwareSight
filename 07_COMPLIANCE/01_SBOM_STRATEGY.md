---
title: "SBOM Strategy"
doc_id: "FS-COMP-002"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Compliance"
last_updated: "2026-09-26"
---

# SBOM Strategy

## 1. Principle

SBOM 是 Release Evidence 的输出之一，不是 FirmwareSight 的唯一产品。

## 2. Initial format

P1/P2 优先：
**CycloneDX 1.7 JSON**

原因：
- 当前稳定规范；
- 适合软件组件、依赖、安全信息；
- 生态成熟；
- machine-readable。

SPDX 3.0 作为后续 export format。

## 3. Do not overclaim detection

Component record 必须记录来源：

```text
name: mbedTLS
version: 3.6.0
state: detected
evidence:
  - version.h macro
  - source path
confidence: high
```

如果只发现路径：
```text
name: mbedTLS
version: unknown
state: detected
```

## 4. Evidence sources

优先级示例：
1. linked object/source + authoritative version header；
2. build-system metadata；
3. package manifest；
4. path fingerprint；
5. manual declaration。

Manual declaration 永远标记 `Declared`。

## 5. Coverage

产品未来可以给：
`SBOM evidence coverage`

但不能给虚假的“100% complete”，除非定义并验证 completeness model。

## 6. Standard evolution

CycloneDX 2.0 在 2026 年仍处于即将发布/演进阶段，因此项目不在 MVP 追新主版本。升级必须 ADR。
