# FirmwareSight NEXT-46 整合交付报告

状态：**NEXT46_BLOCKED_BY_EVIDENCE_OR_ARCHITECTURE / NO_PUSH**。工程实现和本地验证已交付；精确 SHA 的独立审查因必需原生交互证据缺失而拒绝完全验收。推送次数为零。

## 版本与授权

- Owner 实际调度接受 NEXT-46，包含 `OWNER_NEXT46_CONDITIONAL_FF_PUSH=YES`；仅在最终精确 SHA 所有适用必需项独立完全通过后可执行一次普通 FF。
- 基线：`4ac1e4f709396fac7a37a1d92b8095a070b3effa`。本地候选：`76b5c2c006f1ca0850af86da6d1e4b957227a025`；tree：`55ecc38ae7e45976e60845249dee8830d35e054d`。分支 `codex/next46-export-toolchains`，候选工作树干净。
- 原 main 保持 `e34f9e69e515a330a3705470aeba6cfab073634d`、9 个 staged 文件；patch SHA-256 `05f86c0313040899dca1156f538186d686377906e1a62e755fcb029c1eb96d2b`。独立处置说明见 [PROTECTED_MAIN_DISPOSITION.md](PROTECTED_MAIN_DISPOSITION.md)。

## 实现与兼容性证据

1. CLI 可视化和旧详细 HTML 拒绝共用可信输出边界，保留 exit 6、ERR-EXPORT-6001、operation ID、错误类别与恢复建议；不打印目标路径、原始 OS 错误或注入 config。CLI tracing 仅允许 FirmwareSight operation 事件，字体库路径诊断无法绕过边界。两种输出模式、Analyze/Diff、四种导出入口与多类失败/路径 canary 已测试。
2. 零或小到取整为零的前景条省略，整数 bytes 原样保留；没有最小 1px 伪面积，没有虚构百分比。SVG 和 PNG 边界回归通过。
3. Analyze、Compare、Gate 的异步导出结果绑定 selection/snapshot、diff 或 run/snapshot。旧消息不能出现在新结果下；三项 delayed-response 测试先红后绿。
4. 新增四份真实 Clang 22.1.8 relocatable ELF objects，ARM32/AArch64 × LE/BE。raw header、独立 llvm-readobj、file offsets、.data=7/4 bytes、.bss=4 memory bytes/0 payload、symbols 与 parser 一致；重新编译 4/4 字节一致。只证明 basic metadata，不证明 linked memory budgets，不是 Keil/IAR 产物。
5. 明确提供外国 MAP 时返回 ERR-MAP-3001，长头部/混合 GNU 文本不会误入 GNU parser。去掉 MAP 可继续兼容 ELF-only 分析；UTF-16/坏字节/Unicode 路径/截断/size guard 测试通过。
6. Arm armlink 与 IAR ILINK 语义研究采用厂商官方资料；公开 Mbed test examples 的固定 commit、原文与 hash 存在 external-map-research/。其 generic paths、删节、未配对 ELF、压缩/初始化信息不完整，不足以宣称真实厂商 cohort。两类 MAP 保持 NOT_SUPPORTED / FIXTURE_BLOCKED，提供独立 grammar/admission 设计和拒绝 harness，没有伪造 parser pass。
7. 当前导航与 NEXT-36..46 简明台账更新；历史快照、Reviewer 原话和旧 commit message 保留。NEXT-45 为 Owner-declared ENGINEERING_DELIVERED / REMOTE_CI_PROVEN / REVIEW_EXCEPTIONS_OPEN。过去 ACCEPT_WITH_LIMITS 后推送作为 governance nonconformance 记录。

## 实际验证

| 检查 | 结果 | 原始证据 |
|---|---|---|
| Executor 完整 gate | 18/18，0 SKIP，exit 0 | full-gate-accepted-tree.log |
| 独立 Reviewer 完整 gate | 20/20，0 SKIP，exit 0；新增两步为干净工作树的前端资产 install/build 准备 | reviewer/full-gate.log、reviewer/full-gate.json |
| 单次 cargo test --workspace | 1,160 pass / 0 fail / 2 原有 opt-in ignored / 56 result lines | 同一日志首个 rust/test 区段；不重复累计 drift/Core smoke |
| 前端 | 397 tests / 13 files；install/typecheck/lint/build 通过 | 同一 gate 日志 |
| Core smoke | 3/3 | core-smoke.log |
| 在线 cargo-deny | licenses/bans/sources/advisories 四类均 exit 0 | deny-results.json、deny-*-final.log |
| 许可证负控 | 删除例外/错包名/错版本均实际 arrayref 拒绝，exit 4 | deny-negative-*-correct.log |
| ADR-0029 | commit 前后均 PASS；894 tracked/892 entries，0 mismatch | precommit-baseline.log、postcommit-baseline.log |
| 精确 SHA mutation | 隐私投影旁路与 Gate 状态篡改均被命名测试杀死；外部副本逐字节恢复 | mutation-results.json、mutation-*.log |
| 冻结边界 | 13 组路径无变化；既有 goldens 全通过且字节不动 | frozen-boundaries.json、完整 gate |
| PNG/XML/静态输出 | Pillow 与 XML 独立检查六组实际 cohort，通过 | independent-images.json |
| 浏览器打印 | 六组 summary 每组 2 页 A4；详细 Compare 16 页，Release 12 页，末页非空 | browser-evidence.json、print-evidence.json、detailed-print-evidence.json |
| 本地 Windows CLI release bytes | 7,221,248 → 7,222,784（+1,536） | build-impact-corrected.json；不是安装包大小/性能提升声明 |

旧 NEXT-45 单次 Windows workspace 正确总数为 1,154/0/2、54 result lines；1,448/68 旧说法包含重叠运行，未修改历史 Git message。计数补证见 audit-inventory.md 和 ac-inventory.json。

## 可复现产物

六组真实产物及 hash 位于 samples/ 与 independent-images.json：Analyze known、Compare ranked growth、Compare Unknown、Gate REVIEW/BLOCK/UNKNOWN、CJK/长标签/privacy、large u64/region。每组 PNG 为 1600×900；SVG 是真正自包含 vector；HTML 自包含并可在浏览器 Print → Save as PDF。独立 Reviewer 重新生成两次，六组在同一主机字节一致；不宣称跨主机字体栅格化字节一致。Reviewer 还核对所有打印 table-cell 文本、全部详细分页 contact sheets 与非空末页。没有原生应用 PDF 功能声明。独立 `Pillow`/`pypdf`/`Poppler`/Edge 已存在，没有安装新浏览器/字体/商业 IDE。

- [Analyze PNG](samples/analyze-known/analyze-known.png) / [SVG](samples/analyze-known/analyze-known.svg) / [HTML](samples/analyze-known/analyze-known.html)
- [Compare PNG](samples/compare-ranked-growth/compare-ranked-growth.png) / [HTML](samples/compare-ranked-growth/compare-ranked-growth.html)
- [Gate/Release PNG](samples/release-review-block-unknown/release-review-block-unknown.png) / [HTML](samples/release-review-block-unknown/release-review-block-unknown.html)
- [浏览器生成 A4 PDF](samples/release-review-block-unknown/browser-a4.pdf)（测试打印证据，非 native PDF）。

## 历史来源与边界

完整原 NEXT-44 40 项、NEXT-45 30 项及 NEXT-46 36 项源文本/hash 都已取得。真实 NEXT-44 license blocker Reviewer 最终原话与 thread/message 来源已恢复；NEXT-45 提交版完整 Reviewer artifact 未恢复，仅保留 Owner 明确声明的 ACCEPT_WITH_LIMITS，不伪造其原文/验收矩阵。

原生 Windows Save/overwrite/dialog、keyboard/screenreader、1024×720/light-dark 交互观察未执行。当前 CUA runtime 不提供 native Windows surface，也没有在本轮建立能够保证不触碰 Owner store 的隔离桌面运行环境。Rust Session/writer/DOM 回归与 native GUI 观察严格分开。Reviewer 另确认既有 Rust native dialog title/custom confirmation 为英文 literal；React 本地化不构成 native dialog 本地化证明，详见 reviewer/native-evidence-boundary.md。没有启动既有桌面进程，没有打开或修改 Owner 的 FirmwareSight SQLite/AppData/settings、私有 firmware 或 V1 archive；没有 capability/schema/identity/license/signing/updater/network 权限变化。

## 独立复审、推送与 CI

独立 Reviewer 使用不同上下文、干净 detached worktree 和独立构建目录，逐项完成原 NEXT-44 40 项、NEXT-45 30 项、本轮 36 项，共 **106 项**。独立结论为 **BLOCKED_INSUFFICIENT_EVIDENCE / NO_PUSH**，未发现可行动的源码缺陷；这是证据阻断，不能转写成完全通过或静默豁免。

- [独立审查原文](reviewer/REVIEW_VERDICT.md)
- [机器可读 106 项矩阵](reviewer/AC_MATRIX_106.json) / [TSV](reviewer/AC_MATRIX_106.tsv)
- 96 PASS、3 NOT_VERIFIED、1 FAIL（验收前提）、3 CONDITIONAL N/A、3 NOT_RUN（未推送的远端 CI 条件项）。

| 必需项 | 独立结果 | 尚缺证据 |
|---|---|---|
| NEXT44 AC24 | NOT_VERIFIED | 实际原生 Save/取消、覆盖拒绝、写入失败、stale 与 retry 流程观察；已有 Rust 注入/DOM 测试不能替代 |
| NEXT44 AC26 | NOT_VERIFIED | 实际原生键盘/focus/accessibility、Light/Dark 与最小宽度交互观察 |
| NEXT45 AC20 | NOT_VERIFIED | 本地化的实际 OS Save/overwrite 交互观察；typed IPC/source/test 已通过 |
| NEXT46 AC34 | FAIL，衍生前提未满足 | 所有适用必需项无条件完全通过这一前提不成立；并非另一个代码失败 |

NEXT-46 F2/F3 和 Owner 的实际调度均规定 MUST 未验证必须停在本地，所以没有执行 push。Owner 的 YES 开关真实存在，但其条件未满足，不是缺少授权开关。push attempts=0；candidate remote CI=NOT_RUN。最终 `git ls-remote` 与 GitHub REST 双读仍为 `4ac1e4f709396fac7a37a1d92b8095a070b3effa`，main 九个 staged 文件的 paths/OIDs/modes/status、19,794-byte patch 全部精确不变，所有既有工作树 HEAD/status 也不变；见 final-state.json、protected-main-after.json 和 reviewer/git-after.json。

基线 Actions #112（run 38535378887）的 10 Jobs + 10 Checks 均 completed/success，完整 JSON/log 留存；它只证明基线，不证明本候选。没有 tag、公开 Release、安装包或签名产物发布。后续关闭证据缺口必须使用隔离、不访问 Owner store 的原生环境，补足上述三项观察，并由独立 Reviewer 对最终精确 SHA 重新无条件验收；代码修复形成新 SHA 时重跑相关 gate 和审查。

## 测量与局部修复审计

所有失败/更正日志保留，见 measurement-corrections.md。尤其旧 target 共用导致 Cargo 重用 baseline release binary 的候选 size 行已 INVALID；实际候选改用独立 fresh target 重建并通过 --help identity control。错误 cargo-deny 参数导致的 usage failure 未计入 negative policy proof；已用正确 global --config 重跑。

## 命令、交付包与后续兼容性限制

Executor 在 `LOCAL_USER/.codex/worktrees/next46-export-toolchains/FirmwareSight` 执行，Reviewer 在 `LOCAL_USER/.codex/worktrees/next46-final-review/FirmwareSight` 执行。下列命令实际 exit 0；完整展开 argv/cwd/exit/log hash 在 gate 日志与 reviewer/*.json，包含全部 18/20 个步骤，不以摘要替代原日志。

```text
python scripts/check.py
python scripts/check.py --only core-smoke
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo deny check licenses bans sources advisories
cargo deny check licenses
cargo deny check bans
cargo deny check sources
cargo deny check advisories
python scripts/verify_baseline_artifacts.py
corepack pnpm install --frozen-lockfile
corepack pnpm typecheck
corepack pnpm lint
corepack pnpm test
corepack pnpm build
cargo build --release -p fwsight
```

前端命令 cwd 为 `apps/desktop/ui`，其他命令 cwd 为各自候选根。release CLI 比较使用 `CARGO_INCREMENTAL=0` 与分离 target；三个负许可证配置调用 `cargo deny --config <external-policy.toml> check licenses` 实际 exit 4。两种外部 disposable mutation 实际命名断言失败 exit 101，并逐字节恢复；usage/compiler 调用错误未计入有效负控。CLI 失败矩阵实际 exit 6，foreign MAP 实际 exit 3。Windows 桌面 installer/package size **NOT_MEASURED**；7,222,784 bytes 只对应 Windows CLI release exe。

精确源码包：`../FirmwareSight_NEXT46_SOURCE_76b5c2c.zip`。`git archive` ZIP comment 绑定完整候选 SHA，894 个 tracked blob 全部与 Git tree 对比一致，CRC 通过；见 source-delivery-proof.json。证据包：`../FirmwareSight_NEXT46_EVIDENCE_20261014.zip`，包含整合报告、106 行矩阵、审查原文、日志、fixture/readout provenance、六组输出与打印证据；排除 target、构建副本与 exe。这两份 ZIP 为本地私有交付，不是公开发布。包文件 hash/size 见 `../FirmwareSight_NEXT46_DELIVERY_MANIFEST.json`。

后续兼容性限制保持明确：GNU ld 为既有测试支持；真实 Clang ARM/AArch64 LE/BE 四类对象验证 basic metadata。Keil/Arm compiler cohort 与 IAR compiler cohort 均 **UNVERIFIED**；ArmLink MAP 和 IAR ILINK MAP 均 **NOT_SUPPORTED / FIXTURE_BLOCKED**。下一步需要可追溯版本、许可可再分发的真实同源 ELF/MAP 配对及独立工具 readout，才能验证 region/VMA/LMA、初始化/压缩数据及 attribution 语义并启用 adapter。重命名 Clang 为 .axf/.out 和删节 Mbed MAP 不能升级这些结论。
