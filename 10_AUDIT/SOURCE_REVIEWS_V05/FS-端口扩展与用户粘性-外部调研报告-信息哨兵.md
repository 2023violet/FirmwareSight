---
title: "端口扩展与用户粘性·外部调研报告"
product: "FirmwareSight"
report_id: "FS-RSCH-012"
version: "1.0"
date: "2026-09-27"
authored_by: "信息哨兵（外部调研视角）"
nature: "群内工作文件 · 外部调研 · 非基线文档，不占用基线文档编号"
task_source: "群管理员 2026-09-27 指派【外部调研】：①同类工具端口/集成打法盘点 ②单一工具 vs 工作流枢纽实证 ③嵌入式工作流接入点普查+需求信号分级 ④小众深耕案例 ⑤集成类付费意愿与先例 ⑥机会与陷阱初判"
method: "z-ai web_search 多轮检索 + GitHub API 定向核查（仓库/README/元数据）+ HN Algolia API + 官网/Pricing 页直抓；全部信源访问日期 2026-09-27"
grading: "【强】= 官方文档/定价页/一手代码仓库/权威博客原文；【中】= 高分社区讨论、二手报道、搜索摘要可交叉；【弱/待证实】= 仅检索摘要级证据，未取到原文"
upstream: "与 FS-RSCH-011《分析结果可视化与文档导出·外部调研报告》同系列，结论互相引用"
---

# 端口扩展与用户粘性·外部调研报告

> **一句话总结（Agent 结论）**：外部证据支持"端口=粘性"的基本判断——2025–2026 年，嵌入式 size/内存域出现了一个正在商业化验证该命题的产品 **MemBrowse**（GitHub Action + 云端历史 + 按 target 订费），它证明"分析→CI→PR comment→门禁"这条端口链有真实付费需求；但它同时**占领了"云 portal"生态位**，留给 FirmwareSight 的空位不是"做一个云版跟随者"，而是**本地优先的数据纵深 + 仪器级证据输出**——端口打法要学，生态位要避开。
>
> **证据底座**：本报告 28 条信源，其中【强】10 条、【中】12 条、【弱/待证实】6 条。所有 Reddit 证据按惯例标注摘要级/原文级。

---

## §0 执行摘要 · 六条关键发现

1. **深度工具普遍"裸奔"，集成层由社区或商业产品补位**【Fact】：Bloaty、puncover、linkermapviz 均无官方 CI 集成；为它们补端口的是社区 Action（carlosperate/bloaty-action）或商业产品（MemBrowse）。端口是独立于解析核心的另一层能力，且是粘性的主要来源。
2. **嵌入式 size 域的"PR comment + 历史追踪 + 门禁"已被商业验证**【Fact/强】：MemBrowse（Show HN 2025-11）以 Free/$99/$249 每月三档运营，客户含 Flipper Devices、wolfSSL、Apache NuttX、RT-Thread、TinyUSB；其定价单位是 **tracked target（被追踪的构建目标数）**——一个全新的、贴合嵌入式心智的计费维度。
3. **"单一工具被吸收、工作流枢纽难替代"有官方实锤**【Fact】：Bower 官网至今挂着弃用劝退声明（"recommend using Yarn and Vite"）；反例是 PlatformIO、Memfault（2025 被 Nordic 收购）、SonarQube、Codecov（2022 入 Sentry、2026 入 Harness——集成位本身能被反复并购定价）。
4. **嵌入式团队的接入点需求信号真实且具体**【Fact】：Memfault Interrupt（ex-Pebble 团队）两篇长文详述"每次 PR 计算 code size delta + 自建 size dashboard"的完整做法；r/cpp 有高分抱怨："分析工具存在，但没接入开发者工作流"。
5. **收费先例集中在"流程位"，不在"导出物"**【Fact/与上轮结论一致】：CI/PR 集成类产品按 target/席位/LOC 收费（$99/月起～$12/人/月）；而"报告导出"在全部被查工具中仍无收费先例（延续 FS-RSCH-011 结论）。
6. **端口即攻击面**【Fact】：Codecov 2021 年 CI token 供应链泄露事件（波及 GoDaddy、Atlassian 等）是"集成类功能"的安全反例——对本地优先的 FirmwareSight，这既是设计警戒线，也是卖点。

---

## §1 同类工具靠端口/集成建立粘性的打法盘点（对应要求①）

### 1.1 固件 size 五工具的端口形态对照

| 工具 | 自身定位 | 自带端口 | 社区/商业补位的集成 | 粘性机制评估 |
|---|---|---|---|---|
| **Bloaty** (google/bloaty) | ELF/Mach-O/PE size profiler（CLI） | CSV/TSV 输出、stdin 支持（脚本友好）【Fact】 | `carlosperate/bloaty-action`（GHA，支持 Job Summary 与 PR comment 指南，9★，附 Docker 镜像）【Fact/强】；HN 发布帖 202 分（2017）【Fact】 | 上游零集成，靠深度+Google 背书"被动"长存；其 CI 存在感全靠社区补位 |
| **puncover** | 符号级 size/栈分析（本地 Web UI） | JSON 报告导出（`--generate-report`，FS-RSCH-011 已证）【Fact/强】 | 无官方 Action；Memfault/Zephyr 文档引用其用法【Fact/中】 | 端口最少的深度工具；用户自己写胶水进 CI |
| **cargo-bloat** | Rust 二进制 size（cargo 子命令） | 生态位本身就是集成：cargo 子命令 = Rust 工具链的分布式端口【Opinion】 | `orf/cargo-bloat-action`（115★）："runs on every pull request"，评论 size 分解与依赖树变化【Fact/强】 | 官方 Action + 每 PR 自动评论 = 本组内唯一"全套打法"的开源样本 |
| **linkermapviz** | GNU ld map 可视化（CLI→HTML） | 仅本地 HTML 输出 | 无（GitHub 检索无任何 Action/CI 生态）【Fact】 | 78★、2026-05 仍在维护，但无端口=无粘性网络效应，与 FS-RSCH-011 "重复造轮子"判断一致 |
| **STM32CubeIDE Build Analyzer / CubeMX** | IDE 内置分析 / 代码生成 | IDE 内置墙（不外流数据）【Fact，上轮已证】；CubeMX 代码生成是"反向端口"——用户的项目文件依赖它再生【Fact】 | Hackaday 评测：几乎所有 STM32 IDE 起项目都依赖 CubeMX【Fact/中】 | 大厂粘性 = codegen 锁定 + IDE 内置；社区对 HAL 质量长期抱怨（ST 官方论坛 BUG 帖）但锁定照常生效——**锁定不靠质量靠流程位** |

> **要点**：在这个细分域，"端口"几乎从不出自深度工具本身，而出自**第二层**（Action 封装者、SaaS 化者）。这对 FirmwareSight 是结构性的好消息：官方 CLI + 官方 Action 的组合在嵌入式 size 域尚无开源先例（MemBrowse 是商业闭源云路径）。

### 1.2 新变量：MemBrowse 全案拆解【本报告最重要的单一条目，全部一手信源】

- **产品形态**（GitHub README + docs.membrowse.com，2026-09-27 访问）：
  - `pip install membrowse` → 本地 CLI 分析 ELF（无账号、免费）；
  - GitHub Action（Marketplace verified creator）+ GitLab CI 官方模板 + Jenkins/Azure 文档指引；
  - 云 portal：历史趋势、commit 级 diff、多 target 对比、**预算门禁（fail CI when memory exceeds limits）**、PR comment（显示每次 PR 的内存增减）；
  - **Claude Code 插件**：`/plugin install membrowse@...` 让 AI agent 自动接线 CI——2026 年新出现的"agent 端口"形态【Fact/强】。
- **技术口径**：读 ELF+DWARF+链接脚本，支持 **GNU ld、IAR ICF、SEGGER Embedded Studio .emProject** 三种内存布局来源【Fact】——注意：这证明"IAR/Segger 侧数据摄入"不只是 FirmwareSight 的遐想，而是已被商业产品验证的需求方向。
- **隐私口径**："Your source code never leaves your infrastructure. Only JSON reports containing symbol names, sizes, addresses... are uploaded"【Fact】——本地分析+仅上传摘要 JSON，与基线"portable output = versioned JSON"的口径同构。
- **客户墙**：Flipper Devices、wolfSSL、Apache NuttX、RT-Thread、TinyUSB、SuperTinyKernel、CMRX【Fact/强，README 原文】。
- **时间线**：Show HN 2025-11-19（3 分，无声量）→ 2026-09 README 已挂客户墙+Marketplace 认证+定价三档【Fact】。一年内从 Show HN 到可运营商业化。
- **对 FirmwareSight 的意义**【Opinion】：它验证了"size/内存 → CI/PR/门禁"端口链的付费需求；同时把"云 portal（历史/趋势/对比）"生态位占住了。FirmwareSight 的 **本地优先 + 桌面工作台 + 自包含 HTML 证据** 恰好是它没做、也因商业模式（云订阅）**不会做**的形态——这不是威胁，是错位确认。

### 1.3 可类比的通用开发工具：端口形态学（lint / formatter / coverage 族的通用打法）

| 端口形态 | 代表 | 粘性原理 | 对 FirmwareSight 的可移植性 |
|---|---|---|---|
| CLI + 稳定 `--json` | ESLint `--format json`、bloaty CSV、cargo-bloat【Fact】 | 一切集成的地基；"CI 不允许解析自然语言"是行业公理（与基线 FS-TECH-008 完全同构） | 已预留（`fwsight --json`），零新成本 |
| GitHub Action | 官方/社区 Action（esp-idf-ci-action 108★、bloaty-action、cargo-bloat-action）【Fact】 | 进入"必经之路"（CI），从可选变必选 | 薄壳包 CLI，Growth 已锚（FS-DEL-001） |
| Job Summary / PR comment bot | Codecov、size-limit（"posts bundle size changes as a comment in pull request"，npm 官方页）【Fact/强】、axios/bundle-size、BundleMon | 把分析结果送到**评审现场**，制造"每次都看到"的曝光 | 需 JSON→comment 渲染层；M 建议在 PR comment 里只给 delta+链接，克制呈现 |
| 状态门禁 / Quality Gate | SonarQube Quality Gate、Codecov status checks、MemBrowse budget gate【Fact】 | 从"看"变"拦"——进入合并决策链，粘性最强 | `run_release_gate` use-case 已有，exit code 语义已定义，天然映射 |
| watch 模式 | jest --watch、cargo watch、vite dev【Fact/常识级】 | 本地即时反馈环，形成肌肉记忆 | 基线零出现；可做但须克制（见 §6） |
| 编辑器插件 / Agent 插件 | ESLint VS Code 扩展；MemBrowse Claude Code plugin【Fact】 | 在用户已有环境里"零跳转"出现 | 编辑器插件=基线未定的 ADR 级项；agent 插件是 2026 新变量，观察即可 |

---

## §2 "单一功能工具易被替代 vs 工作流枢纽难被替代"实证（对应要求②）

### 2.1 被吸收案例（单一功能 → 被枢纽吞掉）

- **Bower**【Fact/强】：官网至今保留官方劝退——"psst! While Bower is maintained, we recommend using Yarn and Vite for front-end projects."（bower.io，2026-09-27 访问）。包管理这一"单一功能"被 yarn/webpack/vite 工作流枢纽吸收，是官方盖章的实例。
- **同构现象（定性，弱-中源）**：minifier 被 bundler 内置（UglifyJS→Terser→esbuild/swc 内置 minify）、任务执行器被 npm scripts/内置取代（Gulp/Grunt 式微）——社区共识级，本轮未逐条取证，引用时建议标注【Opinion】。
- **嵌入式域的对应物**【Opinion】：`arm-none-eabi-size`/`objcopy` 单条命令正被"分析平台"吸收（Memfault 文章示范的自建 dashboard → MemBrowse 的商业化产品）；FirmwareSight 若停留在"单次分析器"，就是下一个被吸收对象——基线把它定位为"发布工作台"而非"可视化器"是对的方向。

### 2.2 枢纽沉淀案例

| 案例 | 端口路径 | 结局/粘性证据 |
|---|---|---|
| **PlatformIO**【Fact/强】 | 多平台构建系统 → VS Code 扩展 → 注册表（嵌入式横向枢纽） | 2019 年主动把原付费功能（Unified Debugger/Unit Testing/Remote）开源换取枢纽地位；变现走 Premium Support（Business $249/月，Enterprise 定制）【Fact，platformio.org】 |
| **Memfault**【Fact/强】 | 设备端 SDK（集成端口）→ 云端观察/OTA SaaS | 2025 年被 **Nordic Semiconductor 收购**（SimplyWall.st："Memfault, the IoT reliability and observability platform Nordic acquired in 2025"；Tracxn 标记 Acquired，融资 $35M+）——嵌入式"集成→枢纽→被并购"的完整闭环 |
| **SonarQube**【Fact/强】 | 静态分析 → Quality Gate（进入合并决策的流程位） | Developer Edition $34/月起（100k LOC），Server 版按实例/LOC 计费至数千美元/年【Fact，sonarsource.com pricing；appsecsanta 汇总 $750/yr 起】 |
| **Codecov**【Fact/强】 | coverage 上传器 → PR comment + status check → 团队订阅 | 2022 被 Sentry 收购，2026-06 再被 Harness 收购【Fact，Harness 官方新闻稿】——CI 集成位反复被并购定价；反面教材：2021 年 CI token 泄露波及 GoDaddy/Atlassian/P&G【Fact，Reuters/HN 200 分】 |

### 2.3 社区讨论摘录（带链接）

- **Ask HN: Are developer tools startups fighting a losing battle?**（2024-02，news.ycombinator.com/item?id=39282180）【Fact/中】：楼主以 Weaveworks 倒闭为例质疑 devtools 商业化；高赞回复（wmf）："**the same people who won't pay a cent for local dev tools will pay $100/month on SaaS**"——付费位在 hosted/集成层，不在本地单机功能。另一高赞（PaulHoule）举 JetBrains 反例证明"深度桌面工具也可收费"。→ 与 FirmwareSight "本地优先 + Team 档收费（CI 归 Team）"的基线设想同构【Opinion】。
- **r/cpp: "Predicting executable size with C++ makes it very hard..."**（reddit.com/r/cpp/comments/nx6a3i）【Fact/中】：高赞抱怨原文——"**Tools for analyzing where binary size comes from (bloaty, godbolt) exist, but aren't integrated into developer workflows.**"（分析工具存在，但没接入开发者工作流）——用户亲手写出的"端口缺口"。
- **Bloaty 发布帖 HN 202 分**（news.ycombinator.com/item?id=13915324，2017）【Fact】：size 分析域的社区声量峰值，说明该话题自带注意力，端口化后的传播杠杆可期【Opinion】。

### 2.4 抽象：粘性的三层来源【Opinion，基于以上证据】

1. **数据引力**——历史积累在工具里（Codecov 的 coverage 历史、MemBrowse 的 memory timeline、FirmwareSight 的 SQLite build history）；数据越久越难搬走。
2. **流程位**——站在必经之路上（CI/PR/合并门禁/发布归档）；"可选工具"会被替代，"必经一步"不会。
3. **协作网络**——团队共享同一份状态（Team policy、shared release evidence，基线 Growth 项）。
   单一功能工具只有第 4 条弱粘性：**体验优势**。体验会磨损，流程位和数据不会。

---

## §3 嵌入式小型团队构建/发布工作流：可接入点普查与需求信号分级（对应要求③）

### 3.1 接入点 × 惯例 × 证据

| 接入点 | 行业现状惯例 | 证据与强度 |
|---|---|---|
| **build 后处理** | Makefile/CMake 尾部挂 `arm-none-eabi-size`/`objcopy` 脚本；GCC `-Wl,--print-memory-usage`；IDE 的 post-build steps（CubeIDE/Keil 均有此机制） | 【强】Memfault《Tools for Firmware Code Size Optimization》逐命令演示（interrupt.memfault.com/blog/best-firmware-size-tools）；【弱】KDE485 设备手册提到 CubeIDE post-build 加 CRC32（saelig.com PDF）——机制存在但属个例 |
| **PR 审查 / size delta 评论** | Memfault 教科书级示范：每次 PR 计算 code size delta（ex-Pebble 团队：653kB 塞 448kB ROM 的实战）；MemBrowse/cargo-bloat-action/size-limit 产品化该行为 | 【强】interrupt.memfault.com/blog/code-size-deltas（2020，至今是标准引用源）+ 三个产品的 README |
| **CI 门禁** | budget gate（MemBrowse "fails the build when you blow your budget"）；跨域惯例：Codecov status check、SonarQube Quality Gate | 【强】MemBrowse docs + SonarSource 定价页 |
| **发布归档** | GitHub Releases 附 .bin/.elf/.hex + SHA256 校验；签名固件入库 | 【中】Particle 官方 CI 文档描述 PR pipeline 与部署管线（docs.particle.io）；ElectronVector 2023 博客演示 GHA 分阶段固件投递——通用做法成立，逐条溯源一般 |
| **代码评审** | PR comment 即评审面（同上）；Job Summaries 作为轻量证据位（GitHub 官方 2022 特性） | 【中】bloaty-action README 明示支持 Job Summary + PR comment 示例 |
| **版本管理** | `git describe` 注入版本串是嵌入式惯例；CI 从 start 就跑（"I always have CI from the start so I can track binary size and flash usage"） | 【中/弱】后者来自 Reddit r/embedded 检索摘要（u/OldTap7，原帖链接待补，**待证实**）；git describe 惯例为常识级 |

### 3.2 需求信号清单（来源链接 + 强/弱分级）

| # | 信号 | 来源 | 分级 |
|---|---|---|---|
| S1 | "region ROM overflowed by 16 bytes…是噩梦般的终端输出"→自建 size dashboard+PR delta | Memfault Interrupt（ex-Pebble 实战文），interrupt.memfault.com/blog/code-size-deltas | **强** |
| S2 | 商业公司愿意为"size/内存 CI 追踪"付费：MemBrowse 一年内集齐 Flipper/wolfSSL/NuttX/RT-Thread/TinyUSB 客户墙 | github.com/membrowse/membrowse-action README + membrowse.com/pricing | **强** |
| S3 | "分析工具存在，但没接入开发者工作流"——用户点名的端口缺口 | r/cpp，reddit.com/r/cpp/comments/nx6a3i | **中**（原文帖，摘要取证） |
| S4 | 社区专门发帖问"如何在 CI 里追踪二进制体积"（MemBrowse 在回复里营销） | r/cpp，reddit.com/r/cpp/comments/1csmz0a | **中** |
| S5 | "我从项目第一天就上 CI，就为了 track binary size and flash usage" | r/embedded（u/OldTap7），仅检索摘要级 | 弱/**待证实** |
| S6 | "减少编译产物体积是我们的强需求"（求非常规技巧） | r/cpp_questions，reddit.com/r/cpp_questions/comments/1ox6xhb | 弱 |
| S7 | 竞品内容营销持续加温（MemBrowse 博客对比 Bloaty 等，2026-01） | membrowse.com/blog/firmware-memory-footprint-tracking | 中（商业口径，注意利益相关） |

> **判断**【Opinion】：强信号集中在"**PR 级 size delta + 历史追踪**"这一格；"发布归档/版本管理"方向只有惯例没有呼喊——恰好对应基线把 CI 归 Growth、把 Release Bundle 归 P0 的排序，外部证据与内部排序**不冲突、互相印证**。

---

## §4 面向小众专业人群：做深体验、建立依赖的产品策略案例（对应要求④）

| 案例 | 人群 | 深耕手法 | 可移植经验 |
|---|---|---|---|
| **Proxyman**（macOS HTTP 调试代理）【Fact/中】 | 移动/后端开发者（窄） | 免费档+**$89 一次性**标准档；indie 月收入约 $3.5k（boringcashcow 专访口径） | 窄人群养得活单人深度开发；一次性买断在专业工具社区是**道德资本** |
| **PlatformIO**【Fact/强】 | 嵌入式（窄且分散） | 横向枢纽（一个 CLI 吃下所有平台）+ 把付费功能开源换生态位 + 收 Premium Support（$249/月） | "枢纽免费、服务收费"模式在嵌入式成立 |
| **Memfault**【Fact/强】 | 量产 IoT 设备团队 | SDK 集成（设备侧端口）→ 云端 dashboard → 被 Nordic 收购 | 从"开发期工具"长成"量产期基础设施"，粘性沿设备生命周期延伸 |
| **SEGGER 生态**【Fact，上轮已证+本轮补充】 | 嵌入式专业者 | Ozone 随 J-Link 硬件免费（硬件捆绑）；SystemView 商业授权 €1,480（非商业免费双轨） | 硬件/软件捆绑与双轨授权是嵌入式域成熟的定价心理学 |
| **STM32CubeMX**【Fact/中】 | STM32 用户 | 代码生成反向锁定（项目文件离开它无法再生） | 即使质量被长期抱怨（HAL BUG 帖），流程位锁定依然有效——**做枢纽不必完美** |

**归纳**【Opinion】：小众深耕的公共配方 = ①数据纵深（历史/对比，"走得慢"）+ ②确定性输出（可存档、可审计的证据，而非营销式仪表盘）+ ③公平收费（一次性/开源免费/双轨授权）+ ④功能克制但位居必经。全部四条与基线"仪器感/证据优先/四动词纪律"气质兼容，无一条要求 FirmwareSight 变成 dashboard 产品。

---

## §5 集成/粘性类功能的付费意愿与收费先例（对应要求⑤）

### 5.1 定价锚点表（全部 2026-09-27 访问核验）

| 产品 | 收费对象 | 价格 | 计费维度 | 端口相关性 |
|---|---|---|---|---|
| **MemBrowse** | CI 历史追踪+PR diff+门禁（云端） | Open Source 免费（public repo）/ Team **$99/月**（10 targets，年付）/ Growth $249/月（40 targets）/ Enterprise 定制（可 on-prem） | **tracked targets**；"Unlimited users on every plan" | **嵌入式 size 域最直接先例** |
| **Codecov** | coverage 集成 | OSS 免费；Team **$5/人/月**、Pro $12/人/月（guideflow 2026 汇总口径） | per user（开发者席位） | "PR comment 是其最有价值功能"（dev.to 2026 评述原文口径） |
| **SonarQube** | Quality Gate（Server） | Developer Edition $34/月起（100k LOC）；按实例+LOC 阶梯至数千美元/年 | per instance + LOC | 把"分析"变成"门禁"后才能卖出 Server 版 |
| **SEGGER SystemView** | trace 可视化授权 | 商业 €1,480（非商业免费） | per license | 嵌入式工具直接卖授权的先例（FS-RSCH-011 已证） |
| **PlatformIO** | 枢纽本体 | 免费；Premium Support Business $249/月 | per org（服务） | "枢纽免费、支持收费" |
| **Proxyman** | 桌面深度工具 | $89 一次性（免费档并存） | one-time | 窄人群桌面工具的生存线参考 |

### 5.2 归纳【Opinion，证据充分】

1. **收费位在"流程"不在"产物"**：五类先例全部对"持续集成进工作流的位"收费（云端历史/门禁/席位/支持），无一例对"单份报告导出"收费——**二次确认 FS-RSCH-011 的"导出=免费入口能力"结论**。
2. **嵌入式域出现了新计费维度**：MemBrowse 的 per-target 计费（不按人头）贴合资产逻辑（每个被追踪的固件目标=一条产线资产）。若 FirmwareSight 未来定价，"按 project/target"比"按席位"更贴用户心智，且与"Unlimited users"式宽松姿态兼容。
3. **团队档锚点同构**：基线 FS-DEL-001 把 CI integration/Team Policy 归 Growth+Team 档——外部同构证据（MemBrowse/Codecov/SonarQube 均以 CI/PR 能力作为团队档核心卖点）支持这一锚定，但不构成 R1 付费假设的验证（R1 仍需访谈/POC，**本报告不解除该 Unvalidated 状态**）。

---

## §6 "更多端口"的机会与陷阱初判（对应要求⑥）

### 6.1 机会（按基线插槽成本从低到高排序）

| 序 | 端口 | 外部证据支持度 | 与基线的关系 | 成本档 |
|---|---|---|---|---|
| 1 | **CLI headless + 稳定 `--json`** | 一切集成的行业公理（§1.4 第一行） | 架构明文预留（"future CI/headless 不需要重写"，FS-TECH-002）；CLI 已是平级 Interface | **S（已预留）** |
| 2 | **官方 GitHub Action（薄壳包 CLI）+ Job Summary / PR comment** | §3 全部强信号所在格；cargo-bloat-action/bloaty-action/MemBrowse 三重样本 | Growth 已锚（FS-DEL-001）；形式=新 use-case 不越 IPC 边界（CI 侧走 CLI 出口，不经桌面 IPC） | **S–M** |
| 3 | **CI 门禁（gate→exit code→status check）** | SonarQube/MemBrowse 证明"从看变拦"粘性最强 | `run_release_gate` use-case + exit codes 0/2/3/4/5 已定义，天然映射 | **S–M** |
| 4 | **watch 模式（本地）** | jest --watch/cargo watch 惯例；无云依赖、克制呈现可行 | 基线零出现；须防状态噪音（与设计侧"防 dashboard 化"议题衔接） | **M** |
| 5 | **build history 数据引力** | Codecov/MemBrowse 的历史时间线是最强粘性件 | SQLite `gate_runs/gate_findings/release_records` 表已预留；注意 SQLite 非交换格式的红线不破 | **M（数据已在，缺呈现）** |
| 6 | **IAR armclang / Segger MAP adapter** | MemBrowse 已支持 ICF/emProject——慢一步就成"它支持我们不支持" | MapAdapter `future adapters` 明文插槽 + fixture 门禁 | **L** |
| 7 | GitLab CI 模板/其他 CI | MemBrowse 三线并进（GHA/GitLab/Jenkins 文档） | CLI 先行即可覆盖（"CI 不解析自然语言"纪律下，任何 CI 都能消费 `--json`+exit code） | **S（文档级）** |

### 6.2 陷阱

1. **端口=供应链攻击面**【Fact】：Codecov 2021 Bash uploader 泄露事件波及 GoDaddy/Atlassian/P&G（Reuters；HN 200 分）——官方 Action/上传通道必须最小权限（只读源码/只传摘要 JSON）、且**本地优先本身就是对这一风险的差异化回答**【Opinion】。
2. **别做"云 portal 跟随者"**【Opinion】：MemBrowse 已占"云端历史+PR diff+门禁"生态位并有一年客户积累；正面仰攻云订阅是弱势开局。FirmwareSight 的错位=本地优先（数据不出内网）+ 自包含 HTML 证据（可离线归档进 Release）——这恰是云产品商业模式上做不了的。
3. **GitHub-only 绑定**【Opinion】：官方 Action 之前必须有 CLI 优先级，否则 GitLab/Jenkins 用户（嵌入式公司内网 CI 常见）被挡在门外；基线"CLI 不解析自然语言"的纪律恰好保证了 CI 无关性。
4. **端口=长期维护承诺**【Opinion】：每个端口都是新支持面/兼容面（Action 的 inputs 演进、GitLab 模板腐化、JSON schema 坏契约）。窄人群下应"一个 CLI 出口 + 薄壳集成"，而不是 N 个胖客户端；`--json` schema 一旦发布即公共契约（基线已有此口径，外部教训支持严格执行）。
5. **集成剧场**【Opinion】：demo 好看但非必经的集成（如又一个导出格式）不产生粘性。验收应回到用户金句与基线指标："让用户走得慢/不想走" + B1 real recurring use + MVP Return intent ≥5/8。
6. **红线不越**【硬约束】：IDE 插件/LSP（鲁班七号侧属 ADR 级评估）、调试器/烧录器/OTA fleet 运营均属 Non-goals 或 ADR 项；agent 插件（MemBrowse 的 Claude Code plugin 形态）建议只观察不跟进，直至有用户信号。

### 6.3 一句话 Agent 结论

> **端口扩展的正确姿势不是"加更多出口"，而是"占一个别人替代不掉的流程位，然后把所有出口收敛到同一个证据内核"**——外部证据链：CLI `--json`（行业公理）→ PR size delta（Memfault 实战+三个产品验证）→ 门禁（SonarQube/MemBrowse 定价验证）→ 历史数据引力（Codecov/MemBrowse 的时间线），每一环都有现成先例可抄作业；唯一要避开的是"云 portal"正面战场，唯一要补的是访谈验证 R1（外部先例≠我们的付费验证）。

---

## §7 与 FS-RSCH-011 的衔接及残留缺口

- **衔接**：①上轮"puncover JSON→CI 路线"本轮获商业级验证（MemBrowse 整条产品线即该路线）；②上轮"嵌入式 size 域 HTML 报告空位"仍成立——MemBrowse 的呈现物是云端 portal，**自包含 HTML 证据报告依旧无人做**；③上轮"导出免费"结论二次确认（§5.2）。
- **残留缺口（诚实清单）**：
  1. MapView 售价仍未取到（上轮遗留）；
  2. MemBrowse 免费档/付费档的具体限制细节（除 targets 数外）未见公开明细，需注册才能探明；
  3. Tower/Beyond Compare 等小众工具精确定价本轮未取到（§4 已用 Proxyman 替代锚点）；
  4. u/OldTap7（r/embedded）原帖永久链接待补——当前仅检索摘要级；
  5. MemBrowse 客户墙为商业自述口径，未逐家向客户侧交叉验证【中置信】。

---

## 附录：信源清单（访问日期均为 2026-09-27）

**一手·官方/产品（强）**
1. github.com/membrowse/membrowse-action（README：形态/客户墙/Claude Code 插件；44★，pushed 2026-09-24）
2. membrowse.com/pricing（Free/$99/$249/Enterprise，按 tracked targets）
3. docs.membrowse.com（Getting Started：上传口径/预算门禁/PR comment）
4. platformio.org（Premium Support 定价）+ docs.platformio.org（定位）+ community.platformio.org（2019 付费功能开源公告）
5. sonarsource.com/plans-and-pricing（$34/月起，按 LOC）+ sonarsource.com/plans-and-pricing/sonarqube（按实例计费说明）
6. bower.io（官方弃用劝退声明）
7. about.codecov.io + github.com/marketplace（Codecov Marketplace 定价口径）
8. harness.io/press-and-news（"Harness Acquires Codecov from Sentry"，2026-06）
9. github.com/orf/cargo-bloat-action（115★，README 全文）+ github.com/carlosperate/bloaty-action（README 全文，Job Summary/PR comment/Docker）
10. github.com/espressif/esp-idf-ci-action（108★）

**权威博客/媒体（强-中）**
11. interrupt.memfault.com/blog/code-size-deltas（2020-03-18，Tyler Hoffman）
12. interrupt.memfault.com/blog/best-firmware-size-tools（2019-06-06，François Baldassari）
13. reuters.com（Codecov breach，2021-04）+ about.codecov.io/security-update
14. blog.sentry.io（Codecov 入 Sentry，2022-11）
15. simplywall.st（Nordic 2025 收购 Memfault）+ tracxn.com（Memfault Acquired 档案）
16. hackaday.com（Six STM32 IDEs 评测，2017-04：CubeMX 依赖面）
17. boringcashcow.com（Proxyman 专访：licensing 变现、$3.5k/月）+ macappsdaily.com（$89 one-time，2026-08）

**社区讨论（中，含摘要级标注）**
18. news.ycombinator.com/item?id=39282180（Ask HN: devtools losing battle，2024-02）
19. news.ycombinator.com/item?id=13915324（Bloaty 发布帖，202 分，2017-03）
20. news.ycombinator.com/item?id=45979698（Show HN: MemBrowse，2025-11-19）
21. reddit.com/r/cpp/comments/nx6a3i（"tools …aren't integrated into developer workflows"）
22. reddit.com/r/cpp/comments/1csmz0a（"How to track your binary size in CI"）
23. reddit.com/r/cpp_questions/comments/1ox6xhb（binary size 强需求）
24. r/embedded u/OldTap7（CI from start / track size——仅摘要级，**待证实**）
25. docs.particle.io（Github actions CI/CD 文档：PR pipeline）
26. membrowse.com/blog/firmware-memory-footprint-tracking（2026-01，竞品内容营销）
27. npmjs.com/package/size-limit（PR comment 形态）+ github.com/axios/bundle-size
28. saelig.com KDE485 手册（CubeIDE post-build steps 个例，弱）

---

*报告完。下一步建议：本报告 §6.1 的端口排序可与鲁班七号《技术可行性简报》的架构插槽分级做交叉表，交设计匠人做呈现克制方案，最终由管理员与用户裁决接入顺序。*
