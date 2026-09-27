---
title: "FirmwareSight MVP to Complete Product Development Lifecycle"
doc_id: "FS-DEL-005"
product: "FirmwareSight"
version: "0.5.0"
status: "BASELINE"
owner: "Product / Design / Engineering"
last_updated: "2026-09-26"
---

# FirmwareSight MVP → Complete Product Development Lifecycle

## 0. 文档目的

这份文档解决两个核心问题：

1. **FirmwareSight 的 MVP 到底要不要做 UI，以及 UI 应该做到什么程度？**
2. **如果 MVP 验证成功，如何从“可验证产品”一步一步演进成真正可发布、可收费、可长期维护的完整商业产品？**

本文件不是某个 Sprint 的任务清单，而是整个产品从“技术可行”到“商业可用”的**阶段路线图与决策框架**。

它与 `FirmwareSight_Project_Baseline_v0.5.0` 配套使用。

---

# 1. 先给结论：MVP 必须设计 UI，但不是做“最终 UI”

## 1.1 FirmwareSight 的 MVP 不是 CLI-only

FirmwareSight 的核心用户是：

- Firmware Engineer
- Firmware Lead
- QA / Quality
- Small Hardware Team / CTO

产品形态已经被确定为：

> **Local-first Embedded Firmware Release Workbench**

这意味着真实用户最终通过桌面界面完成：

```text
Analyze
   ↓
Compare
   ↓
Gate
   ↓
Release
```

因此如果 MVP 只有：

```text
fwsight analyze xxx.elf
```

那么只能验证：

> “ELF parser / Rust Core 是否能工作。”

无法验证：

- 用户是否理解结果；
- 用户是否看得懂 memory diff；
- 用户是否知道下一步做什么；
- Evidence 设计是否成立；
- Gate 是否真的能帮助 release；
- 工作流是否比现有脚本更省时间；
- 用户是否愿意持续打开这个产品；
- 用户是否愿意为这个体验付费。

所以：

> **FirmwareSight 的 MVP 必须包含 UI。**

但这里的 UI 是：

# Minimum Credible Product UI

而不是：

# Final Commercial UI

---

# 2. FirmwareSight 应该区分四种“验证”

整个项目不能把所有验证都叫 MVP。

正确顺序应当是：

```text
Problem Validation
        ↓
Technical Validation
        ↓
Product MVP Validation
        ↓
Commercial Validation
        ↓
Productization
        ↓
Beta
        ↓
Release Candidate
        ↓
v1.0 Commercial Product
```

---

# 3. Stage 0 — Problem Validation

## 目标

确认我们解决的是一个值得解决的问题。

问题不是：

> “大家需不需要 ELF Viewer？”

而是：

> “嵌入式团队在 firmware release 前是否缺乏统一、可追溯、可比较的发布事实？”

## 当前状态

FirmwareSight 已经基本完成这一阶段。

已有证据包括：

- 工程师自己维护 release manifest；
- 手工比较 firmware；
- 人工整理 Git/tag/build；
- 自己写 Python 工具；
- CRA/SBOM 增加 evidence / traceability 压力；
- 现有工具高度碎片化。

## Exit Criteria

进入下一阶段前至少应满足：

- 痛点是真实存在的；
- 用户群体明确；
- 当前替代方案明确；
- 产品不是单纯“我们觉得酷”。

FirmwareSight 当前满足，可以进入 Technical Validation。

---

# 4. Stage 1 — Technical Validation

## 目标

证明核心技术链可以可靠工作。

这不是完整 MVP。

核心 Vertical Slice：

```text
Real ELF Fixture
      ↓
object parser
      ↓
Normalized BuildSnapshot
      ↓
CLI JSON
      ↓
Minimal Desktop Summary
```

## 必须验证

### ELF
- architecture；
- sections；
- symbols；
- memory；
- hash。

### Normalization
不同 artifact 能转成统一 domain model。

### CLI
```bash
fwsight analyze firmware.elf --json
```

能输出稳定结构。

### Desktop
同一个 ELF：

Desktop 与 CLI 得到相同核心结果。

## UI 要求

这里 UI 可以非常简单。

只需要：

```text
Open Artifact
      ↓
Analysis Summary
```

例如：

```text
Firmware
motor.elf

Target
ARM Cortex-M4

FLASH
187.3 KiB

RAM
72.4 KiB

Sections
.text
.data
.bss

Top Symbols
...
```

## 不做

- 完整 Navigation；
- Dark Mode；
- Preferences；
- onboarding；
- 动画；
- 商业官网；
- updater；
- login；
- SBOM；
- pricing。

## Exit Criteria

只有满足：

- parser 不 crash；
- fixture 结果稳定；
- Core/CLI/Desktop 一致；
- Desktop 不阻塞；
- 数据模型没有明显结构错误；

才进入 Product MVP。

---

# 5. Stage 2 — Product MVP

这才是严格意义上的 FirmwareSight MVP。

## 5.1 MVP 的目的

不是：

> “证明我们能写软件。”

而是：

> **证明一个真实嵌入式工程师，可以用 FirmwareSight 完成一次真实 release 前检查，并认为它比原来的方法更有价值。**

---

# 6. MVP 必须做到什么 UI 程度

## 原则

MVP UI 必须：

- 完整；
- 清楚；
- 一致；
- 可独立操作；
- 没有明显布局错误；
- 没有临时 Demo 感。

但不需要：

- 最终品牌打磨；
- 大量动画；
- 20 个主题；
- 高级可视化；
- 无限设置；
- 插件系统。

---

# 7. MVP 必须有的 5 个页面

## 7.1 Start / Project

目标：

让用户知道：

> “我现在应该做什么？”

包含：

```text
FirmwareSight

[ Open Project ]
[ Analyze Artifact ]

Recent Projects
```

不需要账号。

---

## 7.2 Analyze

必须回答：

> “这个 Firmware 是什么？”

包含：

### Identity
- filename；
- SHA-256；
- architecture；
- Git commit；
- tag；
- dirty state；
- toolchain evidence。

### Memory
- FLASH；
- RAM；
- sections；
- budget。

### Symbols
- top symbols；
- filter；
- search；
- sort。

### Evidence
允许用户看到：

```text
Observed
Derived
Declared
Unknown
```

---

## 7.3 Compare

回答：

> “和上一版相比到底发生了什么？”

必须有：

```text
Build A
vs
Build B
```

显示：

- FLASH delta；
- RAM delta；
- section delta；
- added symbols；
- removed symbols；
- changed symbols；
- top growth contributors。

必须能 drill down。

---

## 7.4 Release Gate

回答：

> “这个 build 在我们的规则下是否准备好 release？”

显示：

```text
BLOCK
REVIEW
PASS
N/A
```

例如：

```text
PASS
Git tree clean

BLOCK
firmware.bin missing

REVIEW
FreeRTOS version unknown
```

每一条必须有：

- Result
- Why
- Evidence
- What to do next

---

## 7.5 Release Bundle

显示：

```text
Release v1.3.0

firmware.bin
firmware.elf
SHA256SUMS
release-manifest.json
analysis.json
release-report.html

[ Export Release Bundle ]
```

必须在导出前 Preview。

---

# 8. MVP 不应该做什么

以下全部推迟：

```text
Cloud
Account
Team workspace
License system
Plugin marketplace
AI assistant
CVE cloud service
Auto update
Dark theme
Custom dashboard builder
Complex graphs
Remote artifact repository
Firmware flashing
HIL
JTAG/SWD
OTA
```

MVP 的原则：

> **一个真实 Release Workflow 完整，胜过二十个半成品功能。**

---

# 9. MVP UI 的设计要求

## 9.1 不是“工程师界面随便做”

UI 是验证对象之一。

必须做到：

### Hierarchy
用户第一眼知道：
- 当前 Build；
- 当前状态；
- 当前主要操作。

### Consistency
统一：
- spacing；
- typography；
- table；
- status；
- button；
- error。

### Evidence Visibility
所有关键结论都能看到：

> Why?

### Unknown Visibility
Unknown 不允许被隐藏。

### Keyboard
最基本 keyboard navigation 可用。

### Performance
100k symbols 不允许整个 WebView 卡死。

---

# 10. MVP 的设计完成标准

我们不以“看起来像正式产品”作为标准。

使用以下标准：

## Visual
- 没有明显错位；
- 没有文字重叠；
- 层级清晰；
- 状态统一；
- 1024×720 可使用；
- Windows 125%/150% scaling 正常。

## UX
第一次用户在不看说明书的情况下能够：

```text
Import
→ Analyze
→ Compare
→ Gate
→ Export
```

## Product
用户能说出：

> “FirmwareSight 帮我发现了什么？”

而不是：

> “这个 UI 很漂亮。”

---

# 11. MVP 应该如何验证

不能只让我们自己测试。

## Validation Group

至少：

- 8–15 名真实嵌入式开发者；
- 最好来自不同 MCU / 公司；
- 必须使用他们自己的 artifact。

---

## Session

不要先教学。

给用户：

```text
这里有 FirmwareSight。

请拿你自己的一个 firmware build 试一下。
```

观察：

- 他首先点什么；
- 哪些地方不知道什么意思；
- 哪些数据最先看；
- 哪些页面完全没用；
- 哪里停顿；
- 哪里问问题；
- 哪里觉得比原工具更方便。

---

# 12. MVP 产品验证指标

## Activation

用户是否成功：

```text
Import → Analysis
```

目标：

> >80% 测试用户可以完成。

## Time to First Value

从打开软件到看到有价值结果：

目标：

> < 60 秒。

## Core Workflow Completion

至少：

> 60% 外部测试用户能够完成 Analyze → Compare。

## Gate Understanding

用户看到：

```text
BLOCK / REVIEW / PASS
```

能够正确理解含义。

目标：

> >80%。

## Real Discovery

至少：

> 30% 测试用户通过 FirmwareSight 发现一个以前没有立即意识到的信息。

例如：

- Memory regression；
- version mismatch；
- dirty Git；
- missing artifact。

## Return Intent

至少：

> 5/8 真实试用者表示下一次 release 愿意再次使用。

## Payment Signal

不是问：

> “你觉得多少钱合适？”

而是：

> “如果今天 Pro 版本是 ¥X/年，你会买吗？”

进入 Productization 前：

至少需要：

- 3 个明确付费意愿；
或
- 1 个团队愿意进行真实 Pilot。

---

# 13. MVP 验证失败怎么办

失败并不等于项目失败。

必须判断失败类型。

## A. 技术失败

例如：

> Keil artifact 根本无法可靠解析。

处理：

修 Parser / Adapter。

---

## B. UX 失败

例如：

> 用户看不懂 Compare。

处理：

重新设计信息架构。

---

## C. Workflow 失败

例如：

> 用户根本不会在 release 前打开这种工具。

这是严重问题。

需要重新定位使用节点。

---

## D. Value Failure

例如：

> “这些我 IDE 都能看到。”

这代表当前价值不足。

不能靠增加 UI 美化解决。

---

## E. Payment Failure

例如：

> 大家喜欢，但没人愿意付钱。

需要重新评估：

- Pro 功能；
- Buyer；
- Team 价值；
- 定价；
- 是否应该服务化。

---

# 14. Stage 3 — Productization

只有 MVP 验证成功，才进入这一阶段。

产品目标从：

> “能完成 Workflow”

转为：

> **“可以让陌生用户稳定、长期、低风险地使用。”**

---

# 15. Productization 第一重点：兼容性

MVP 可以只支持：

```text
GCC ELF
GNU ld MAP
```

Productization 要建立：

```text
Toolchain Compatibility Matrix
```

例如：

| Toolchain | Level |
|---|---|
| GNU Arm Embedded | Supported |
| Zephyr GCC | Supported |
| ESP-IDF | Supported |
| ArmClang | Beta |
| Keil AXF | Beta |
| IAR | Experimental |

只有：

fixture + regression + known limitations

才可以写：

> Supported

---

# 16. Productization 第二重点：可靠性

增加：

- malformed artifact fuzz；
- migration tests；
- crash recovery；
- interrupted import recovery；
- export integrity；
- database backup；
- diagnostics bundle；
- package smoke tests。

目标：

> 用户不能因为一个坏 ELF 把整个 App 弄崩。

---

# 17. Productization 第三重点：完整 Design System

到了这一阶段才真正系统完善：

### Typography
最终 token。

### Color
Light + Dark。

### Components
- Button；
- Input；
- Table；
- Menu；
- Dialog；
- Sheet；
- Tooltip；
- Status；
- Empty；
- Skeleton；
- Progress。

### Interaction
- hover；
- focus；
- pressed；
- disabled；
- loading；
- selected；
- error。

### Accessibility
完整 keyboard / focus / contrast。

---

# 18. Productization 第四重点：Onboarding

MVP 不需要复杂 onboarding。

正式产品需要：

### First Launch

```text
Welcome to FirmwareSight

1. Open your firmware project
2. Import an ELF
3. Compare builds
4. Prepare release
```

可以提供：

```text
Sample Project
```

让没有 ELF 的用户也能体验产品。

---

# 19. Productization 第五重点：Project Model

从：

> “打开一个 ELF”

演进成：

> “管理一个 Firmware Project”

例如：

```text
MotorController

Current
v2.4.1

Previous Releases
v2.4.0
v2.3.5
v2.3.0
```

具有：

- build history；
- budgets；
- policies；
- release history；
- component declarations。

---

# 20. Stage 4 — Private Beta

## 目标

从实验用户变成真实项目持续使用。

建议：

> 20–50 名用户

其中至少：

> 3–5 个真实团队。

---

## Private Beta 必须有

### Installer

Windows：
- signed installer。

macOS：
- signing/notarization。

Linux：
- AppImage/deb。

### Diagnostics
遇到问题可以：

```text
Copy Diagnostics
```

### Privacy
明确：

- 什么会联网；
- 什么不会联网；
- 数据放在哪里；
- 如何删除。

### Changelog
每个 Beta 更新都可追踪。

---

# 21. Beta 阶段开始验证商业模型

例如：

## Free

```text
Analyze
Basic Diff
Local Project
```

## Pro

```text
Release Gate
Release History
Advanced Diff
Release Bundle
CLI
SBOM
```

## Team

未来：

```text
Shared Policies
CI
Team Release Evidence
Support
```

但不要因为设计了三层套餐就马上全部开发。

---

# 22. Stage 5 — Release Candidate

RC 的定义不是：

> “功能差不多了。”

而是：

> **我们愿意让完全陌生的用户在生产 Release 中依赖它。**

---

# 23. RC Gate

必须满足：

## Function
- P0 workflow 无阻断 bug。

## Data
- migrations verified。

## Parser
- Supported formats fixture matrix green。

## Security
- dependency review；
- Tauri capability review；
- no arbitrary shell/fs exposure。

## Packaging
- installers；
- signing；
- upgrade test。

## Recovery
- corrupt DB recovery；
- export failure recovery。

## Documentation
- Getting Started；
- Supported Formats；
- Known Limitations；
- Privacy；
- Release Notes。

---

# 24. Stage 6 — FirmwareSight v1.0

这才是真正的：

# Complete Product

但“完整”不是“功能全部做完”。

完整代表：

> **一个明确范围内的产品闭环已经成熟。**

---

# 25. FirmwareSight v1.0 应该是什么样

## Core

```text
Analyze
Compare
Release Gate
Release Bundle
History
```

全部成熟。

## Compatibility

至少覆盖目标用户中最常见的：

```text
GCC / GNU
+
一个重要商业 Toolchain
```

而不是试图一次覆盖全部 MCU 世界。

## UI

完整 design system。

## CLI

```text
fwsight analyze
fwsight diff
fwsight gate
fwsight release
```

可以在 CI 使用。

## Release

- installers；
- signing；
- auto updater；
- rollback；
- changelog。

## Documentation

- Getting Started；
- Concepts；
- CLI Reference；
- Project Config；
- Toolchain Support；
- Troubleshooting。

## Commercial

- license；
- pricing；
- billing/distribution model；
- support channel。

---

# 26. v1.0 后才逐步扩展的能力

## Component Evidence

```text
FreeRTOS
mbedTLS
STM32Cube
littlefs
...
```

---

## SBOM

```text
CycloneDX
SPDX
```

---

## Vulnerability Intelligence

只有联网后：

```text
SBOM
↓
advisory feed
↓
CVE
↓
VEX
```

---

## CI Integration

```bash
fwsight gate --project . --json
```

GitHub Actions / GitLab / Jenkins。

---

## Team Policy

统一：

```text
Memory Budget
Required Artifacts
Version Policy
Release Rules
```

---

# 27. 产品不要直接跳到这些东西

以下功能虽然“听起来像完整产品”，但很容易让项目失控：

```text
AI Chat
Cloud Dashboard
Plugin System
Firmware Flashing
Device Management
OTA
Full Vulnerability Scanner
HIL
Source Code IDE
Build System
```

它们不是 FirmwareSight v1.0 的完整性要求。

---

# 28. 一张图理解整个制作流程

```text
┌───────────────────────────┐
│ 0. Problem Validation     │
│ 我们解决的是不是问题？      │
└──────────────┬────────────┘
               ▼
┌───────────────────────────┐
│ 1. Technical Validation   │
│ Core / Parser 能不能成立？  │
└──────────────┬────────────┘
               ▼
┌───────────────────────────┐
│ 2. Product MVP            │
│ 用户能不能完成完整 Workflow？│
│ Minimal Credible UI       │
└──────────────┬────────────┘
               ▼
          Real Users
               │
        ┌──────┴──────┐
        │             │
      FAIL          PASS
        │             │
        ▼             ▼
 Learn / Pivot   Commercial Signal
                      │
                      ▼
┌───────────────────────────┐
│ 3. Productization         │
│ 可靠性 / 兼容 / UX / Docs  │
└──────────────┬────────────┘
               ▼
┌───────────────────────────┐
│ 4. Private Beta           │
│ 真实团队长期使用             │
└──────────────┬────────────┘
               ▼
┌───────────────────────────┐
│ 5. Release Candidate      │
│ Production readiness      │
└──────────────┬────────────┘
               ▼
┌───────────────────────────┐
│ 6. FirmwareSight v1.0     │
│ 可发布 / 可收费 / 可维护      │
└──────────────┬────────────┘
               ▼
┌───────────────────────────┐
│ 7. Growth                 │
│ SBOM / CI / Team / Intel  │
└───────────────────────────┘
```

---

# 29. 每一个阶段设计投入应该是多少

| 阶段 | UI/UX 深度 | 工程深度 | 用户研究 |
|---|---|---|---|
| Problem Validation | Wireframe | 极低 | 极高 |
| Technical Validation | 最小 UI | 核心极高 | 低 |
| Product MVP | **完整最小 UX** | 高 | **极高** |
| Productization | **系统化设计** | **极高** | 高 |
| Private Beta | 接近正式 | 极高 | 极高 |
| RC | 正式 | Production | 回归验证 |
| v1.0 | Production Design | Production | 持续 |

---

# 30. FirmwareSight 当前所处的位置

截至 `FirmwareSight_Project_Baseline_v0.5.0`：

```text
Problem Validation      ✅ 基本完成
Product Definition      ✅ 完成
Brand Baseline          ✅ 完成
Design Baseline         ✅ 完成
Technical Baseline      ✅ 完成

Technical Validation    ⬅ 下一阶段
Product MVP             ⏳
Productization          ⏳
Private Beta            ⏳
RC                      ⏳
v1.0                    ⏳
```

因此：

> **现在还不应该直接做“最终产品”。**

正确下一步：

# Phase 0 — Technical Vertical Slice

然后：

# Phase 1 — Product MVP

---

# 31. 推荐的实际开发阶段编号

为了以后文档和 AI Agent 不混乱，建议正式采用：

## P0 — Foundation / Vertical Slice
核心技术成立。

## P1 — Analyzer
完整 Analyze。

## P2 — Compare
完整 Build Diff。

## P3 — Release Gate
Release policy 闭环。

## P4 — Release Bundle
MVP workflow 闭环。

到这里：

# MVP Candidate

---

## V1 — External MVP Validation
真实用户试用。

---

## P5 — Productization
兼容、可靠性、Design System、Docs。

---

## B1 — Private Beta
真实项目持续运行。

---

## RC1 — Release Candidate
Production readiness。

---

## GA 1.0 — General Availability
正式商业产品。

---

# 32. 最重要的开发纪律

以后任何人提出：

> “我们顺便把 XXX 做了吧。”

都必须先问：

### 1.
它属于哪个阶段？

### 2.
当前阶段是否需要它？

### 3.
没有它是否无法验证本阶段假设？

如果：

> 不影响当前阶段验证，

默认：

# 不做。

---

# 33. 最终原则

FirmwareSight 不应该走：

```text
想法
 ↓
疯狂开发半年
 ↓
做完整 UI
 ↓
上线
 ↓
找用户
```

应该走：

```text
Problem
 ↓
Technical Proof
 ↓
Minimal Credible Product
 ↓
Real User
 ↓
Evidence
 ↓
Productization
 ↓
Beta
 ↓
Commercial Product
```

换句话说：

> **MVP 的任务不是做一个“很差的最终产品”。**

而是：

> **用最小的完整产品体验验证最关键的商业与产品假设。**

而 Productization 的任务才是：

> **把已经被证明有价值的东西，做成值得长期依赖和付费的专业软件。**


## v0.4.0 Canonical Stage Mapping

权威 stage identifiers：

```text
G0 Problem Baseline        — passed
V0 Workflow Prototype     — next, parallel with P0 after authorization
P0 Technical Vertical Slice — next, parallel with V0 after authorization
G1 = V0 PASS + P0 PASS
P1 Analyzer
P2 Compare
P3 Release Gate
P4 Release Bundle
G2 Product MVP Candidate
V1 Own-artifact External Validation
P5 Productization
B1 Private Beta
RC1 Release Candidate
GA1 General Availability
```

自然语言 `Problem Validation / Technical Validation / Product MVP / Productization / Beta / RC / GA`
只作描述；自动化、任务、交付和 Agent 必须使用上述 identifier。

V1 比例指标必须报告实际 numerator/denominator；Return intent 阈值统一为 `>=62.5% 且至少 5 人`，不再写死成只适用于 n=8 的 `5/8`。
