# 情报报告：VoltAgent/awesome-design-md 仓库调研 & FirmwareSight 设计风格选型

> 报告人：信息哨兵 ｜ 日期：2026-09-27 ｜ 信源：GitHub API + raw 文件直抓（全部 74 份 DESIGN.md 已完整下载核验）
> 置信度：⭐⭐⭐⭐⭐（基于仓库原始文件逐份取证，非二手转述）
> 接收：群管理员 ｜ 关联项目：FirmwareSight（嵌入式固件发布工作台）

---

## 摘要

`VoltAgent/awesome-design-md` 是一个 2026-03 创建、GitHub 118k stars 的精选集：把 74 个知名品牌/产品的设计系统逆向提取成标准化 `DESIGN.md` 文档（Google Stitch 提出的"AI 可读设计系统规范"），供 AI 编码代理直接消费生成 UI。**核心结论：IBM（Carbon）、Vercel（剥离渐变后）、WIRED、Cal.com 四个条目与 FirmwareSight 的 Instrument-grade clarity 哲学高度契合，其中 IBM/DESIGN.md 是首选基准——白底、直角、1px 发丝线、单一蓝 accent、语义状态色，几乎就是"工程仪器感"的字典定义。** 落地方式建议：将选定 md 改造为"FirmwareSight DESIGN.md"（保留 token 体系 + 删除营销模式 + 追加硬约束清单），置于仓库根目录并在 AGENTS.md 中声明为 UI 基准。

---

## ① 仓库结构与收录内容概览

### 基本信息
| 项 | 值 |
|---|---|
| 仓库 | https://github.com/VoltAgent/awesome-design-md |
| 定位 | "A collection of DESIGN.md files analysis by popular brand design systems. Drop one into your project and let coding agents generate a matching UI." |
| 规模 | 118,094 stars / 13,218 forks（GitHub 全球排名约 #150） |
| 时间线 | 创建于 2026-03-31，最近推送 2026-09-21（活跃维护中） |
| 许可 | MIT（可自由复制、修改、再分发，无品牌归属声明） |

### 目录结构
```
awesome-design-md/
├── README.md                  # 目录清单（73 条，按 10 个行业分类）+ 格式规范说明
├── CONTRIBUTING.md            # 贡献指南（先开 issue 再提 PR）
├── LICENSE                    # MIT
└── design-md/
    ├── ibm/
    │   ├── DESIGN.md          # ★ 核心资产：完整设计系统文档（20~44KB 不等）
    │   └── README.md          # 仅一行：指向 getdesign.md 站点的预览/下载页
    ├── airbnb/ … slack/       # 共 74 个风格目录
```
> 注意两点：① 每个风格目录内**只有** DESIGN.md + README.md 两个文件，README 中提到的 `preview.html`/`preview-dark.html` 实际托管在 getdesign.md 网站，不在 git 仓库内；② `slack` 条目存在于仓库但未列入 README 目录（README 徽章计数为 73，实际文件 74 份）。

### DESIGN.md 是什么
- 概念源自 **Google Stitch**：一份纯文本（Markdown）设计系统文档，AI 编码代理读取后即可生成视觉一致的 UI——定位相当于"给设计代理的 AGENTS.md"（AGENTS.md 管怎么构建，DESIGN.md 管长什么样）。
- 无需 Figma 导出、无 JSON schema、无需解析工具；Markdown 是 LLM 阅读效果最好的格式。
- 这些文档不是官方品牌指南，而是对公开网站 CSS 的逆向提取分析（含真实 hex 值、字号阶梯、间距体系）。

### 每份 DESIGN.md 的典型要素（两代格式并存）
**新版 9 节格式**（10 个较新条目：kraken / lamborghini / lovable / mastercard / runwayml / sanity / spotify / starbucks / tesla / theverge）：
1. **Visual Theme & Atmosphere** — 气质、密度、设计哲学
2. **Color Palette & Roles** — 语义色名 + hex + 功能角色
3. **Typography Rules** — 字体族 + 完整层级表
4. **Component Stylings** — 按钮/卡片/输入框/导航及各交互状态
5. **Layout Principles** — 间距标尺、网格、留白哲学
6. **Depth & Elevation** — 阴影体系、表面层级
7. **Do's and Don'ts** — 设计护栏与反模式清单
8. **Responsive Behavior** — 断点、触控目标、折叠策略
9. **Agent Prompt Guide** — 速查色板 + 即用型提示词

**基础版格式**（其余 64 份）：Overview（含 Key Characteristics 逐条拆解，质量很高）→ Colors（Brand & Accent / Surface / Text / Semantic 四组 token）→ Typography → Layout → Components → Elevation & Depth → Shapes → Do's and Don'ts → Responsive Behavior → Iteration Guide → Known Gaps。

> Agent 评注：两种格式对 AI 消费等价，关键共性是 **token 化（每色每字号都有语义名）+ 护栏化（Do's and Don'ts）+ 状态完备（hover/pressed/focus/disabled 全标注）**，这正是它比"截图给 AI 看"强得多的原因。

---

## ② 完整设计风格条目清单（74 条，名称 + 一句话特征）

标注：⚠️ = 与 FirmwareSight 红线冲突的主要维度（暗底 / 紫蓝渐变 / 玻璃拟态 / 霓虹 / 消费玩趣）。

### AI & LLM 平台（12）
1. **Claude** — 暖陶土色 accent + 干净的编辑部式排版（Anthropic AI 助手）
2. **Cohere** — 白底编辑部气质 + 深绿/藏青暗色带穿插，"实验室节奏"的企业 AI 指挥台 ⚠️(局部暗带)
3. **ElevenLabs** — 米白画布 + 粉彩渐变光球作唯一"颜色时刻"，轻字重杂志感 ⚠️(氛围渐变)
4. **Minimax** — 大胆的暗色界面 + 霓虹 accent ⚠️(暗底/霓虹)
5. **Mistral AI** — 法式工程极简，紫色调 ⚠️(紫)
6. **Ollama** — "README 即系统"的纸白文档美学、终端优先单色极简
7. **OpenCode AI** — 开发者向暗色主题 ⚠️(暗底)
8. **Replicate** — 奶油画布 + 橙色"图章式"accent（每屏至多一橙），三字体严格分工
9. **Runway** — 电影级暗色 hero + 纸白阅读带、单一专有无衬线、纯黑胶囊 CTA ⚠️(暗底)
10. **Together AI** — 蓝图式技术风 + 橙-品红-长春花三色渐变带 ⚠️(渐变/暗蓝底)
11. **VoltAgent** — 虚黑画布 + 祖母绿 accent、终端原生（本仓库维护方自家条目）⚠️(暗底)
12. **xAI** — stark 单色、未来主义极简

### 开发者工具 & IDE（7）
13. **Cursor** — 利落暗色界面 + 渐变 accent ⚠️(暗底/渐变)
14. **Expo** — 暗色主题、紧字距、代码为中心 ⚠️(暗底)
15. **Lovable** — 玩趣渐变 + 友好开发者气质 ⚠️(渐变/玩趣)
16. **Raycast** — 利落暗色 chrome + 鲜活渐变 accent ⚠️(暗底/渐变)
17. **Superhuman** — 高端暗色 UI、键盘优先、紫色辉光 ⚠️(暗底/紫)
18. **Vercel** — 黑白精密主义 + Geist 字体、200 级灰阶、发丝线表格（唯一装饰是品牌渐变，可剥离）
19. **Warp** — 暗 IDE 式界面、块状命令 UI ⚠️(暗底)

### 后端 / 数据库 / DevOps（8）
20. **ClickHouse** — 近纯黑画布 + 电光黄单色压强、数据密集、JetBrains Mono 代码窗口 ⚠️(暗底)
21. **Composio** — 现代暗色 + 彩色集成图标 ⚠️(暗底)
22. **HashiCorp** — 黑底企业净版，签名手法是"每产品专属 accent 色编码"（Terraform 紫/Vault 黄/Consul 红…）⚠️(暗底)
23. **MongoDB** — 深青 hero + 亮绿 CTA pill、白底文档与定价面 ⚠️(局部暗底)
24. **PostHog** — 玩趣刺猬品牌 + 开发者友好暗色 UI ⚠️(暗底/玩趣)
25. **Sanity** — 暗色优先编辑部风：112px 巨型标题 + IBM Plex Mono 眉题 + 珊瑚红单一最高优先级 accent ⚠️(暗底)
26. **Sentry** — 深紫夜空画布 + 青柠关键词高亮芯片 + 粉红标点，调试台美学 ⚠️(暗底/紫/霓虹)
27. **Supabase** — 暗祖母绿主题、代码优先 ⚠️(暗底)

### 生产力 & SaaS（7）
28. **Cal.com** — 白底 + 黑 CTA + 浅灰卡片，产品 UI 碎片直接嵌入卡片展示（"不画营销图，直接给你看真的"）
29. **Intercom** — 友好蓝调 + 对话式 UI 模式 ⚠️(消费向)
30. **Linear** — 全收藏最深的暗画布(#010102) + 薰衣草蓝 accent(#5e6ad2)、密集产品截图 ⚠️(暗底/紫蓝)
31. **Mintlify** — 天空渐变 hero + 3 栏文档密度（侧栏/正文/TOC）、Inter + Geist Mono ⚠️(渐变 hero)
32. **Notion** — 深藏青 hero + 标志性紫色 pill CTA + 多彩插画贴纸 ⚠️(紫/玩趣)
33. **Resend** — 极简暗色 + 等宽字 accent ⚠️(暗底)
34. **Zapier** — 暖橙 + 友好插画驱动 ⚠️(玩趣)

### 设计 & 创意工具（6）
35. **Airtable** — 彩色友好、结构化数据美学
36. **Clay** — 有机形状 + 柔和渐变、艺术指导式排版 ⚠️(渐变)
37. **Figma** — 鲜活多色、玩趣而专业
38. **Framer** — 大胆黑白蓝、动效优先
39. **Miro** — 亮黄 accent + 无限画布气质
40. **Webflow** — 蓝 accent、精致营销站美学

### 金融科技 & 加密（7）
41. **Binance** — 单色底上的币安黄、交易大厅紧迫感 ⚠️(暗底/紧迫感)
42. **Coinbase** — 干净蓝身份(#0052ff)、机构信任感、编辑部式沉稳、weight-400 显示字
43. **Kraken** — 紫 accent 暗色 UI、数据密集仪表盘 ⚠️(暗底/紫)
44. **Mastercard** — 暖奶油画布 + 轨道胶囊形、编辑部暖度
45. **Revolut** — 利落暗色界面 + 渐变卡片 ⚠️(暗底/渐变)
46. **Stripe** — 标志性渐变 mesh（奶油/橙/薰衣草/靛蓝/宝石红）+ 靛蓝 CTA + 细字重排版 ⚠️(渐变/紫蓝)
47. **Wise** — 鲜绿 CTA(#9fe870) + 鼠尾草底、weight-900 粗标题、友好清晰 ⚠️(消费向)

### 电商 & 零售（5）
48. **Airbnb** — 暖珊瑚 accent、摄影驱动、圆润 UI ⚠️(消费向)
49. **Meta** — 摄影优先、二元明暗表面、Meta 蓝 CTA
50. **Nike** — 单色 UI、巨型大写 Futura、满版摄影
51. **Shopify** — 暗色优先电影感 + 霓虹绿 accent、330 超细字重标题 ⚠️(暗底/霓虹)
52. **Starbucks** — 四层大地绿系统 + 暖奶油画布

### 媒体 & 消费科技（12）
53. **Apple** — 高级留白 + SF Pro、电影级影像、单蓝交互色(#0066cc)、全系统仅一处阴影
54. **HP** — 纯白画布 + 电光蓝信号 CTA(#024ad8)、几何 Forma DJR 字体、蓝色 V 形装饰
55. **IBM** — Carbon 设计系统忠实应用：白底 + 炭黑 + 单一 IBM 蓝(#0f62fe)、全局 0px 直角、1px 发丝线、无渐变无阴影
56. **NVIDIA** — 绿黑能量、技术力量美学 ⚠️(暗底)
57. **Pinterest** — 红 accent + 瀑布流、图像优先
58. **PlayStation** — 三表面渠道布局 + 青色悬停缩放
59. **SpaceX** — 纯黑白 + 满版影像/发射视频、全大写 D-DIN、电影片头式构图 ⚠️(暗底/摄影驱动)
60. **Spotify** — 暗底鲜活绿、粗体字、专辑封面驱动 ⚠️(暗底)
61. **The Verge** — 酸薄荷 + 紫外 accent ⚠️(霓虹/紫)
62. **Uber** — 黑白二重奏 + 紧凑工程感字型、无第三色、pill 签名形状
63. **Vodafone** — 纪念碑式大写标题 + 红色章节带
64. **WIRED** — 纸白报纸密度、三字体体系（衬线展示/衬线正文/无衬线标注）、全站方角按钮、黑白二重奏 + 单一墨蓝链接(#057dbc)

### 汽车（7）
65. **BMW** — 暗色高级表面、德式精密工程气质 ⚠️(暗底)
66. **BMW M** — 赛车运动对比 + M 三色 accent ⚠️(运动化)
67. **Bugatti** — 电影黑画布、单色严峻、巨型标题 ⚠️(暗底)
68. **Ferrari** — 明暗对照黑白编辑风 + 法拉利红、极致留白 ⚠️(暗底)
69. **Lamborghini** — 纯黑大教堂 + 金色 accent ⚠️(暗底)
70. **Renault** — 鲜活极光渐变 + 零圆角按钮 ⚠️(渐变)
71. **Tesla** — 激进减法、满版摄影、Universal Sans、单蓝 CTA(#3E6AE1)

### 复古网页 · Nostalgia 系列（2）
72. **Dell (1996)** — 目录时代企业网页：黑色页面边框、八色目录色带卡片、Helvetica-Black 标题 + Times 正文、GIF 贴纸
73. **Nintendo.com (2001)** — Y2K 主机铬合金风：斜纹金属面板、琥珀色导航、像素马里奥气泡

### 未列入 README（1）
74. **Slack** — 茄紫主色 + 奶油淡紫粉彩渐变 hero + 精细 UI 拟真卡片 ⚠️(紫/渐变)

---

## ③ FirmwareSight 候选筛选

**筛选基准（FirmwareSight 设计哲学）**：Instrument-grade clarity——工程仪器感、克制、证据优先、dense not crowded、浅色主题为主。**硬性排除项**：赛博朋克 / 霓虹 / 玻璃拟态 / 紫蓝渐变。

### 第一梯队：可直接作为 UI 基准

#### 🥇 T1-1 IBM（`design-md/ibm/DESIGN.md`，26KB）— 首选基准
- **设计要点**：忠实应用 Carbon 设计系统。纯白画布 + 浅灰 #f4f4f4 承载层级 + 炭黑 #161616 文字，IBM 蓝 #0f62fe 是唯一品牌 accent；**全局 0px 直角**（按钮、输入框、卡片全部方角）+ 1px 发丝线替代阴影，"no rounded pills, no soft shadows, no atmospheric gradients, The system is engineered, not stylized"（原文）；IBM Plex Sans 轻字重(300)大标题 + 正文 0.16px 正字距的精密细节；完整语义状态色（Success 绿 / Warning 黄 / Error 红 / Info 蓝）；聚焦态是 Carbon 标志性的"底部炭黑下划线"；层级靠 1px hairline 与表面色变，从不靠投影。
- **匹配理由**：与四条红线**零冲突**——无渐变、无玻璃拟态、无霓虹、浅色为主。语义状态色体系天然对应固件工作台的"通过/警告/失败/未签名"证据灯逻辑；表格、审计日志、交替行条纹（surface-1）正是数据密集场景的原生形态；"engineered, not stylized"一句话就是 Instrument-grade clarity。Plex Sans/Plex Mono 开源可用，等宽字体恰好承载哈希、寄存器地址、版本号。
- **风险提示**：源自 ibm.com 营销面的提取，营销 hero 类模式需要删除；落地前建议与 Carbon 官方文档（carbondesignsystem.com）核对关键 token。

#### 🥈 T1-2 Vercel（`design-md/vercel/DESIGN.md`，41KB，全库最大）— 精密次选
- **设计要点**：近白 #fafafa 页面底 + 纯白卡片 + 墨黑 #171717；**200 级灰阶**让每条分隔线/边框/禁用态都有独立色阶；Geist Sans + Geist Mono 双字体（mono 承载终端/代码/文件名标签）；"堆叠式极微阴影"（三层 4-12% 黑偏移 + inset 边框）代替重投影；标题负字距、mono 眉题标签；表格行/卡片边界全部 1px #ebebeb 发丝线。
- **匹配理由**：灰阶工程化程度全库第一，"给工程师看的部署平台"与固件发布工作台是同一物种；mono 标签系统适合校验值/时间戳/构建号。**唯一冲突点**：品牌 mesh 渐变（青-品红-紫 #ff0080/#7928ca）——必须剥离。剥离后剩余系统与红线完全兼容。
- **引用建议**：作为"灰阶纪律与表格系统"的参考实现，与 IBM 基准互补。

#### 🥉 T1-3 WIRED（`design-md/wired/DESIGN.md`，23KB）— 编辑部黑马
- **设计要点**：纸白画布 + 严格黑白二重奏，唯一彩色是正文链接墨蓝 #057dbc；三字体体系（高对比衬线展示 / 人文衬线正文 / 无衬线标注）；**全站 0px 方角按钮**；无投影、靠 1px hairline 分隔；"报纸级密度"——大特写卡 + 双列次级 + 署名行垂直堆叠，全部发丝线分隔；唯一装饰是报头黑色细带。
- **匹配理由**："paper-white broadsheet density" 几乎是 "dense not crowded" 的字面来源；编辑部的**证据排版逻辑**（署名、时间戳、来源引用、版本记录）与固件发布的 evidence-first（谁在何时签发了哪个构建）高度同构；方角 + 发丝线 + 黑白纪律完全合规。适合作为"审计日志/发布历史页"的密度参照。
- **风险提示**：三个专有字体不可商用，需替换为开源等价物（如 Source Serif/IBM Plex Serif + Inter）；编辑气质与"仪器感"有温差，建议只取其密度与栅格，不取其衬线个性。

#### T1-4 Cal.com（`design-md/cal/DESIGN.md`，31KB）— 务实稳妥项
- **设计要点**：白底 + 黑 CTA(#111111) + 浅灰 #f5f5f5 卡片；Cal Sans + Inter 双字体、8px 圆角矩形按钮（非 pill）；签名手法是**把真实产品 UI 碎片（日历组件、表单、集成网格）直接嵌入营销卡片**；页脚翻转为 #101010 是全站唯一暗面。
- **匹配理由**：气质原文"confidently engineered without trying to impress"——克制且证据优先（展示真实 UI 即展示证据）；技术栈（Inter 系）团队可直接复用，无字体许可风险；各模式映射到工作台组件的心智成本最低。作为第一梯队备胎，若团队觉得 IBM 直角过于"硬"，Cal.com 是降级最优解。

### 第二梯队：整库不作基准，局部模式值得移植

| 条目 | 可移植的局部模式 | 为何只作局部 |
|---|---|---|
| **Apple** | 字体纪律（负字距阶梯）、"全系统仅一处阴影"的克制规则、单一交互蓝 | 营销密度极低 + 摄影驱动，与 dense 冲突 |
| **Mintlify** | 3 栏文档布局（侧栏/正文/TOC）、14px Inter 长文密度、Geist Mono 代码块 | 天空/青薄荷渐变 hero 触碰红线，需剥离 |
| **Uber** | 黑白二元纪律、"engineering-grade"字型配对、无第三色的极致收敛 | pill 几何签名与仪器感相悖 |
| **HashiCorp** | **"每模块专属 accent 色编码"**（一眼区分产品段）——可映射为 FirmwareSight 的 bootloader/runtime/debugger 模块色标 | 黑底画布为主，不符浅色要求 |
| **ClickHouse** | 大数字统计 callout、暗色代码窗口、状态语义色（绿/红/蓝）的用法 | 黑画布"无浅色模式"（原文），可作暗色辅助主题参考 |
| **Coinbase** | 机构级沉稳感、weight-400 显示字、涨跌语义色"只作文字色不作底色" | 胶囊几何 + 96px 编辑节奏密度偏低 |
| **HP** | 目录式实用主义、纯白 + 单蓝信号色 | 消费目录气质 + 装饰性 V 形纹样 |
| **Ollama** | "文档即设计"的 README 美学 | SF Pro Rounded + 全 pill 几何偏圆润 |

### 明确排除（踩红线）与理由速查
- **紫蓝渐变/紫 accent**：Linear、Stripe、Superhuman、Together AI、Notion、Mistral、Slack、Kraken、The Verge
- **暗色画布为主**（不符"浅色为主"）：SpaceX、Shopify、Sentry、HashiCorp、ClickHouse、MongoDB、Binance、Raycast、Warp、Cursor、Expo、Supabase、Resend、PostHog、NVIDIA、BMW/Bugatti/Ferrari/Lamborghini、Runway、VoltAgent、OpenCode、Minimax、Composio、Spotify、xAI*
- **氛围渐变/玻璃拟态倾向**：ElevenLabs（粉彩渐变光球）、Stripe（渐变 mesh）、Mintlify（渐变 hero）、Clay、Renault（极光渐变）、Revolut（渐变卡片）
- **霓虹/赛博朋克气质**：Sentry（青柠+粉红+星空贴纸）、Shopify（霓虹绿）、Binance（交易大厅紧迫感）、NVIDIA（绿黑能量）
- **玩趣/消费向**：Zapier、Lovable、Wise、Starbucks、Airbnb、Miro、Intercom、Pinterest、Figma、Airtable
- **复古怀旧**：Dell-1996、Nintendo-2001（与仪器感无关，仅作彩蛋）

*\*xAI 的 stark 单色气质本身接近，但其文档以暗色叙事为主且信息量少（21KB），不如 IBM/Vercel 直接。*

### Agent 结论
> IBM/DESIGN.md 与 FirmwareSight 的哲学重合度接近 100%——它本质上就是"仪器面板"的数字化规范。**建议以 IBM 为单一基准（single source of truth），Vercel 供灰阶与表格细节，WIRED 供审计页密度，Cal.com 作圆润度备选**。若团队追求差异化，差异化应来自信息架构（证据链如何呈现：构建→签名→测试→发布），而非视觉噱头——这正是该仓库各条目证明的规律：最强的品牌都把装饰预算压到了最低。

---

## ④ 团队如何引用风格 md 作为 UI 基准文件

### 标准用法（仓库官方主张）
1. 把某个品牌的 `DESIGN.md` 复制进项目根目录；
2. 告诉 AI 编码代理"use DESIGN.md for all UI"——AGENTS.md 与 DESIGN.md 并排：**AGENTS.md 定义"怎么构建"，DESIGN.md 定义"长什么样"**。Claude Code、Cursor、Google Stitch 等均可直接消费。

### 推荐给 FirmwareSight 的落地流程（五步）
**Step 1 · 锚定基准源**
```bash
# 从仓库 raw 下载 IBM 基准（建议 pin 到具体 commit 保证可复现）
curl -o DESIGN.md https://raw.githubusercontent.com/VoltAgent/awesome-design-md/<commit>/design-md/ibm/DESIGN.md
```
**Step 2 · 改造为 `FirmwareSight DESIGN.md`**（放仓库根目录或 `docs/design/`）
- 保留：全部 Color token（语义名 + hex）、Typography 阶梯表、Component 状态矩阵、Do's and Don'ts；
- 删除：营销页模式（hero band、logo marquee、CTA banner 节奏等 ibm.com 特有的页面章节）；
- 映射：stat callout → 版本健康度指标卡；feature card grid → 构建批次卡；alternating-row stripes → 发布审计表；
- **追加"FirmwareSight Overrides"一节**（这是关键增量）：
  - 硬约束：禁止渐变/玻璃拟态/霓虹/紫蓝系；浅色主题为默认，暗色仅作可切换辅助主题；
  - 密度条款：表格行高、信息卡内边距上限（dense not crowded 量化成数字，如行高 ≤ 40px）；
  - 等宽字体条款：哈希/签名/寄存器/时间戳一律 mono 字体呈现（向 Vercel 的 Geist Mono 或 IBM 自带 Plex Mono 看齐）；
  - 证据优先条款：任何状态断言（签名有效/测试通过）必须携带可追溯来源链接或元数据展示。
**Step 3 · 在 AGENTS.md 中声明**
```markdown
## UI Rules
- All UI must follow the root DESIGN.md (FirmwareSight baseline, forked from
  VoltAgent/awesome-design-md ibm/DESIGN.md @ <commit-hash>).
- Conflicts between DESIGN.md and this file: DESIGN.md wins on visuals.
- Do NOT use gradients, glassmorphism, neon or purple-blue palettes.
```
**Step 4 · 纳入研发流程**
- 编码阶段：AI 代理生成页面前自动读取根目录 DESIGN.md（Cursor 的 Rules / Claude Code 的 CLAUDE.md 均可指向它）；
- 评审阶段：把 DESIGN.md 的 Do's and Don'ts 当作设计 review checklist 逐条对照；
- 新人阶段：DESIGN.md 即 onboarding 文档，比口口相传的"感觉"可复现得多。

**Step 5 · 版本治理**
- 在仓库内 pin 上游 commit hash；上游 2026-09-21 仍在活跃更新，建议每季度 diff 一次上游变更（用 `git diff` 对比 raw 文件即可）；
- MIT 许可允许修改与内部再分发，无法律风险；但注意文档为**逆向提取公开网站**的产物，非 IBM 官方发布物，关键 token 落地前与 Carbon 官方文档复核一次。

### 备选组合方案（如团队想混合）
- **A 方案（纯 Carbon 路线）**：基准 = IBM 全文 + FirmwareSight Overrides。最稳，与开源 Carbon 组件库（React）直接兼容，甚至可逐步替换为真实组件库。
- **B 方案（灰阶精密路线）**：基准 = Vercel（剥离渐变节）+ WIRED 的表格密度条款。观感更"当代工具"，但需要自己做剥离手术。
- **C 方案（温和路线）**：基准 = Cal.com 全文 + 8px 圆角改 2px。改动最小，上手最快。

---

## 原始信源
1. GitHub API `repos/VoltAgent/awesome-design-md`（stars/forks/时间线/license）— 抓取于 2026-09-27
2. Git Trees API（74 份 DESIGN.md 文件清单与体积核验）— 同上
3. `raw.githubusercontent.com/VoltAgent/awesome-design-md/main/` 下 README.md、CONTRIBUTING.md 及全部 74 份 DESIGN.md 全文（本地已完整存档于本 agent 工作区 `awesomedesignmd/styles/`）
4. 条目一句话特征优先采用仓库 README 原文表述（忠实转译），候选条目特征基于 DESIGN.md 原文 Overview/Colors 章节逐份取证
