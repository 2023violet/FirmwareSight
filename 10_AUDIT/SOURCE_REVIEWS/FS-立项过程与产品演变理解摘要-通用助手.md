# FirmwareSight 立项过程与产品演变理解摘要

> 依据：用户提供的立项决策对话记录（粘贴文本1，止于 v0.2.0 交付）× 我已掌握的 v0.3.0 基线（已抽查 ADR/COMPLIANCE/RESEARCH/DELIVERY 原文交叉核对）。
> 我的理解范围：治理 / 工程 / 生命周期（v0.3.0 基线摘要见另一份文档）。

---

## ① 从硬件受限到产品选型的决策链与关键转折

**决策链全景：**

```
想做的方向不成立 → 重立选型标准 → 8 方向打分 → 定位 Firmware Release Workbench → 拒绝 CRA SaaS 陷阱 → MVP 五功能分层 → 先验证后写码 → v0.1.0 立项
```

**关键转折（按顺序）：**

1. **硬件前提崩塌（第一转折）**：用户想做 Embedded Test Workbench，但现实是"只有一台老式电源，连示波器都没有"。这直接推翻了上一轮把 HIL / 生产测试软件放第一的结论——致命问题不是技术，而是 **"我们自己都无法成为产品的高频真实用户，也无法充分验证它"**。HIL 方向付费信号明明是 ★★★★★（全场最高），仍被否决——判断依据是资源适配度而非市场热度。
2. **选型标准重立**：由此形成硬原则——**不做必须依赖硬件才能证明价值的软件，只做"输入就是文件"的软件**（.elf/.map/.hex/.bin + Git/CMake/Keil/IAR 工程）。网上公开项目（Zephyr、FreeRTOS Demo、STM32 HAL、ESP-IDF、MCUboot、littlefs、LVGL）即可当自动化测试素材，"一台电脑 + Rust" 就能开发验证 90% 以上核心逻辑，买开发板"不是产品成立的前提"。
3. **8 方向打分矩阵**：无硬件开发 / 痛点证据 / 付费信号 / 竞争 / 适配度五维打分。Firmware Release Workbench 第一；ProtocolSpec（协议单一真源）第二但败于 Adoption Problem（要求用户改变开发方式）；GUI Builder 第三（Electric UI 证明能收费但赛道更挤）；**ELF/MAP Analyzer "是功能，不是公司"**；串口助手 "太挤"；Embedded IDE "不做"；HIL "现在不做"；AI Embedded Agent "不适合当前资源"。
4. **需求本质转移**：痛点不是"测试"，是**发布工程**——"我准备发布这个固件了，它安全吗？"这里的"安全"不是网络安全，而是"**我到底知不知道自己发布了什么**"。产品主张 "Know exactly what ships." 由此而来。
5. **拒绝 CRA SaaS 陷阱**：CRA 法规窗口（2026-09-11 报告义务生效 / 2027-12-11 全面适用）是强 tailwind，但市场已有 CRA Kit（€99~€1,200/月）、Interlynk 等玩家，结论是"**不做 ONEKEY / Interlynk / CRA Kit 的低配复制品**"，改为"发布工具为主，CRA/SBOM 是高级能力"——产品第一页不是 Cybersecurity Compliance Platform，而是 "Drop your ELF. Understand exactly what you're shipping."
6. **MVP 分层（反一步到位）**：第一版只做 5 个功能（读 ELF/MAP、Firmware Diff、Build Identity、Release Gate、Release Bundle）；Dependency inventory → SBOM → CVE/VEX/CRA evidence 逐阶段后置。核心理由："**即使明天 CRA 不存在了，这个软件仍然有价值**"——法规只是顺风，不是生存前提。
7. **零工作流迁移优势**：对比 ProtocolSpec 的"以后协议先写我们的 YAML"（用户会抗拒迁移），本产品只要求"把你现有的 .elf/.map/Git repo 给我"，不改固件/RTOS/协议/IDE/硬件——这成为后续所有产品原则中最重要的一条商业护城河。
8. **先验证后写码**：明确"不是赶紧写代码"，而是先把 5~7 个核心界面 + 完整工作流做成原型，找 20~30 个真实嵌入式工程师/硬件公司/欧盟出口团队验证；出现主动问"什么时候能下载 / 支持 Keil IAR 吗 / 多少钱"，再写第一行产品代码。
9. **自用即验证**：团队自己的 MonoOLED、热牙胶设备、STM32/ESP32 项目就是第一批输入——"我们自己就是第一批用户"，极大降低做错方向的概率。
10. **移交立项**：用户要求完成命名 + 严格目录规范的交付包 → 产出 v0.1.0 立项基线（73 文件），命名经碰撞筛查定为 FirmwareSight。

---

## ② 市场调研核心证据与竞争判断（及其塑造的产品原则）

**核心证据链（记录为原始形态；v0.3.0 `08_RESEARCH/00_MARKET_EVIDENCE.md` 是其蒸馏版）：**

| # | 证据 | 类型 |
|---|---|---|
| 1 | 2026-07 开发者：固件对应多设备+多工具，Git Tag→CI→Firmware→QA 全做了，仍觉得"设备实际状态存在很高的歧义"，公开问大家用什么（Release Manifest? SBOM? Cryptographic fingerprint? GitLab artifacts? Internal DB?） | 真实痛点（纯软件问题，与仪器无关） |
| 2 | 另一开发者想把 Git commit→CI→UnitTest→自动版本号→重建→ELF/BIN→GitHub Release→Release notes 全链自动化 | 真实痛点 |
| 3 | 2026-07 STM32 开发者为 CRA 做 SBOM：五年前老项目无现代依赖管理（STM32Cube/FreeRTOS/mbedTLS/littlefs/vendor/patched source），现有工具"不好使"，准备手写 SBOM + Python 分析 .o 判断真编进固件的组件；8 月有人问 MCU SBOM 有没有真能用的工具（CPE 对不上、PURL 不清楚、VEX 不会搞） | **"工程师正在自己写 Python 补工具缺口"** ——最有价值的信号 |
| 4 | 中国链路：瑞芯微 2026-08 客户 CRA FAQ（销往欧盟产品）；SGS 中国 7 月出口欧盟直播；锦天城律所 2026-09-14 面向中国制造商的 CRA 文章 | 法规压力→客户压力→企业压力→工程部门工作量的完整传导链，证明不是"只和欧洲开发者有关" |
| 5 | CRA Kit 定价 Free/Solo €99/Team €399/Fleet €1,200（Team 已支持上传 .bin/.hex/.elf 分析）；Interlynk 嵌入式 SBOM 支持 GCC/CMake/IAR，自述交流 100+ 工程师、50+ POC（**记录特意标注：厂商自述，不能当独立市场统计**） | 竞争拥挤证据 |
| 6 | ELF 可视化已饱和：VS Code .map visualizer 获 88 赞；elfvis / elfyzer / elf-size-analyze / GCC Map View / Map View / st-mem | "ELF Analyzer 是功能，不是公司" |
| 7 | Electric UI 证明"帮硬件工程师少写一次桌面 GUI"有人付钱（Personal $80/year），但赛道有 Electric UI / Wavecake / Serial Studio / IO Ninja / EEZ Studio / Qt 开源工具 | 收费可行但更挤 |
| 8 | IO Ninja 把串口能力拆分卖钱（$45/$35/$20/$65/全功能 $495）+ B 站"挑战全网最强串口助手"10.2 万播放 4619 收藏且**永久免费**、另一开源串口工具 3.9 万播放 | **"用户多 ≠ 最好的创业机会"**——本轮调研最重要的反直觉结论 |

**这些结论直接塑造的产品原则：**

1. **Zero workflow migration**（零迁移）→ 对应 v0.3.0 的产品定位与"输入即文件"的 artifact adapter 设计。
2. **Trust 是最大风险**："如果软件说你的 SBOM 正确，却漏了一个组件，那很严重" → 绝不宣传"一键 CRA 合规" → 固化为 **No Compliance Verdict** 原则（v0.3.0 `07_COMPLIANCE` 明确禁用 "Certified"、"compliant/non-compliant verdicts"、"guaranteed security"）。
3. **确定性分级表达**：Detected / Likely / Unknown / Manual confirmation required，"unchecked 不能冒充 clean" → 固化为 **Evidence-first + Explicit Unknown**，即 Observed/Derived/Declared/Unknown 四态、"Unknown 不允许被隐藏"。
4. **Evidence confidence 可视化**（记录中的原型界面：FreeRTOS 100% 来自路径检测，mbedTLS 82% 推断自 version.h）→ 四态证据模型 + Gate 结果必须带 Result+Why+Evidence+Next 的 UI 雏形。
5. **Gate 价值 > 漂亮图表**："8/10 checks passed" 对工程团队的价值远大于"看一个漂亮的 Flash 饼图" → 价值公式 Understand → Compare → Verify → Package（而非 Upload → AI → Magic）。
6. **PLG 分层**：免费 ELF/MAP Analyzer 获客 → 真实发布场景（Gate/history/provenance/SBOM/CI/Team/Compliance）收费；且**买家不一定是开发者本人**（Lead/EM/QA/Quality Manager/Compliance/CTO/老板），因为"这个 release 能不能发"是公司问题 → v0.3.0 G4 Private Beta 验证 Free/Pro/Team 边界；`05_BUSINESS_MODEL` 明确"本文件是验证假设，不是最终定价，定价暂不冻结"。
7. **研究资料的证据等级意识**已写入治理：v0.3.0 明确"社区证据用于发现痛点，不等同于市场规模统计"，且文档权威顺序中研究资料最低、不能覆盖冻结决策。

---

## ③ v0.1.0 → v0.2.0 → v0.3.0 演变脉络与各阶段冻结项

三版均完成于 2026-09-26（见 CHANGELOG），**零实现代码**贯穿始终。

### v0.1.0 — 产品立项基线（73 文件，ZIP SHA-256 171b8fc4…）
**冻结了"做什么、为谁做、叫什么、凭什么原则"**
- 从市场调研对话直接固化为正式立项包（00_GOVERNANCE ~ 09_ADR + .ai/ + AGENTS.md + BASELINE.yaml + design tokens/schema/templates）。
- 命名：FirmwareLens / Firmware Release Studio → 碰撞筛查后定为 **FirmwareSight**（Embedded Firmware Release Workbench），Tagline "Know exactly what ships."，中文主张"每一次固件发布，都知道自己到底发布了什么"。
- 冻结五大核心原则：**Local-first、Evidence-first、Deterministic Core、Explicit Unknown、No Compliance Verdict**。
- 首批 6 项 ADR（0001 命名 / 0002 桌面栈 / 0003 Rust Core / 0004 Local-first / **0005 AI 不进可信路径** / 0006 MVP 格式范围）。
- 治理姿态：**PRE-IMPLEMENTATION BASELINE，.ai/ACTIVE_TASK = NONE**——明确"创建基线 ≠ 授权实现"，防止后续 AI 看到 Roadmap 就擅自把整个项目搭出来。

### v0.2.0 — 技术基线冻结（111 文件 / 96 Markdown，ZIP SHA-256 bfa4036a…）
**冻结了"用什么技术、坚决不用什么技术、为什么"**
- 输入：用户提供的三份 Rust GitHub 调研（原样存档于 `08_RESEARCH/SOURCE_REPORTS/`）。**关键克制：没有把调研里的 Serde+Tokio+Tracing+Reqwest+Rustls+SQLite+Tauri+wgpu 全部塞进项目**，而是服从普查的最高层结论 **Core-first, Adapter-driven, Product-specific Shell**。
- 冻结栈：Rust 1.98.1 / Edition 2024 / resolver 3；Tauri 2 + React 19.3 + TS strict + Vite 8 + Node 24 LTS + pnpm 12；object + gimli；Serde；tracing；clap（CLI `fwsight` 是一等公民界面）。
- 三个标志性收敛：① **rusqlite bundled 定案**，终结 SQLx 悬念（Local-first 单机 → rusqlite；JSON 退出数据库角色，仅限 config/snapshot/IPC/export/golden fixture）；② **Tokio 只在 Application 边缘**，core 同步确定性、禁 Tauri/SQLite/Tokio/React 类型；③ **Reqwest/Rustls/wgpu "Approved when needed ≠ Installed today"**——MVP 零网络零 GPU，未来路线预批准，避免重复争论（ADR-0013/0014）。
- 发布生命周期整体设计进去（NSIS/MSI、macOS 签名+公证、AppImage/deb、SHA256SUMS、updater 签名键、回滚），但自动更新**暂不启用**（私钥是安全资产，ADR-0015）。
- 新增 04_TECH/09–22 共 14 篇 + ADR-0007~0017；策略："**文档冻结 architecture family，lockfile 冻结 exact dependency graph**"。
- 方法论自觉：GitHub Star ≠ 商业收入；大样本只作工程先验，决策 = 先验 + 产品性质 + 官方状态 + 自身能力 + 维护成本。

### v0.3.0 — 生命周期与治理基线（无实现代码）
**冻结了"按什么节奏走、每一步怎么算过关、哪些坚决不做"**
- 新增两份权威文档：`06_DELIVERY/05_MVP_TO_PRODUCT_DEVELOPMENT_LIFECYCLE.md` + `06_DELIVERY/06_STAGE_GATES.md`。
- 完整生命周期：Problem Validation → **Technical Validation** → Product MVP → External MVP Validation → Productization → Private Beta → RC → GA 1.0 → Growth，配套 **G0–G6 门禁**（G0 已 PASS）。
- 把 v0.2.0 时代未拆分的"MVP"拆成两道门：**G1 Technical Validation**（最小链：真实 ELF fixture → Parse → Normalize → BuildSnapshot → `fwsight` CLI JSON → 极简 Desktop Summary，UI 仅"极简技术 UI"）与 **G2 Product MVP**（五页面 Start/Analyze/Compare/Gate/Bundle + Minimum Credible Product UI + 外部 8–15 名真实用户量化验证：Activation>80%、TTFV<60s、≥30% 发现真实新信息、付费信号 ≥3 明确意愿或 ≥1 团队 Pilot）。
- SBOM/CVE/CI 集成/Team Policy 明确排到 **Post-GA**；MVP Deferred 清单（account/cloud/team/AI/SBOM/CVE/updater/dark mode/plugin/HIL/flashing/OTA）。
- 当前全局状态：PRE_IMPLEMENTATION、G0 PASS、Active Task = NONE；下一步唯一合法动作是**获授权后创建 P0 Technical Vertical Slice**。

**一句话总结三版**：v0.1.0 冻结方向与原则（为什么做），v0.2.0 冻结技术路线（怎么做），v0.3.0 冻结节奏与门禁（做到哪算数、什么坚决不做）。

---

## ④ 对话记录与 v0.3.0 基线之间的差异、矛盾或疑点

1. **记录止于 v0.2.0**：v0.2.0→v0.3.0 的演变（治理/工程/生命周期深化）在记录中完全没有过程交代，只能靠 CHANGELOG 反推。阅读时不要把这份记录当成完整立项史。
2. **验证策略前后不一致（最实质的演变）**：记录主张"**原型先行**"——先做 5~7 个核心界面+完整工作流原型，找 **20~30 名**真实工程师验证，出现购买信号"再正式写第一行产品代码"；v0.3.0 实际路径是 **G1 Technical Vertical Slice 先行**（先写最小可运行链），外部真人验证后移到 G2、规模收敛为 **8–15 人**。顺序相反、人数不同，基线未解释为何放弃原型验证路线。按文档权威顺序，应以 v0.3.0 为准。
3. **Dependency/SBOM/CVE 排期口径差**：记录说"第二阶段 Dependency inventory、第三阶段 SBOM、第四阶段 CVE/VEX/CRA"；v0.3.0 把 Component Evidence/CycloneDX-SPDX/CVE/CI 集成/Team Policy 全部推迟到 **Post-GA**。记录中"第二阶段"语义有歧义（是产品第二阶段还是 v0.2.0 基线？），以 v0.3.0 的 Post-GA 口径为准。
4. **CLI 命名遗留**：记录第 20 节示例用 `firmwarelens analyze/diff/release`，后文又用 `fwsight`——记录内部本身不一致。v0.3.0 已由 ADR-0001 + BASELINE.yaml + `04_TECH/07_CLI_SPEC.md` 明确冻结 Binary 名为 **`fwsight`**；FirmwareLens 在 `08_RESEARCH/01_NAMING_SCREEN.md` 中因同名既有项目被排除。别把 firmwarelens 当有效命名。
5. **证据数字的等级问题**：记录里的具体数字（CRA Kit 定价、IO Ninja 定价、B 站播放量、Interlynk "100+ 工程师/50+ POC"）在 v0.3.0 市场证据文档中已被蒸馏掉，只保留 CRA 两个日期与证据模式。引用这些数字时必须注明出处等级——记录自己都标注了 Interlynk 数据是"厂商自述"。这些均为转述，本次未独立复核原始来源。
6. **交付包元数据未验证**：记录称 v0.1.0 包 73 文件、v0.2.0 包 111 文件（96 Markdown）及两个 ZIP SHA-256，属记录自述，无法对现存材料独立校验。
7. **Tagline 变体**：记录中出现三个变体（"Drop your ELF. Understand exactly what you're shipping." → "Know exactly what you're shipping." → 最终 "**Know exactly what ships.**"），中文主张全程未变。以 02_BRAND 冻结版为准。
8. **交叉确认（非矛盾）**：我此前在 v0.3.0 发现的 6 处文档卫生疑点（macOS CI 事件口径 FS-ENG-003 vs FS-ENG-007、06_DELIVERY 缺 04 号、FS-DEL-005 内部仍引用 v0.2.0、两套阶段命名体系等），与本次记录交叉后确认均为 v0.3.0 自身的文档同步问题，**不是记录与基线的实质矛盾**——例如 FS-DEL-005 引用 v0.2.0 只是版本引用未更新，实质内容与 v0.3.0 冻结结论一致。
9. **无方向性冲突**：记录中的所有定位、原则、技术收敛（五原则、Core-first、rusqlite、网络/GPU deferred、发布生命周期）在 v0.3.0 中均被完整继承并强化，未发现任何被推翻的决策——演变是"加深"而非"改向"。

---

## ⑤ 对话记录中隐含的产品 UI 气质约束

1. **反魔法、反 AI 化**：价值公式是 Understand → Compare → Verify → Package，而不是 Upload → AI → Magic；AI 不得进入可信路径（后成 ADR-0005）。工具感优先于智能感，"我们自己就是第一批用户"意味着界面首先是工程师的日常工具，不是演示品。
2. **反营销化、合规谦逊**：第一页不是 "Cybersecurity Compliance Platform"，而是 "Drop your ELF. Understand exactly what you're shipping."；**绝不宣传"一键 CRA 合规"**；"unchecked 不能冒充 clean"。后固化为 07_COMPLIANCE 禁词清单（Certified / compliant verdicts / guaranteed security）与 03_DESIGN 文案语言规范。
3. **证据优先、不确定性可视化**：Evidence confidence 是一等 UI 对象（FreeRTOS ██████████ 100% 来自路径检测；mbedTLS ████████░░ 82% 推断自 version.h）；必须区分 Detected / Likely / Unknown / Manual confirmation required。后固化为四态证据模型 + Gate 每条结果带 Result+Why+Evidence+Next。
4. **工程报表气质，而非仪表盘炫技**：核心叙事是具体数字——FLASH +8.4 KB、motor_control.o +3.2 KB、函数级 Δ 表、"Release readiness 8/10 checks passed"、✓/⚠/✕ 检查单；"漂亮的 Flash 饼图"被明确贬为价值不足。**但注意内在张力**：免费获客又需要"漂亮的 Treemap/Diff"吸引工程师——好看是获客手段，可信是留客理由。v0.3.0 用 G1"极简技术 UI"与 G2"Minimum Credible UI"的分层化解了这个张力。
5. **克制专业、量化不夸口**："8/10 checks passed" 是量化但克制的表达；主动暴露"VendorLib Version unknown → Manual verification required"；这种"承认不知道"的坦率是产品人格的一部分 → 后成 Explicit Unknown 原则与"禁用'智能地/自动地/完美'等不可验证形容词"的写作规范。
6. **零门槛进入**：拖入 .elf 即得 FLASH/RAM/百分比即时反馈，不注册、不改用户现有工作流、不动硬件 → Local-first + 无账号 + MVP Deferred（account/cloud）的 UI 源头。
7. **面向发布决策，而非文件查看**：界面语言围绕 "这个 release 能不能发"（readiness/gate/bundle），而不是 "这个文件长什么样"——五页面 Start/Analyze/Compare/Gate/Bundle 就是这个叙事的空间化；这也是它区别于 elfvis 等纯 viewer 的 UI 分界线。
8. **错误呈现是清单式、非弹窗恐吓**：⚠ mbedTLS changed 3.5.1→3.6.0、✕ Release notes missing 逐条列出 → 后来 05_ENGINEERING 错误模型四要素（What happened / Why we know / What user can do / Diagnostics ID）的雏形。

---

## 结论

这份对话记录回答了 v0.3.0 基线文档无法回答的"**为什么**"：为什么是发布工作台而不是测试平台（硬件现实）、为什么拒绝 CRA SaaS（竞争与信任风险）、为什么原则长这样（每条原则都能在调研证据中找到源头）、为什么 MVP 这么克制（先证明价值再谈合规）。v0.3.0 与记录之间**无方向性矛盾**，演变是单向收敛；需记住的两处分歧（原型先行 vs Vertical Slice 先行、SBOM 排期）均应以 v0.3.0 冻结口径为准。当前状态不变：PRE_IMPLEMENTATION，等待授权进入 P0 Technical Vertical Slice。
