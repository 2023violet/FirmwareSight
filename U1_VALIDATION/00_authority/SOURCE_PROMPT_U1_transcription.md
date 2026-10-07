# U1 source instruction — agent transcription, NOT an owner-delivered file

Delivery status: the owner supplied this prompt as inline message text in session
`78e09536-12ab-4712-adbe-865215e0a59f` on 2026-10-06. Unlike every round whose prompt arrived as an
attachment file, there is no owner-side byte stream to hash here. The text below is a transcription
made by the executing agent from the delivered message. It is archived here, in
`U1_VALIDATION/00_authority/`, and NOT in `10_AUDIT/SOURCE_PROMPTS/`: that directory holds copies whose
SHA-256 proves delivered bytes, and no such byte stream exists for this track. The directory's README
records this prompt as `File: none (delivered inline)`, the way the three inline P0 precedents are
recorded, and points here for the content. Section numbering, ordering and wording follow the delivered
text.

Fidelity limit, stated rather than assumed: this is a faithful transcription of content and
structure, **not** a byte-identical copy. The delivered text used full-width CJK quotation marks
(" ") in roughly twenty places; they are rendered here as ASCII straight quotes, and the filename
`..._verbatim.md` therefore overstates what this file is. The SHA-256 below is the digest of this
transcription and MUST NOT be cited anywhere as the digest of the owner's prompt. If exact
provenance for this track is ever required, the owner should supply the prompt as a file the way
every P5 and V1 round did, and the archive entry will then carry both digests.

Provenance decisions taken by the owner in the same session, before any file was written:
- the delivered title calls this track `B1`; `B1` is already the canonical identifier for Private
  Beta in `06_DELIVERY/06_STAGE_GATES.md`, and both F3 §37 and V1 §40 record `B1 = NOT AUTHORIZED`.
  The owner chose to register this track as **`U1_UI_PRODUCTIZATION_CONVERGENCE`** instead, so the
  beta identifier stays reserved and no beta authorization is implied. The section text below still
  reads `B1` as delivered; where this round writes a token it writes `U1`.
- the delivered §0 start state (`active_task = NONE`) was true at F3's head `08fdfcb`. The V1 round
  authorized afterwards moved `active_task` to `V1_OWN_ARTIFACT_EXTERNAL_VALIDATION` at `f481b78`.
  The owner chose: pause V1 with its state intact, activate U1.
- the delivered §2C makes the mockups design authority. The owner chose: keep the frozen precedence
  chain of ADR-0018 / AGENTS.md 11 (治理红线 > frozen assets/ADRs/tokens > DESIGN.md > accepted
  screenshots), so where a mockup shows a treatment the design system forbids, structure converges
  and the treatment does not, and the case is recorded.

---

《FirmwareSight B1 — UI Productization / Design Convergence》
版本：v1.0
性质：新轨道 / 产品界面收敛
目标：让当前 FirmwareSight 从"工程完成的 MVP Candidate"进一步收敛到"更接近产品预期效果的 UI"

0. 你现在接手的不是一个"从零开始设计"的项目

FirmwareSight 当前不是空壳，也不是纸面原型。
它已经完成了 MVP 主线与 Productization 主线，当前状态可概括为：
- P5 = PASS_COMPLETE
- Productization = ENGINEERING_COMPLETE
- Product state = MVP_CANDIDATE
- active_task = NONE
- 核心产品能力已存在并可运行
  - Analyze
  - Compare
  - Release Gate
  - Bundle / History
  - Packaging / Diagnostics / Installed validation / productization evidence
因此，这一轮工作的性质不是：
- 重写产品定义
- 改写核心语义
- 推翻已有实现
- 做一套"漂亮但脱离当前产品"的 UI 壳子
这一轮工作的性质是：
在不破坏现有稳定工程基线的前提下，让现有桌面产品 UI 朝着目标产品设计方向收敛。

1. 本轮任务名称

B1 — UI Productization / Design Convergence
这是一个新的产品体验轨道。
它不是：
- P5 续写
- V1 外部验证
- Beta / RC / GA
- 重新定义产品
它是：
对当前已完成工程主线的产品界面进行视觉、布局、信息架构与交互表达层面的升级。

2. 输入权威（Authority）

你必须同时以以下内容为权威来源：

A. 当前仓库与当前可运行产品
先读取并确认：
- 当前 HEAD
- origin/main
- git status --short
- git diff
- git diff --cached
你必须先确认工作树状态，再开始任何改动。
如果仓库有未授权的远端移动或未解释的脏状态，先停下并报告。

B. 当前产品已实现能力
你必须尊重并保留当前产品已有的真实能力边界，包括但不限于：
- Analyze
- Compare
- Release Gate
- Bundle & History
- 既有的状态语义（PASS / REVIEW / BLOCK / UNKNOWN / N/A）
- 既有的 evidence / diagnostics / bundle / manifest / schema / release identity 语义
- 既有的 local-first / nothing leaves this machine 原则
不要因为 UI 收敛而擅自改变这些语义。

C. 本轮 UI 目标参考 —— 使用这次会话里上传的 UI 参考图
本轮请把这批参考图视为设计方向权威。
它们至少覆盖了这些目标页面 / 状态：
1. Release Gate
2. Bundle & History
3. Analyze — parse failure / invalid ELF 状态
4. Analyze — Dependencies
5. Overview
6. Analyze — Sections / Symbols
7. Compare
这些图用于定义：
- 产品视觉方向
- 布局风格
- 页面层级
- 卡片、表格、侧栏、状态条、按钮、chip、badge 的表达方式
- "像产品"的观感目标
但它们不是无条件的功能强制合同。
也就是说：
- 如果参考图里出现的功能，当前产品已经有真实后端支持 → 可以实现到位。
- 如果参考图里出现的功能，当前产品尚不具备真实能力支撑 → 你可以吸收其视觉语言与布局模式，但不能伪造后端语义。
例如：
- 如果某个 mockup 出现了"Declare version..."一类交互，而当前产品并没有对应完整能力，就不要硬造一套假逻辑来冒充完备功能。
- 可以记录为：
  - DESIGN_REFERENCE_ONLY
  - VISUAL_PATTERN_ADOPTED
  - NOT_IMPLEMENTED_IN_B1
  - 或等价、诚实、不夸大状态的记录语言。

3. 本轮目标

本轮的核心目标不是"变好看一点"，而是：

目标 1：让当前 UI 更像真正的产品，而不是内部工具页
要达成：
- 页面结构更稳定
- 信息层级更清晰
- 组件体系更一致
- 视觉密度更成熟
- 状态表达更可信
- 空态、错误态、结果态更像产品

目标 2：让关键页面更容易理解
尤其是：
- Analyze：先看结论，再看细节
- Compare：先看变化，再看证据
- Release Gate：先看"能不能发"，再看为什么
- Bundle & History：先看历史轨迹，再看右侧详情

目标 3：让 FirmwareSight 的"技术密度"变得可读，而不是压迫
FirmwareSight 不是轻娱乐产品。
它有天然的技术信息密度。
所以不要把它做成花哨大屏，也不要做成营销页。
它应该呈现出：
冷静、可信、专业、结构清晰、带工程判断力的产品感。

4. 本轮范围（Scope）

本轮应优先覆盖的页面
按这个顺序推进：

第一优先级
1. Overview
2. Analyze
3. Compare
4. Release Gate
5. Bundle & History

第二优先级
如果时间与结构允许，再考虑：
- 通用 shell
- 左侧导航
- 顶部项目头
- 顶部"local workspace / nothing leaves this machine"信息条
- 通用状态 chips / badges / panels / side detail cards / tab styles / empty states / error states

5. 本轮严格禁止事项

以下内容默认禁止，除非你证明是实现 B1 必不可少的最小改动，并在报告中明确说明：

禁止改动的主线语义
- 不重写产品定义
- 不重写 Gate 规则定义
- 不改写 Release identity 语义
- 不改写 evidence 语义
- 不改写 bundle 语义
- 不改写历史记录含义

禁止随意改动的底层范围
- Schema / migrations
- Storage contracts
- Analysis wire format
- Release identity rules
- ADR-0028 / ADR-0029 既有结论
- package / CI / productization evidence 除非确有必要

禁止伪造新功能
- 不要为了贴 mockup 而硬造假的行为闭环
- 不要把不存在的后端能力包装成"已支持"
- 不要新增没有 authority 的产品承诺

禁止把这轮做成"全面重构"
- 不要重写所有页面逻辑
- 不要大范围重拆业务数据流
- 不要借 UI 收敛之名顺手做一堆 unrelated engineering

6. 你必须先做的事：B1-A0 设计差距审查

在改任何代码前，你必须先完成一个设计差距审查（Gap Audit）。
请输出一份类似如下结构的审查结论（可写入仓库文档，也可作为本轮报告的一部分）：

B1-A0 — UI Gap Audit
至少覆盖：

A. 当前产品与参考图一致的地方
例如：
- 左侧导航结构
- 产品整体冷静风格
- 状态型页面方向
- 局部信息面板形式

B. 当前产品与参考图差距最大的地方
例如：
- 页面层级不够清晰
- 顶部层信息密度不够成熟
- 各页标题 / metadata / 操作条不统一
- 结果区域与细节区域没有形成清晰主次
- 辅助信息卡不够"产品化"
- 表格状态样式与 badges 一致性不够
- 错误态 / 空态 / 未提供态不够成熟
- 通用组件（按钮 / tabs / chips / section cards）观感不够统一

C. 哪些 mockup 可直接落地
D. 哪些 mockup 只能吸收视觉模式、不能直接复制功能
E. 本轮 UI 收敛优先级列表
F. 本轮明确不做的内容

7. 设计收敛原则（必须遵守）

7.1 先建立统一设计系统，再逐页收敛
不要先东修一个按钮、西修一个表格。
应先统一：
- spacing
- card radius / border / elevation
- typography hierarchy
- status colors
- tabs
- chips / pills / badges
- table headers / row selection / side detail panel
- action bar / page title / metadata row
- empty state / error state / warning state

7.2 先提升表达，再考虑新增交互
优先级应该是：
1. 表达优化
2. 信息架构优化
3. 页面操作路径优化
4. 必要的轻量交互增强
而不是一上来新增很多功能。

7.3 维持 FirmwareSight 的"专业冷静感"
避免以下风格误区：
- 过度营销风
- 过度花哨动效
- 夸张渐变与装饰
- 华而不实的视觉噪声
- 信息层级被装饰打乱

7.4 保持 local-first 产品气质
产品视觉上应持续强化以下认知：
- 所有分析留在本机
- workspace 是本地的
- nothing leaves this machine
- 工程判断是可追溯的
- 结果不是"猜的"，而是有证据的

8. 页面级目标

8.1 Overview
目标：
- 让用户一眼理解"当前这个 artifact 的总体状态"
- 更像一个产品首页，而不是零碎信息堆叠
- 明确表达：
  - ELF / MAP / Git 的状态
  - 当前能否 ship
  - 最新 gate 结论
  - 下一步操作
  - 关键摘要卡（flash used / largest section / symbols / last gate）
重点：
- 顶部 artifact header
- 三状态小卡（ELF / MAP / Git）
- "Can we ship now?" 大结论卡
- 次级摘要卡布局

8.2 Analyze
目标：
- 这是用户最容易感知产品成熟度的页面
- 需要同时支持：
  - 成功分析态
  - 错误态 / parse failed
  - last-good 相关提示
  - sections / symbols / evidence / dependencies（如果当前真实能力允许）
重点：
- 页面顶部状态 chips
- 页面主标题与上下文
- sections 表 + 右侧 detail panel
- symbols 区域
- 状态 / counts / capabilities 的结构化呈现
- bytes / KiB 切换的可见性与可信度
- 错误态卡片质量
- 空态与诊断信息布局

8.3 Compare
目标：
- 一眼看到 old / new 的关系
- 一眼看到增减结论
- 表格与右侧 summary / contributors 更像产品而不是调试输出
重点：
- baseline / target 头部表达
- top summary line
- pass/review badges
- biggest growth contributors
- section diff table
- added / changed / removed / unchanged 的视觉区分
- 导出按钮位置与权重

8.4 Release Gate
目标：
- 这是最关键的决策页面之一
- 用户必须能一眼看懂：
  - 当前是否能发
  - 为什么不行
  - 哪些是 BLOCK
  - 哪些是 REVIEW
  - 哪些是 UNKNOWN
  - 哪些已经 PASS
重点：
- 顶部大 verdict 条
- 左侧规则主列表
- 右侧 bundle preview
- next step / accept review / rerun gate 按钮层级
- 分组块（BLOCK / REVIEW / UNKNOWN / PASS / N/A）

8.5 Bundle & History
目标：
- 左侧历史轨迹 + 右侧详情面板结构清晰
- 更像产品中的"历史与详情"页
- bundle 详情更可信、更稳重
重点：
- history table 的信息密度控制
- selected 状态行
- 右侧详情卡
- verify / export / open folder 按钮层级
- 内容列表的条目视觉

9. 技术实施约束

9.1 最小破坏原则
优先：
- CSS / styling
- layout composition
- UI components
- presentational refactors
- page structure cleanup
谨慎：
- 数据流重构
- 大面积组件替换
- 横向行为修改

9.2 如果需要新组件
可以新增统一组件，但必须：
- 命名清晰
- 复用性明确
- 不制造重复体系
- 不引入审美冲突
- 不引入过重依赖

9.3 设计 token
如果当前已有 token / style 基线，请优先在既有体系内演进。
不要轻率引入一套完全平行的新 token 体系。

10. 本轮允许的"功能边界微调"

以下情况允许做轻量改动：
- 为了让页面结构成立而调整已有组件层次
- 为了让 mockup 方向成立而补充当前页面已有真实数据的展示方式
- 为了统一产品感而增加轻量辅助文案、状态说明、空态说明
- 为了改进布局而引入右侧 detail panel、摘要卡、元信息条等结构

以下情况默认不允许：
- 新增需要 Core / storage / migration / wire 支撑的完整新功能
- 新增会改变产品定义的新流程
- 新增没有 authority 的承诺能力

11. 本轮验证要求

你不能只说"我改好看了"。
必须完成验证。

11.1 回归验证
至少验证：
- Rust / UI / check.py 等既有 gate 不退化
- 页面能正常构建运行
- 关键页面不出现明显功能回归
- 关键按钮仍可用
- 关键状态页仍正确表达

11.2 视觉收敛验证
至少提供：
- 改造前 / 改造后 对照说明
- 与参考图的收敛点
- 仍然未达到的部分
- 哪些只是视觉收敛，哪些已经做到结构收敛
- 哪些功能因当前能力边界而只吸收了设计语言，没有完整实现

11.3 页面级验证
至少逐页给出：
- Overview
- Analyze
- Compare
- Release Gate
- Bundle & History
每一页：
- 改了什么
- 为什么改
- 与 mockup 对齐了什么
- 还差什么
- 是否引入功能风险

12. 交付物（Deliverables）

本轮至少交付：
A. B1 UI Gap Audit
设计差距审查
B. B1 Design Convergence Plan
说明本轮设计策略、组件策略、页面顺序、边界
C. 实际代码修改
在 repo 中完成 UI 收敛实现
D. B1 Validation Report
记录：
- 改动范围
- 页面变化
- 验证结果
- 与 mockup 的对齐程度
- 未完成项
- 风险与后续建议

13. 报告中必须诚实记录的内容

你必须明确写清楚：
- 哪些页面完成了高质量收敛
- 哪些页面只完成了第一轮产品化
- 哪些参考图功能没有真实后端能力支撑，因此没有做成"真功能"
- 哪些只是视觉靠近，不应夸大为"完全实现"
- 哪些交互仍需未来继续 refinement

14. 停止条件（Stop Conditions）

若出现以下情况，立即停下并报告，不要硬做：
1. 需要改动 storage / migration / schema / release identity 才能满足 mockup
2. 需要伪造新功能才能"像参考图"
3. 需要大面积重写当前产品逻辑
4. 当前仓库状态与 authority 严重不符
5. 收敛需求与既有核心语义冲突

15. 最终目标表述（不要过度承诺）

本轮完成后，你可以把结果描述为：
FirmwareSight 的 UI 已从工程完成态的 MVP Candidate，收敛到更接近产品预期的桌面产品界面。

但不要写成：
- 已完成 V1
- 已商业发布
- 已进入 GA
- 已完成全部产品设计
- 已完全等同于 mockup
- 已完成所有未来交互

16. 执行顺序（建议）

按这个顺序执行：
1. Preflight
2. 读取当前 UI 与参考图
3. 产出 B1-A0 Gap Audit
4. 先统一通用设计语言 / 组件风格
5. 优先改造：
   - Overview
   - Analyze
   - Compare
   - Release Gate
   - Bundle & History
6. 做验证
7. 输出 B1 Validation Report
8. STOP，不擅自开启下一阶段

17. 你现在的直接任务

请现在开始执行 B1 — UI Productization / Design Convergence：
- 基于当前仓库
- 基于当前已实现产品能力
- 基于这次提供的 UI 参考图
- 在不破坏核心语义与产品基线的前提下
- 对 FirmwareSight 的桌面 UI 做一次有边界、有验证、真实收敛的产品化提升
完成后给出完整报告，并停止。
