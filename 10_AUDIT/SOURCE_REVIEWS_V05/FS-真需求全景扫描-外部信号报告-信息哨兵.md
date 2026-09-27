---
title: "真需求全景扫描·外部信号报告"
product: "FirmwareSight"
version: "1.0"
date: "2026-09-27"
authored_by: "信息哨兵（外部调研视角）"
nature: "第三轮 Post-MVP 探索《真需求功能全景扫描》外部调研交付件 · 非基线文档，不占用基线文档编号，不动基线副本"
basis: "《背景回顾-已覆盖面与本轮扫描边界清单-通用助手》（2026-09-27，下称《边界清单》）§1–§5 全部口径 + FirmwareSight_Project_Baseline_v0.3.0；外部证据经 2026-09-27 实时检索核验"
purpose: "在《边界清单》划定的合法产出形态内，交付规划外真需求的外部信号：每条含强度分级（强/中/弱）、来源链接、与四动词（分析→对比→门禁→发布）的相容性初判"
---

# 真需求全景扫描·外部信号报告

## §0 执行摘要

**三句话总评：** ①外部信号最密集的方向不是管理员提示清单里的任何一条，而是「栈安全证据」（puncover 两个最热 issue + Zephyr 官方 issue 同构指向"最坏栈/调用链/函数指针不确定性"），它与第二轮 P-10 结论（`.su` 挂 V1 访谈）强相关，按《边界清单》§1.3 以"佐证升格提请"口径上报；②合规证据链方向获得时效性最强的驱动——EU CRA 于 2026-09 起进入报告义务期、r/embedded 出现三个独立 SBOM 生成求助帖，直接佐证基线 SBOM P1 排期并激活 H3/H4/H9 三个悬置候选；③管理员提示的 changelog 生成（H1）外部信号偏弱——该需求已被平台内置（GitHub Release notes 自动生成）大量吞掉，嵌入式侧无独立强信号。

**信号一览表（12 条，编号 S-01…S-12，分级口径：强=多源独立且带场景原声；中=单一强源或二源间接；弱=单点/间接/无信号）：**

| # | 信号 | 分级 | 性质 | 相容动词 | 边界标注（《边界清单》§5） |
|---|---|---|---|---|---|
| S-01 | 栈安全证据：最坏栈+函数指针调用链 | **强** | 佐证升格提请 | 分析+门禁 | §5 排除项→§1.3 强冲突提请 |
| S-02 | CRA→SBOM（P1 排期佐证） | **强** | 排期佐证 | 分析+发布 | §5 已排期→佐证 |
| S-03 | license 风险提示（H9） | 中 | 候选验证 | 门禁+发布 | §5 本轮候选 |
| S-04 | SBOM evidence completeness / retention（H3/H4） | 中 | 候选验证 | 分析+发布 | §4 本轮候选 |
| S-05 | 组件版本漂移检测（H8） | 中 | 候选验证 | 对比+门禁 | §5 候选，须与 P2 区分 |
| S-06 | 发布物符号档案（version→symbol file 归档） | 中 | **新方向** | 发布 | §4 之外，X4 相邻 |
| S-07 | 组件 ABI 兼容性检查 | 中 | **新方向** | 对比+门禁 | §4 之外 |
| S-08 | 构建身份/工具链复现指纹 | 中 | 佐证+新方向 | 分析+发布 | 与 P-8 非 Git provenance 相邻 |
| S-09 | OTA 发布物元数据（极窄候选） | 中 | 极窄候选佐证 | 发布 | §5 高危擦边，默认不做 |
| S-10 | changelog/release notes 生成（H1） | 弱 | 候选验证 | 发布 | §5 候选（H1） |
| S-11 | 二进制/分区级 diff | 弱 | 新方向（近禁区） | 对比 | 紧邻 Non-goals #5 |
| S-12 | 解析适配需求池（overlay/debuglink/大文件） | 弱 | 佐证池 | 分析 | R2 佐证 |

---

## §1 扫描方法与信源说明

- **通道**：HN Algolia API（高质量讨论稳定通道）、GitHub Issues API 直接核取（Bloaty/puncover/Zephyr 三仓全量开放 issue 按热度排序 + 关键 issue 正文与评论原声抓取）、搜索引擎多轮定向（site:reddit.com 精确短语、引号短语、工具名+场景词），共 24 轮检索。
- **限制声明（去伪存真）**：本环境对 reddit.com 直连被阻断（Connection reset），Reddit 帖子证据均为**搜索引擎摘要级**，已在逐条信号中标注"摘要级/待证实"；GitHub/HN/官方文档证据为一手核验。综合环节引用 Reddit 条目时应保留"待证实"标记。
- **排除执行**：《边界清单》§1–§3 全部主题级/结论级/禁区口径已执行——凡撞见已排除主题的信号，一律降格为"佐证"（S-01/S-02）或"极窄候选佐证"（S-09），不作新发现上报。

---

## §2 强信号

### S-01 栈安全证据：最坏栈 + 函数指针调用链 【强｜佐证升格提请】

**证据（一手核验，三个独立来源同构）：**

1. **puncover#87 "Support Worst-Case Stack in symbol listings"**（开放中，7 评论，puncover 最热开放 issue）：
   https://github.com/HBehrens/puncover/issues/87
   用户原声："I frequently use Puncover to find the worst case stack usage of a function… It would be exceptionally useful if I could see this information as a column in the symbol list"（来自 Palmsens 公司固件工程师）。评论区已有人给出模板层 PR。诉求核心：**逐函数最坏栈进入主列表并可排序，而非点进单函数才可见**。
2. **puncover#105 "Chasing callbacks/function pointers?"**（开放中，5 评论）：
   https://github.com/HBehrens/puncover/issues/105
   用户原声：Zephyr 固件开发者自述用 puncover "determining safe stack sizes for different threads"，但函数指针/回调使静态调用图断链，"tripped us a number of times"。另一位用户（Prusa 打印机固件）跟帖附上自己的函数指针实例，并有人提交 PR#108。诉求核心：**调用图对间接调用显式标注"未知被调者"**——与 FirmwareSight "显式 Unknown" 哲学同构。
   另注意 #27 "Overview page / sort by stack usage"、#99 "nothing in stack column" 同向：https://github.com/HBehrens/puncover/issues/27
3. **Zephyr#27189 "syscall stack usage report"**（开放中，7 评论，官方仓库）：
   https://github.com/zephyrproject-rtos/zephyr/issues/27189
   诉求原文：为每个 `__syscall` API 生成栈上界报告，"Outliers that exceed the default privilege stack size should be flagged as issues"。**"报告 + 超限标红" 与 FirmwareSight 门禁四态逻辑完全同构。**

**为什么是"佐证升格提请"而非新发现**：栈用量分析在第二轮 P-10 已探讨并挂 V1 访谈验证（`.su` 文件）。但外部证据强度（三个独立仓库的高互动 issue + "线程定容"这一具体工程场景）显著超过普通访谈线索，按《边界清单》§1.3 "强冲突提请"条款，**提请综合环节考虑将 `.su` 从"访谈验证"升格为"P1/P2 候选排期"**——升格与否由综合环节裁决，本文不改结论。

**相容性初判**：分析（`.su`/调用图解析，Rust Core 已有 ELF 解析层，`.su` 为伴生文本产物，解析成本低）→ 门禁（"最坏栈 > 线程配额"是天然 deterministic policy，符合 §3.1-7 无 AI 红线）。**相容：高。**

**潜在影响**：若采纳，FirmwareSight 将成为"内存+栈"双维发布证据工具，与 puncover（只读 web app、作者已声明"currently not spending any time on this project"）形成代差——puncover 的维护真空期正是窗口。

---

### S-02 CRA→SBOM（P1 排期佐证）【强｜排期佐证】

**证据（三个独立 r/embedded 帖，摘要级/待证实；另有一手法规时间线）：**

1. "SBOM (Software Bill Of Material) generation for embedded"（2026 帖，为 Eclipse STM32 项目生成 CycloneDX 以满足 CRA）：
   https://www.reddit.com/r/embedded/comments/1ukhpg6/sbom_software_bill_of_material_generation_for
2. "SBOM generation for make/cmake projects / embedded"（明确提到 "requirement for CRA compliance if your auditor is a stickler for build-time manifests"）：
   https://www.reddit.com/r/embedded/comments/1s2cwa9/sbom_generation_for_makecmake_projects_embedded
3. "EU Cyber Resilience Act - How do you approach it"：
   https://www.reddit.com/r/embedded/comments/1h2m8iw/eu_cyber_resilience_act_how_do_you_approach_it
4. WizzDev（嵌入式服务商）招聘帖同向："You need an SBOM: You won't survive this without generating a proper SBOM for your builds"（摘要级）。

**要素拆解**：Who=中小固件团队/服务商；What=为 STM32/CMake 项目产 CycloneDX；Why=CRA 合规 + 审计师要 build-time manifest；When=CRA 2026-09 报告义务启动、2027-12 全面适用（法规时间线为一手事实）；How=目前手工/拼凑脚本。

**边界标注**：SBOM（CycloneDX 1.7）基线已排 P1（PRD §3）——本组信号**不是新发现，而是 P1 排期的外部强佐证**，同时为 S-03/S-04 的悬置候选提供上下文：社区痛点不在"要不要 SBOM"，而在"生成的 SBOM 可信吗、全吗、能存多久"。

**相容性初判**：分析（组件证据提取）→ 发布（SBOM 入 Bundle）。**相容：高（已排期方向）。**

**潜在影响**：CRA 时间窗使"本地 SBOM 生成"从 nice-to-have 变为 deadline 驱动采购——V1 访谈与 Beta 定价实验宜挂靠这一时间窗。

---

## §3 中信号

### S-03 license 风险提示（H9）【中｜候选验证】

**证据**：S-02 帖群中 auditor 语境 + CRA 附件要求（含 license 与安全更新信息）；工具侧现状：嵌入式 license 扫描依赖通用工具（ScanCode/FOSSA），无固件工作台原生集成（多轮检索未见嵌入式专属 license 提示工具，摘要级）。
**相容性初判**：门禁（"发现 GPL 组件进入闭源固件"类 deterministic 规则可行）→ 发布（license 清单入 SBOM/Bundle）。**相容：高，但须守 R4 边界：提示≠法律结论**（《边界清单》§5 已预判）。
**潜在影响**：license 字段是 SBOM 的自然扩列，若 SBOM P1 落地时预留 license 维度，H9 可低成本搭车而非独立立项。

### S-04 SBOM evidence completeness / retention（H3/H4）【中｜候选验证】

**证据**：S-02 第 2 帖 "auditor is a stickler for build-time manifests" 直指"SBOM 完备性/证据留存"诉求；CRA 2027-12 后市场监督期要求技术文档可追溯留存（法规背景一手）。基线 FS-COMP-002 §5/FS-COMP-003 有"未来可以支持"字眼、无排期锚（《边界清单》§4.3 H3/H4）。
**相容性初判**：分析（evidence coverage 模型——五级证据源已有设计基础）→ 发布（retention 归档形态，与 Bundle/History 相邻）。**相容：高。**
**潜在影响**：外部信号显示"合规"痛点在证据链可信度而非格式转换——H3 的"completeness model"比 H2（SPDX 导出）更值得综合环节优先讨论。

### S-05 组件版本漂移检测（H8）【中｜候选验证，须与 P2 严格区分】

**证据**：①ESP-IDF v5 component manager 官方引入 `dependencies.lock`（lockfile 模式已正式迁入嵌入式构建，一手：ESP 组件注册表与官方文档生态，辅助佐证 https://www.warpbuild.com 嵌入式 CI 文章讨论 lockfile 缓存）；②Renovate 宣称覆盖 C/C++（CMake/Meson/Make）依赖更新（工具侧一手：https://stagefreight.prplanit.com 特性页），但社区认知其不贴合本地优先固件工作流；③r/embedded 有 firmware/hardware 版本联动管理讨论（菲律宾语帖，摘要级/待证实：https://www.reddit.com/r/embedded 相关检索）。
**边界标注**：P2 已排**构建级 diff**（symbol/section delta）；H8 的差异点是**组件级跨 build 版本对比**（"mbedTLS 从 3.4.0 变 3.6.1 且没人记录"）。外部证据表明嵌入式侧 lockfile 已存在但**消费端缺位**——没有工具回答"两次发布间组件版本漂移了什么、是否被批准"。
**相容性初判**：对比（BuildSnapshot 间组件实体 diff，基线 Component Evidence 已有实体模型）→ 门禁（"未声明版本升级即 REVIEW"是 deterministic 规则）。**相容：高。**
**潜在影响**：这是"SBOM→发布"链条上离现有 P1/P2 排期最近的一步扩展（数据已在，缺对比视图），成本收益比在候选清单中最优。

### S-06 发布物符号档案（version→symbol file 归档）【中｜新方向】

**证据（一手，二源一致的行业惯例）**：
1. nRF Cloud 官方文档明确要求按固件版本上传 Symbol files 以解码设备数据："In order to properly decode your Device data, we first need to upload Symbol files to nRF Cloud for this firmware"（https://docs.nrfcloud.com Nordic Bluetooth Quickstart，2026-08 更新）；
2. Memfault SDK changelog 多次出现 "uploading symbol files" API 流程（https://components.espressif.com Memfault 组件页）；Memfault 模式="crash 事后解析必须依赖与固件版本精确对应的 ELF/符号"。
**需求形态**：两大固件云平台都把"版本↔符号文件"对应关系作为运维必需品，但**本地优先侧没有对应物**——团队事后想知道"v1.2.3 发布时的 ELF 在哪、hash 是多少"时只能翻 CI 产物或个人机器。
**相容性初判**：发布（Release Bundle 增补可选符号档案 + version→hash 映射；与基线 SHA256SUMS/release-manifest 天然衔接）。**相容：高，且不越界**（只是归档与映射，不涉云上传）。
**潜在影响**：这是本报告新方向中唯一"零新增解析器、纯 Bundle 扩列"的候选；若 V1 访谈确认 crash post-mortem 场景存在，可低成本并入 P4。

### S-07 组件 ABI 兼容性检查【中｜新方向】

**证据**：①cargo-semver-checks（Rust 生态"跨发布 API diff 门禁"）被 corrode.dev Rust 工具索引评价为 "The single most valuable check"（https://tools.corrode.dev，2026-09），并进入 Rust 官方项目目标（https://blog.rust-lang.org 2025-11）——**"API 破坏性变更检测作为发布门禁"是 Web/后端已验证的真需求模式**；②嵌入式 C 侧对应工具 libabigail（abidiff）存在但"工具存在但没接入工作流"（前轮 R1 结论的同构复现，摘要级）。
**需求形态**：嵌入式组件升级（vendor HAL、中间件）时，"结构体布局变了/函数签名变了导致 ABI 破坏"目前靠编译侥幸发现——链接期才暴露，或运行期更糟。
**相容性初判**：对比（两 BuildSnapshot 间符号签名/布局 diff，是 P2 symbol diff 的语义化扩展）→ 门禁（"公共头文件 ABI 破坏未声明"规则）。**相容：中高**——技术可行但需要 debug info 深度（DWARF 类型信息），超出 MVP 解析深度，宜标 P3+ 候选。
**潜在影响**：差异化机会点——cargo-semver-checks 证明该品类可成为"开发者离不开的门禁"，嵌入式侧尚无产品化先例。

### S-08 构建身份/工具链复现指纹【中｜佐证+新方向】

**证据**：r/embedded "Repeatable builds and IDEs"（容器化工具链=主流答案，摘要级：https://www.reddit.com/r/embedded/comments/1hjjxp2/repeatable_builds_and_ides）；"What is the best way to manage different (build) toolchains?"（摘要级：https://www.reddit.com/r/embedded/comments/ywas6y）。共识形态：Docker 锁工具链版本，但**构建产物的"身份页"（编译器版本、flag、源 revision、环境指纹）仍靠手工记录**。
**边界标注**：与第二轮 P-8 非 Git provenance（source tarball+SHA、构建元数据文件）直接相邻——本信号作为 P-8 保留产出的**外部佐证**，并提示一个具体扩展点：**工具链版本/编译 flag 指纹应进入 Identity/证据**（基线 Identity 已有构建元数据位，扩展成本低）。
**相容性初判**：分析（Identity 增列）→ 发布（指纹入 release-manifest）。**相容：高。**
**潜在影响**：CRA 报告义务同样要求"构建环境可追溯"，本方向与 S-02/S-04 共用同一合规叙事。

### S-09 OTA 发布物元数据（极窄候选）【中｜极窄候选佐证，默认不做】

**证据**：MCUboot image header 结构为行业标准（version/size/hash/signer/key id/signature/anti-rollback，一手：https://docs.alplab.ai 等多源文档一致）；openmv-ota trailer 设计含 SHA-256+ECDSA+targeting+anti-rollback+JSON 元数据 blob（一手：https://github.com/openmv/openmv-ota trailer.md）；FreeRTOS 论坛"OTA succeeds but AWS shows it as failed"显示 OTA 状态/版本元数据失配是真实故障类（https://forums.freertos.org，2023-01）；HN 侧 OtaFlux（OTA 包走 OCI registry，2025-05）显示 OTA 元数据标准化在演进（https://news.ycombinator.com/item?id=43995415）。
**边界标注（严格执行《边界清单》§3.8）**：OTA 本身两处禁区；本信号仅用于验证"OTA 所需元数据作为 Release Bundle 可选内容"这一极窄候选的**外部存在性**——MCUboot header 字段清单可直接作为"Bundle 可选 OTA 元数据节"的字段参考（version、hash、signer、anti-rollback 计数器）。**默认立场：不做，须过 Scope Creep 四问再议。**
**相容性初判**：发布（Bundle 扩列，纯元数据输出，不涉更新执行）。**相容：窄，但合规。**
**潜在影响**：若 V1 访谈中 OTA 团队频现，可作 P4 可选项；否则维持不做。

---

## §4 弱信号

### S-10 changelog/release notes 生成（H1）【弱｜候选验证】

**证据**：①GitHub 已内置 Release notes 自动生成（generate release notes from PR labels，2021+ 平台能力）——Web/后端侧该需求大量被平台吞掉；②嵌入式/固件专属的 changelog 痛点帖经多轮定向检索（git-cliff/conventional commits × embedded/firmware、site:reddit.com 精确短语）**未获独立强信号**（仅 Rust 工具索引间接提及 git-cliff，https://tools.corrode.dev）；③基线 H1 仅有两处字眼（PRD P0-6 "release-notes.md 如有"、Lifecycle §25 v1.0 Release 含 changelog）、无 US。
**研判**：外部市场对"通用 changelog 生成"需求已被免费平台能力满足；嵌入式差异化空间仅在"从构建元数据（组件漂移、门禁结论、符号档案）**汇编**发布说明草稿"——即把 H1 与 S-05/S-06 联动才有独特性，独立立项依据不足。
**相容性初判**：发布（report crate 出草稿）。**相容：高但价值弱。** 建议：维持基线"如有"口径，不升格。

### S-11 二进制/分区级 diff【弱｜新方向（近禁区）】

**证据**：bsdiff/xdelta 在 delta OTA 场景是标准件（Memfault Interrupt 博客一手：https://interrupt.memfault.com/blog/delta-firmware-updates）；binwalk 内置 fuzzy-hash 目录 diff（一手：https://github.com/ReFirmLabs/binwalk）；通用 binary diffing 工具盘点 7+（https://www.packetlabs.net）。工具生态丰富=用户可自行组合；面向"发布物完整性自证"的产品化空白存在但窄。
**边界标注**：紧邻 Non-goals #5（不做 binwalk 替代品/逆向套件）。若定位"自家两版发布的字节/分区级对照"（发布完整性自证）则合法；若滑向"未知固件分析"即触禁区。
**相容性初判**：对比（P2 的底层视角补全）。**相容：中，价值弱**——符号级 diff（P2 已排）覆盖 90% 发布场景，字节级差异对工程师的决策增量有限。建议不立项，综合环节知悉即可。

### S-12 解析适配需求池【弱｜R2 佐证池】

**证据**：Bloaty 开放 issue 全景显示需求集中在格式覆盖：#161 "Handle overlays (used in embedded)"（嵌入式 overlay 地址复用，增强请求）、#195 `.gnu_debuglink` 支持、Mach-O/PE/WASM 支持系列（一手：https://github.com/google/bloaty/issues 全量核取）；puncover #101 大 ELF 分析慢（请求并行化）、#42 Rust 支持。无一处指向"分析→对比→门禁→发布"链条外的新功能。
**研判**：相邻工具的 feature request 证明"解析深度/广度"是持续摩擦点，佐证基线 R2（工具链碎片化）与 fixture-driven expansion 机制，不构成规划外功能方向。
**相容性初判**：分析（parser 域内）。**相容：高，属既有排期域。**

### 双排除方向如实记录（无新信号）

- **团队 policy 共享**（已排期 Growth 排除）：定向检索未获独立新信号；唯一形态提示来自 S-08（指纹/配置文件化的合规叙事）。→ Growth 排期佐证不足，维持原状。
- **多构建趋势追踪**（已排期 P1 + 已覆盖 V7/B1 双排除）：检索所得均为既有佐证（Memfault "Tracking Firmware Code Size"，https://interrupt.memfault.com/blog/code-size-deltas），无增量。→ 维持双排除。

---

## §5 Web/后端已验证模式 × 嵌入式发布场景迁移矩阵（任务点③）

| Web/后端模式 | 已验证证据 | 嵌入式发布场景迁移判断 | 迁移结论 |
|---|---|---|---|
| Lockfile 漂移检测（npm/Cargo lockfile + Renovate/Dependabot） | npm/Cargo 生态标配 | ESP-IDF v5 已官方引入 dependencies.lock——**模式已自发迁入嵌入式构建侧**；缺的是消费端（跨发布对比/门禁） | **可迁移**（落点：S-05，Compare+Gate） |
| API 破坏性变更门禁（cargo-semver-checks，"single most valuable check"） | Rust 官方项目目标背书 | 嵌入式对应物 libabigail 存在但未进工作流；需要 DWARF 深解析 | **可迁移**（落点：S-07，Compare+Gate，P3+ 成本） |
| SBOM/CycloneDX（npm audit 生态 + CRA 法规） | 合规驱动成熟 | CRA 把同一合规义务直接压到固件团队头上，本地化生成是空位（云平台已占云端位） | **已迁移中**（落点：S-02 佐证 P1 + S-04 候选） |
| Release notes 自动生成（GitHub 内置/changesets/semantic-release） | 平台内置吞掉通用需求 | 嵌入式无平台红利，但独立生成价值弱；差异化仅在"从构建元数据汇编" | **弱迁移**（落点：S-10，不建议独立立项） |
| 依赖更新机器人（Renovate bot 模式） | Web/后端标配 | 嵌入式构建碎片化+本地优先，机器人形态不适配；且属 CI 域（前轮已探讨排除） | **不可迁移**（形态冲突+域排除） |
| Artifact attestation/SLSA provenance（npm provenance/GitHub attestations） | 供应链安全趋势 | 固件侧 MCUboot header/签名体系已有安全域对应物；FirmwareSight 的对位是"构建指纹+manifest 完整性"（S-08/S-09），非密码学 attestation | **部分迁移**（Build Identity 深化即可，不做密码学层） |
| Bundle size budget（size-limit/bundlesize）+ PR 门禁 | 已验证且前两轮已论证 | CI 位被 MemBrowse 占领；本地门禁位仍空但属前轮已探讨域 | **维持前轮结论**（不重复论证） |

---

## §6 风险与机遇（Agent 结论）

**机遇（按置信度排序）：**
1. **栈安全是当前外部需求密度最高、且与基线哲学最同构的方向**（S-01）——puncover 维护真空 + Zephyr 官方诉求 + "显式 Unknown"哲学契合，三条件同时成立的窗口罕见。建议综合环节优先裁决"佐证升格"。
2. **CRA 时间窗是免费的市场推力**（S-02/S-04）——2026-09 报告义务已启动、2027-12 全面适用，SBOM P1 的价值叙事、V1 访谈话题、Beta 定价都应挂靠此窗口。
3. **H8 组件漂移是性价比最高的"新功能"**（S-05）——数据模型已在 P1 Component Evidence 内，只差跨构建对比视图与一条门禁规则。
4. **S-06 符号档案是零解析成本的新方向**——若 V1 访谈出现 crash post-mortem 场景，P4 低成本并车。

**风险：**
1. **Reddit 证据均为摘要级**——S-02/S-08 引用社区帖前建议综合环节自行开帖核验原帖（本环境网络受限，已如实标注）。
2. **S-09 OTA 元数据是高频擦边区**——外部证据越强越要警惕 scope creep，默认不做立场不变。
3. **S-11 二进制 diff 与禁区 #5 的距离只有一步**——不建议立项，防止产品叙事滑向逆向工具。
4. **H1 changelog 外部信号弱与基线字眼候选的落差**——说明该方向是"内生理想"而非"外部拉力"，综合环节不应因基线有字眼而自动升格。

**给综合环节的一句话推演**：本轮外部信号的整体指向是——FirmwareSight 的规划外真需求不在"更多输出格式/更多端口"，而在**证据的完整性与可追溯性**（栈安全、组件漂移、符号档案、构建指纹、SBOM 完备性），它们全部可以塞回四动词链内，无需越过任何一条 ADR。

---

## §7 原始信源清单

**一手核验（本环境可直接访问）：**
| 信源 | 链接 | 用途 |
|---|---|---|
| puncover#87 | https://github.com/HBehrens/puncover/issues/87 | S-01 |
| puncover#105 | https://github.com/HBehrens/puncover/issues/105 | S-01 |
| puncover#27 | https://github.com/HBehrens/puncover/issues/27 | S-01 |
| Zephyr#27189 | https://github.com/zephyrproject-rtos/zephyr/issues/27189 | S-01 |
| Bloaty 开放 issues 全景 | https://github.com/google/bloaty/issues | S-12 |
| nRF Cloud Symbol files 文档 | https://docs.nrfcloud.com | S-06 |
| Memfault SDK changelog（symbol files） | https://components.espressif.com | S-06 |
| Memfault delta OTA 博客 | https://interrupt.memfault.com/blog/delta-firmware-updates | S-11 |
| Memfault code size 博客 | https://interrupt.memfault.com/blog/code-size-deltas | 双排除佐证 |
| openmv-ota trailer 设计 | https://github.com/openmv/openmv-ota（trailer.md） | S-09 |
| MCUboot header 结构（多源一致） | https://docs.alplab.ai 等 | S-09 |
| FreeRTOS 论坛 OTA 状态失配帖 | https://forums.freertos.org（2023-01） | S-09 |
| OtaFlux（HN Show） | https://news.ycombinator.com/item?id=43995415 | S-09 |
| corrode.dev Rust 工具索引 | https://tools.corrode.dev | S-07/S-10 |
| Rust 官方博客（cargo-semver-checks） | https://blog.rust-lang.org（2025-11） | S-07 |
| Renovate C/C++ 覆盖宣称 | https://stagefreight.prplanit.com | S-05 |
| WarpBuild 嵌入式 CI 文章 | https://www.warpbuild.com | S-05/S-08 |
| HN Algolia 检索（firmware size/OTA） | https://hn.algolia.com/api | S-01/S-09 背景 |
| MemBrowse（HN Show，2025-11） | https://news.ycombinator.com/item?id=45979698 | 竞争位确认 |

**摘要级/待证实（Reddit 直连受网络阻断，仅搜索引擎摘要）：**
| 信源 | 链接 | 用途 |
|---|---|---|
| r/embedded SBOM generation for embedded | https://www.reddit.com/r/embedded/comments/1ukhpg6 | S-02 |
| r/embedded SBOM make/cmake | https://www.reddit.com/r/embedded/comments/1s2cwa9 | S-02/S-04 |
| r/embedded EU CRA approach | https://www.reddit.com/r/embedded/comments/1h2m8iw | S-02 |
| r/embedded Repeatable builds and IDEs | https://www.reddit.com/r/embedded/comments/1hjjxp2 | S-08 |
| r/embedded manage toolchains | https://www.reddit.com/r/embedded/comments/ywas6y | S-08 |
| r/embedded firmware/hardware versioning（菲律宾语） | https://www.reddit.com/r/embedded（检索所得） | S-05 |
| WizzDev SBOM 表态（招聘/项目帖） | reddit 检索摘要 | S-02 |

---

*整理：信息哨兵 · 2026-09-27 · 供第三轮综合成文使用；本文不修改任何基线文档；所有强度分级与相容性初判为调研方判断，最终取舍由综合环节与管理员裁决。*
