# FirmwareSight NEXT-47 整合复审与工程集成报告

**NEXT47_ENGINEERING_INTEGRATED_REMOTE_CI_PROVEN / NATIVE_GUI_QA_DEFERRED / READY_FOR_ARCHITECT_CLOSE**

最终 SHA：`76b5c2c006f1ca0850af86da6d1e4b957227a025`。父提交：`4ac1e4f709396fac7a37a1d92b8095a070b3effa`；tree：`55ecc38ae7e45976e60845249dee8830d35e054d`。本轮新增源码提交 **0**，沿用原 NEXT46 候选；工程集成已完成，三项原生 QA 仍未验证。

## 独立双层判决与历史保留

**CODE_INTEGRATION_GATE = PASS**。独立 Reviewer `/root/next47_independent_review` 在新 clean detached worktree 中完成源代码、实际进程、组件、解析器、图片/分页与完整工程门禁复测，没有发现 S0/S1 或未受理的严重实际缺陷。

**NATIVE_USER_ACCEPTANCE_GATE = NOT_VERIFIED / BLOCKED_BY_ENV**。三项原始 MUST 状态没有改为 PASS；只新增 `OWNER_ACCEPTED_DEFERRED_NATIVE_QA` 集成属性。原 NEXT46 `BLOCKED_INSUFFICIENT_EVIDENCE / NO_PUSH`、原 NEXT46 AC34 `FAIL` 和原历史审查措辞全部保留。

本轮使用实际人类调度中的两个 YES 开关，不从合同示例推断授权；实际调度逐字转录、范围及 hash 在 OWNER_DISPATCH_VERBATIM.md、01_NEXT47_PRECHECK.json 和签署文件中。

- [独立工程复审原文](02_NEXT47_CODE_REVIEW.md)
- [独立签署 JSON](reviewer/signed-verdict.json)
- [原106项+新决定 TSV](03_NEXT47_AC_MATRIX_106.tsv) / [JSON](03_NEXT47_AC_MATRIX_106.json)
- [原字段保留审计](reviewer/original-field-comparison-audit.json)

原106分类仍为96 PASS /3 NOT_VERIFIED /1历史验收前提FAIL /3 CONDITIONAL N/A /3历史remoteCI NOT_RUN。新增工程列为96 PASS /3明确native递延 /7有依据CONDITIONAL N/A。JSON的1,378个原row值和所有旧顶层值、TSV的742个原单元格及七列前缀逐项一致；新旧门禁不混用。

## 三项递延 QA 风险与强制截止

| 原始项 | 原状态 | 真实未观察内容 |
|---|---|---|
| NEXT44 AC24 | MUST / NOT_VERIFIED | 原生Save、Cancel、拒绝/确认覆盖、write failure、stale selection和retry |
| NEXT44 AC26 | MUST / NOT_VERIFIED | 原生keyboard/focus/accessibility、System/Light/Dark及1024×720 |
| NEXT45 AC20 | MUST / NOT_VERIFIED | 中文/英文原生Save/overwrite提示与结果；现有Rust dialog title/custom confirmation英文literal明确为风险 |

[完整递延清单](04_NEXT47_DEFERRED_NATIVE_QA.md) / [机器可读清单](04_NEXT47_DEFERRED_NATIVE_QA.json) 每项保留原AC/Reviewer文字，列明自动化覆盖层、用户后果、潜在影响、可恢复性、隔离Windows环境、14步真实用例、精确构建/安装包/操作者/录像/输出hash/缺陷证据和解除条件。所有操作步骤都是未来计划，未摆拍或声称执行。

**必须在 Developer Preview 对外分发、Private Beta 或 RC 中最早发生的阶段之前全部实测关闭。** 工程CI/package/DOM/Edge打印都不能代替原生或安装态验收；重大问题必须修复并按新SHA重测。

## 实际本地工程验证

| 项目 | 最后成功运行的实测结果 |
|---|---|
| 完整 scripts/check.py | 18/18、0 SKIP、exit0 |
| Rust workspace单次区段 | 1,160 passed /0 failed /2原有opt-in ignored，56 result lines |
| 前端 | 397 tests /13 files；install/typecheck/lint/build通过 |
| 外部组件探针 | 2 passed；37复制旧案例被filter，不计入官方总数，不证明native键盘 |
| Core smoke | 3/3、exit0 |
| 在线 cargo-deny | licenses/bans/sources/advisories四类分别exit0，完整gate联合调用exit0 |
| ADR-0029 | 894 tracked /892 checksum entries，0 mismatch，RESULT PASS |
| 实际CLI | Analyze/Compare PNG/SVG/HTML、静态隐私失败边界、foreignMAP拒绝/ELF-only恢复、Gate/Release和新Bundle独立64/64验证 |
| 图像/分页 | 六组实际1600×900 PNG独立Pillow解码、SVG XML/static检查，重复生成同主机字节一致；A4 summaries2页、Compare16页、既有Release12页、新CLIRelease10页 |

真实命令、cwd、UTC时间、exit及log hash在05_NEXT47_TEST_GATE_LOGS/*.json及.log中；统计由最后一次完整成功运行提取，不累加Core/IPC重叠运行。源码894个真实文件在独立worktree中与index clean-filter OID逐项匹配；before/after源文件字节清单相同。

首轮clean gate为19/20、exit1：一个合成Bundle目录rename返回PermissionDenied。相同原二进制精确用例重跑3/3、完整100项套件和另一次Cargo100项套件均通过，最终完整18/18通过，期间未改源代码、测试、权限或加skip。原始Windows拒绝主体/errno未保留，原因Unknown，不推测是杀毒软件。失败、增量编译access-denied note和所有诊断重跑保留在 [verification-events](reviewer/verification-events.md)。

推送前完整字段闸门又捕获外部矩阵编码/TSV列格式错误，实际git push尚未调用。Reviewer保留两个未批准签署版本，显式UTF-8修复JSON复制乱码，并从历史TSV直接追加列；最终第三版重新签署且所有hash绑定核对。原历史材料与候选源码都未改。这些是已关闭的外部审计材料错误，不被隐藏成首轮成功；见 [artifact-correction](reviewer/artifact-correction.md) 和 PRE_PUSH_ARTIFACT_GUARD_ABORT.json。

## 唯一推送与远端CI实证

推送前现场双读main仍为精确父提交4ac1e4f；最终D1签署、21项前置条件、原106/TSV保留、风险/授权/hash以及原main九项保护全过，才调用下列一次普通非强制refspec：

```text
git push --porcelain origin 76b5c2c006f1ca0850af86da6d1e4b957227a025:refs/heads/main
```

实际exit0、push_attempts=1；没有force、盲重试、rebase、merge、stash、FirmwareSight tag、GitHub Release或签名。stdout、stderr与守卫证据在push-attempt.json、push.stdout、push.stderr和PRE_PUSH_GUARD.json。之后GitHub REST与ls-remote都证明main为上述精确SHA。

新 [Actions #113](https://github.com/2023violet/FirmwareSight/actions/runs/38595778036)：run `38595778036`、attempt `1`、event `push`、branch `main`、head SHA `76b5c2c006f1ca0850af86da6d1e4b957227a025`，全部 **10 Jobs +10 Check Runs completed/success**。不是旧基线#112的复用。

| Job | 结果 | 证据 |
|---|---|---|
| Generated output drift | completed / success | [job](https://github.com/2023violet/FirmwareSight/actions/runs/38595778036/job/115962008130) |
| Dependency policy | completed / success | [job](https://github.com/2023violet/FirmwareSight/actions/runs/38595778036/job/115962008279) |
| Desktop UI (windows-latest) | completed / success | [job](https://github.com/2023violet/FirmwareSight/actions/runs/38595778036/job/115962008402) |
| Package Windows | completed / success | [job](https://github.com/2023violet/FirmwareSight/actions/runs/38595778036/job/115962008461) |
| Rust (ubuntu-latest) | completed / success | [job](https://github.com/2023violet/FirmwareSight/actions/runs/38595778036/job/115962008471) |
| macOS Core Smoke | completed / success | [job](https://github.com/2023violet/FirmwareSight/actions/runs/38595778036/job/115962008485) |
| Desktop UI (ubuntu-latest) | completed / success | [job](https://github.com/2023violet/FirmwareSight/actions/runs/38595778036/job/115962008499) |
| Package macOS | completed / success | [job](https://github.com/2023violet/FirmwareSight/actions/runs/38595778036/job/115962008758) |
| Package Ubuntu | completed / success | [job](https://github.com/2023violet/FirmwareSight/actions/runs/38595778036/job/115962008892) |
| Rust (windows-latest) | completed / success | [job](https://github.com/2023violet/FirmwareSight/actions/runs/38595778036/job/115962008972) |

实际 Dependency policy 日志已读取，包含 `cargo deny check licenses bans sources advisories`、PASS及1/1；实际 Generated output drift 日志包含verify_baseline_artifacts.py、894/892、RESULT PASS和9/9。整套job日志、run/attempt/jobs/checks REST原响应、log hash均在07_NEXT47_REMOTE_CI/。

- [精确SHA远端CI证明](07_NEXT47_PUSH_CI_PROOF.json)
- [独立post-push复核](reviewer/POST_PUSH_CI_REVIEW.json) / [文字版](reviewer/POST_PUSH_CI_REVIEW.md)

CI package构建只证明对应runner可构建并验证unsigned产物。没有安装观察，没有签名/notarization或公开发行授权。

## Owner工作树、兼容声明与交付边界

原Owner main保持 `e34f9e69e515a330a3705470aeba6cfab073634d`；九个staged mode/OID和19,794-byte patch均精确不变，SHA256 `05f86c0313040899dca1156f538186d686377906e1a62e755fcb029c1eb96d2b`。额外未跟踪的Playwright console日志如实列出，未读内容、未删除，不能宣称与旧NEXT46整个raw status完全相同；在相同status选项下本轮开始/结束状态一致。详见 [保护回读](06_NEXT47_OWNER_PROTECTION_READBACK.json)。

没有打开Owner的FirmwareSight生产AppData/store、私有firmware、凭据或V1内容。新测试只用synthetic fixtures/disposable Git/临时stores；没有启动原生应用或创建VM/Sandbox，没有安装浏览器/字体/IDE。

**Keil/IAR compiler cohorts仍UNVERIFIED；ArmLink/IAR MAP仍NOT_SUPPORTED / FIXTURE_BLOCKED。** 四类真实Clang对象只证明basic metadata，不证明厂商同源ELF/MAP、region/VMA/LMA/初始化压缩或source attribution；GNU ld既有正向fixtures和goldens不变。没有许可选择、schema、身份、安全/capability、签名/updater/network或新产品功能变更。

所有NEXT47记录放在仓库外，不为索引产生新commit；旧NEXT44/45/46报告与Review字节不改。整合报告及JSON、独立签署、原字段保留矩阵、QA风险、真实日志/样例、push/CI原证据收录于同名证据ZIP；构建target/node_modules/exe/scratch源副本排除，包hash清单在ZIP旁的交付manifest。
