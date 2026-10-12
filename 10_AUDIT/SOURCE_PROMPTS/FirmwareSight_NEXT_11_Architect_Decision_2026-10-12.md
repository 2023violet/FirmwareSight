# FirmwareSight｜NEXT-11 架构师裁决（2026-10-12）

## 身份与证据边界

NEXT-11 由 Coding Agent 在本地 HEAD `a21a987db3233499c7b7109182e6b7946754c3e9` 上执行并报告独立复测；执行日志和 `target/n11/n11_report.md` 留在本机，本评审不能远程读取未推送提交或该本地报告。GitHub `origin/main` 独立核对为 `a0b5c886051a6b10042ea26f52809baea713d7af`（CI #101 成功）；NEXT-11 报告中的 1,068 Rust、380 UI、17/17 是本地实测声明，而非 B2-U1R1 远端 CI。

## 裁决

- **NEXT-10 R1/R2：ACCEPT_WITH_EXPLICIT_LIMITS。** 接受 NEXT-11 以独立目录二进制重建后取得的「parent 哨兵同名 fail-open / head 在 stat 之前拒绝」对照。原第一次共享 target/debug 的测量已作废，不采用；新提交待正式独立源码和远端 CI 验证。
- **D-N10-1：必须修复。** 已存在于远端的 Release Notes 词法路径合法集与 join 规则冲突，可能将无关文件内容绑定进 Gate 和 Bundle。保留合法 `docs/../x`，统一由 `Result` 解析，拒绝不可伪装为可打开的路径；不改 Gate canonical grammar、旧存储、schema 或出错码。
- **D-N11-1：独立的物理 containment 安全合同问题。** 对 `release_notes_path`，架构建议的安全目标是项目配置相对文件应真实对应项目内来源；但把 junction/symlink 的外部目标直接拒绝会改变现行行为及可能影响既有 run 身份，因此须另行明确 Owner 的兼容性与安全授权，并证明 Windows/POSIX、retarget race 的真实保证。一次 canonicalize + starts_with 不能被误报为严格 TOCTOU 安全。
- **明确保留 C1 已作裁决的附件路径能力。** `04_TECH/28` §14 已接受用户明确选择的源 symlink 可以跟随，绝对路径附件可以位于项目外，并有对应 M11 用例；不能由 D-N11-1 顺手禁止它们。不要把「source-side symlink」与「bundle 内不得包含 symlink」混为一谈。
- **Python tests：同意升级到正式门禁，要求非空测试收集、负面突变和真实的计数读回。**

## 下一阶段决策建议

正式授权 NEXT-12 v1.1（仅 D-N10-1 + Python 门禁 + D-N11-1 事实披露），完成本地小范围代码修正。然后单独决定是否实施「Release Notes 物理 root-contained」安全加强；在此之前不得宣布 D-N11-1 已解决，也不得把附件的明确选择能力当作风险修补的牺牲品。

修复完成后，先验收最终 HEAD，再由 Owner 单独批准 fast-forward push，读取新十项 CI；B2-U2 尚未授权。
