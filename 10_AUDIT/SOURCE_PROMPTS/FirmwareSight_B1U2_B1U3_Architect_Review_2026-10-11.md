# FirmwareSight｜B1-U2 / B1-U3 独立评审与推进裁决

日期：2026-10-11｜身份：Architect review（按已获取证据分级）｜不构成 push 或新开发授权

## 1. 已独立验证的远端事实

- GitHub `2023violet/FirmwareSight` 当前远端 `main=e0b8fc4687aefadfc76c2e64bfbdf6ab6389d2dc`。
- GitHub Actions #100，run `38160046797`，对应 `e0b8fc4`，十项 jobs 全部 `success`。直接读取 CI job logs：Rust Windows **1050**（49 条结果）、Ubuntu **1051**（49 条结果），UI Windows **334**、Ubuntu **334**。
- 仓库 `09_ADR/ADR-0031-developer-preview-theme.md` 已接受三态主题决定，只定向 supersede ADR-0018 **D3** 的 MVP Light-only 句子；其余设计规则保留。
- `B1_VALIDATION/B1_U2_THEME_REPORT.md` 明确区分 DEV_RUNTIME_VERIFIED 与 INSTALLED_VERIFIED，也披露浅色 `diff.removed`/`diff.removed.bg`=**4.4974:1** 的历史 AA 普通文字对比度不达标及相关报告 Golden 牵连。
- `a0b5c88`、`9ccd3f0`、`40ce0b6` 尚不可从远端 GitHub 取到，故 B1-U3 源码和完整测试无法由本评审从远端独立复测。

## 2. NEXT-05 / B1-U2 裁决

**B1-U2：ACCEPT_WITH_EXPLICIT_LIMITS（就已测工程行为而言）**。三态主题、偏好与隔离的远端代码已经通过十项 CI，相关本地真实窗口证据由执行报告提供。**不接受“所有 WCAG AA 测试通过”或“已安装验证”之类扩大结论。** 未验证的真实 OS 主题翻转、125%/150% DPI、forced-colors、B1 版 CI 安装包运行，须按范围保留 NOT_VERIFIED。

对比度：4.4974 < 4.5，不允许四舍五入通过。可在 Owner 明示授权后，采取最小颜色修正（候选 `#FBECEA→#FCEEEC`）并有意更新两个受影响 HTML 报告 Golden。提交前须有报告字节差异和合同测试证据；不能在其他主题或翻译任务中顺手更改 Goldens。具体 NEXT-05 的 C1/C2/C3 判词只有执行日志中的简述，未收到独立 NEXT-05 全文，不代替它做超出证据的裁决。

## 3. NEXT-06 / B1-U3 裁决

执行者报告：本地三个提交 `9ccd3f0`（67 paths）+ docs-only `40ce0b6` / `a0b5c88`；工作树 clean；`origin/main=e0b8fc4`；本地 gate 17/17 exit 0，Rust 1056，UI 379，schemas/goldens/身份不随语言变化；schema v2→v3 先兼容 1/2/3；9 namespaces × 918 keys，类型安全 Catalog；七个页面开发态真窗口走查，中文/英文 Gate ID、digests、五态计数不变。上述均**暂为执行者的本地报告**。

**B1-U3：LOCAL_GATE_PASS_REPORTED / ARCHITECT_INDEPENDENT_REVIEW_PENDING / REMOTE_CI_PENDING / INSTALLED_NOT_VERIFIED**。

独立验收应重点补证：
1. 三提交范围、未推送状态、绿色最终 staged-tree 而非历史中途失败；设计与技术变更有没有超出语言范围。
2. Preferences v1/v2→v3 的逐字节只读与首次写入、损坏文件/未来版本保护、连续写、Reset；不会修改 project policy。
3. 918 keys 是否确有双语同构、插值和遗漏检查；**Key 全≠翻译正确**。抽查专业文案、共同组件、errors、非 TSX 的 `format.ts`/`evidenceBasis.ts` 等呈现层，以及 Help、History、Release 附件字段。技术常量、文件名、符号名保留英文是正常行为，禁止把它们视为未翻译的 UI 文案。
4. system 语言识别、zh-HK/zh-TW/非支持语言回退、真正跨重启；真实窗口 1024×721/1440×901 与要求的 1024×720/1440×900 不应说成像素完全相同。缩放、高对比度和 CI 安装件另标 NOT_VERIFIED。
5. 中文和英文下的 Gate & Bundle 身份、SHA256SUMS、portable docs、CLI JSON、59 个 ErrorCode 均不漂移；确认绝对宿主路径没有进入 UI 错误和 portable 包。
6. 执行者明确承认真实开发 SQLite store 新增一条 Gate run 记录，应记录为副作用；下一次验收不得直接清理或改写 Owner store。配置目录不支持环境变量重定向时，应换沙箱用户/独立隔离环境，而非假称隔离。

若核验全部满足，建议 Architect 判 `ACCEPT_WITH_EXPLICIT_LIMITS`（至少安装态尚未证实）；提交/CI 的状态必须另列。未经 Owner 单独授权，不推送 `a0b5c88`，不自行开始 B2 开发。

## 4. 下一开发块建议

B2-U1 = **File Workflow Correctness & Release Usability**，先收敛 NEXT-01 的 N01-F01～N01-F04（相对路径语义/诊断、非 ASCII 叶名展示、无 diff 的便携验证器、CLI/桌面状态码解释），只针对可复现痛点，不碰解析器与 Gate 身份；对潜在 CLI 路径兼容改动设置 STOP 及独立决策。完成后进入 B2-U2 的 ELF/MAP 内存口径与 Evidence 解释专项（需要真实 Fixture，不可先猜内存结论）。

V1 frozen GitHub Artifact #11573661113 预期于 2026-10-22 到期，务必完成独立私有归档，不要修改冻住的 V1 cohort 身份。
