---
title: "Project Charter"
doc_id: "FS-GOV-002"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Product"
last_updated: "2026-09-26"
---

# 项目章程

## 1. 项目名称

**FirmwareSight — Embedded Firmware Release Workbench**

## 2. 项目使命

让嵌入式团队在发布固件之前，对“自己即将发布的东西”形成可复核的工程事实，而不是依赖文件名、人工记忆和零散脚本。

## 3. 核心问题

典型小型嵌入式团队的发布信息散落在：

- Git tag / commit；
- `.elf/.map/.bin/.hex`；
- IDE 工程；
- Excel / Word；
- CI artifact；
- 第三方中间件文件夹；
- 研发人员经验。

结果是：
- 版本与构建产物可能不一致；
- memory 回归很晚才被发现；
- 发布包内容缺失；
- 很难回答“这个 binary 里到底进了什么”；
- 交接和审计依赖个人；
- SBOM/CRA 等要求到来时需要临时人工补资料。

FirmwareSight 把这些离散事实收敛成一个本地的 Release Evidence Model。

## 4. 成功定义

项目成功不以“功能数量”衡量。MVP 成功需证明：

- 用户把一个受支持的 ELF/MAP 拖入后，60 秒内得到可信分析；
- 用户能比较两个 build 并明确看到增长来源；
- Release Gate 能发现至少三类常见发布错误；
- 用户能导出一份可独立保存的 Release Bundle；
- 所有结论能追溯到 evidence；
- 即使完全离线，核心功能也成立。

## 5. 核心价值

**Understand → Compare → Verify → Package**

不是：
**Upload → AI → Magic answer**

## 6. 项目非目标

当前阶段不追求：
- 替代 Keil/IAR/VS Code；
- 烧录、调试、示波器、HIL；
- 自动生成固件；
- 自动漏洞利用分析；
- 云端设备管理；
- 法律合规认证；
- 万能支持所有 toolchain。

## 7. 项目原则

1. Evidence before verdict.
2. Deterministic before intelligent.
3. Local before cloud.
4. Existing workflow before migration.
5. Narrow compatibility before false compatibility.
6. Fast time-to-value before platform ambition.
