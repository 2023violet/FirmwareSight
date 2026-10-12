# FirmwareSight｜B1-U1 / B1-U2 阶段架构裁决（2026-10-11）

## 已独立复核（远端）
- HEAD: `2cff7026f2e2953411b08008b8834424cbf0e9c0`。
- Actions #99 `38124118299`: 10/10 jobs SUCCESS，Rust Windows `1043` / Ubuntu `1044`，UI Windows/Ubuntu `319`（10 files），GitHub 工作日志已直接读取。此仅覆盖 B1-U1。
- B1-U1 独立评审代理还报告了真实 WebView2/CDP 开发态点击/重启/重置与错误恢复，且修复了文案连接错误 `2cff702`。这部分为用户提供的执行日志而非本评审现场重跑。

## B1-U1 决定
`ACCEPTED_WITH_DECLARED_SCOPE`：功能验收和远端回归证据足以关闭 B1-U1；开发态运行不是 CI 安装包真实安装态验收。

## B1-U2 本地报告（尚不能远端复核）
- 提交顺序 `bf3d494` ADR-0031 → `bd62289` 功能 → `e0b8fc4` 文档修正，均本地，仅 3 ahead，未 push。
- 报告：`17/17`，Rust `1050`、UI `334`、command `37`、verifier `820/818/949`；并报告 OS 实际主题翻转、125/150% 缩放、forced-colors 和安装态 NOT_VERIFIED。
- 源码未在远端，不能出具独立源码验证结论；阶段状态 `LOCAL_B1_U2_GATE_PASS / ARCHITECT_REVIEW_PENDING`。

## 已知必须核查的项目
1. Light diff.removed on diff.removed.bg 比率 `4.4974:1`。若是普通文字，则低于 WCAG AA 4.5 门槛，不能四舍五入，也不能以“冻结 golden”代替例外；必须窄范围处理，必要时提出授权后更新 token 和受影响的 golden。
2. 系统主题真实切换与 UI 响应区别于在 WebView 中模拟 media query；真实窗口可验证的项目要如实区分。
3. System/Light/Dark 重启持久化、快速切换、旧 schema 迁移、Reset、未来版本拒写以及 Gate/release data isolation 必须独立复核。
4. ADR-0018 的 Light-only 条款是 **D3**，不是 D4；新增 ADR-0031 不得重写旧 ADR 历史。

## 授权边界
- 以前批准的 push 覆盖 B1-U1 相关提交；不覆盖本地 B1-U2 的三个提交。
- NEXT-05 仅独立审查，不 push、不修改 tracked 文件。
- NEXT-06 只有在 B1-U2 独立验收、Owner 独立 push 许可、对应新 head 的远端 CI 通过、Owner 明确发出 B1-U3 开发授权时才能执行。
- B1-U3 已完成的只读预检不是代码开发授权；i18next vs typed catalog 必须按实际维护/测试/依赖成本决定，不能直接“零依赖优先”或“必须 i18next”。

## 期限提醒
旧 V1 cohort 冻结构建的私有备份期限仍为 2026-10-22，单独处理；不得借当前 B1 升级替换 V1 冻结制品。
