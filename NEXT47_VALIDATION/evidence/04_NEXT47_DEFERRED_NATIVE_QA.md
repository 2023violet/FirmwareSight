# NEXT-47 原生 QA 递延风险登记

候选：`76b5c2c006f1ca0850af86da6d1e4b957227a025`。这是一份新的工程集成风险裁决记录；历史 NEXT44/45/46 verdict、原 MUST 与 NOT_VERIFIED 状态全部保持不变。

**集成例外：`OWNER_ACCEPTED_DEFERRED_NATIVE_QA`，仅 C1–C3。原生产品验收：`NOT_VERIFIED / BLOCKED_BY_ENV`。**

实际Owner调度中的双开关是授权来源；不从附件内自举授权。本轮不启动原生应用、不准备VM/Sandbox、不访问Owner数据、不拍摄虚构界面。所有以下步骤均为未来真实QA计划，尚未执行。

## C1 / NEXT44-AC24

原AC（verbatim）：[MUST] save dialog cancel, overwrite denied, write failure, stale session, retry recover correctly.

原Reviewer：**NOT_VERIFIED**；MUST=true。来源 FirmwareSight_NEXT_44_Visual_Report_Export_Integrated_Product_Batch_2026-10-14.txt:342。

原Reviewer理由（verbatim）：Rust injected-dialog/export tests cover cancellation, denied overwrite, file preservation, stale handles, replaced destination and retry. Actual native Save-dialog cancellation/overwrite/write-error/retry sequence was not observed; native APIs disabled and no isolated app store established.

自动化覆盖边界：Rust 注入 dialog/writer/session 测试与 React DOM：Cancel、拒绝 overwrite 的文件保持、write failure、stale/source-target recheck、retry 和 delayed response；原 NEXT46 独立测试已执行，本轮新结果待 Reviewer 独立签署。

用户后果与未观察动作：用户可能取消后仍有输出、误覆盖旧报告、失败留半文件、换 selection 后保存旧结论或无法重试；错误证据可能误导工程判断，误覆盖可能丢失目标文件。

影响区域：Analyze / Compare / Gate / Release visual export and native dialog/controls。影响程度：潜在高影响：报告数据/目标文件安全；当前无观测到的S0/S1或已复现危险覆盖缺陷，风险假设不是已发现缺陷。

可恢复性：普通取消/拒绝覆盖应无写入；错误可换可写目录、重新 Analyze/Compare/Gate 后重试；若误覆盖或旧报告被当真，不能承诺自动恢复，只能依赖事先独立备份和重新生成/核验。

最小复测环境：Real Windows native app in dedicated isolated non-Owner user/profile; synthetic project/outputs; prove resolved store paths before launch. No VM or Sandbox prepared in NEXT47.

真实用例（计划，NOT_RUN）：

1. 在授权的隔离 Windows 原生用户/profile 中，先证明 data/config/cache/temp 全部在合成 namespace；Owner AppData、生产项目、凭据、V1均不可达。登记精确代码SHA、binary hash、若使用安装包则installer SHA256和来源；不调用Owner现有桌面进程。
2. 创建合成 ELF/MAP 项目及各状态输入，只用公开repo fixtures/人工合成数据。记录操作人、Windows版本、DPI、字体、语言、theme、窗口尺寸和空输出目录列表。对任何覆写目标先建立独立sentinel备份与hash。
3. 按 Analyze、Compare、Gate/Release 中实际提供的visual export入口和 PNG/SVG/HTML各格式重复适用用例，记录显示主体的snapshot/diff/run身份。
4. 正常 Save：鼠标启动真实OS Save，验证格式filter/default扩展名/目标，保存到合成可写目录；独立解码PNG、parseSVG和浏览HTML，核对精确bytes、Unknown/五态、中文长标签与当前身份。
5. Cancel：打开真实Save后取消；核对输出目录entries/hash和既有sentinel保持原样，UI没有虚假success，控件可再次使用。
6. 拒绝覆盖：选择预先存在sentinel目标，实际拒绝系统/应用确认；核对sentinel字节/hash不变，无临时/旁路写入，UI可恢复。
7. 确认覆盖：另一个有备份的合成sentinel，实际核对确认文案和完整目标再同意；只替换指定目标，输出完整可解码/核验，邻近sentinel不变。
8. 无权限写入：仅在合成namespace中设置并独立证明不可写的测试目录（不能用Owner目录/系统目录），触发真实保存失败；检查目标/旁文件无破坏，错误和diagnostics/retry可见。
9. Stale selection：保持Save流程或长导出未完成时更换合成选中文件/成功分析或diff/run；旧结果不能显示为新主体success/error，source stale应拒绝且无误写。分别对Analyze/Compare/Gate主体执行。
10. 重新分析与retry：从失败/stale状态返回、重新生成当前成功主体，保存到可写新目标；验证错误清除和按钮状态、输出当前身份，旧异步完成不会污染结果。
11. Keyboard-only：Tab/Shift-Tab、Enter/Space、Escape贯穿入口、格式、真实对话框、确认/取消、错误/retry；focus可见、顺序合理、关闭后回到正确发起控件，无trap。用实际Windows辅助技术检查name/role/state及五态词，不只看DOM。
12. 在System/Light/Dark及中文/英文分别重复入口/Save/拒绝覆盖/错误恢复；检查真实dialog title、filter、自定义确认copy和结果提示，不把OS自动本地化或英文技术detail当全UI本地化。
13. 在1024×720及正常窗口、代表DPI重新执行keyboard/focus/错误/retry/主题；记录重要表格/控件无不可达遮挡、无纯颜色状态；完整录像/截图加步骤结果和异常，不摆拍点击。
14. 退出并再次启动同一隔离构建，证明新合成store行为；记录所有目录/目标hash变化，核查无Owner访问。发现S0/S1/错误覆盖/泄漏/可重复流程缺陷立即停止分发，形成缺陷并最小修复后以新SHA重新测试。

有效证据：

- exact full build SHA and clean source/tree
- binary SHA256; installer SHA256+CI origin if installed, otherwise NOT_RUN and no installed acceptance claim
- identified human operator, UTC timestamps, Windows/DPI/font/language/theme/window
- continuous actual native UI video/screenshots, individual steps/expected/actual results
- before/after synthetic outputs/sentinels hashes, independently decoded outputs
- defect IDs/severity, repair SHA and repeat observations

解除条件：All applicable original native MUST scenarios actually observed PASS; serious defects fixed/retested and independent native acceptance signed. Integration CI or component tests alone cannot close.

强制截止：**MUST close BEFORE any Developer Preview external distribution OR Private Beta OR RC, whichever is first; no date guessed.**

风险处置：OWNER_ACCEPTED_FOR_ENGINEERING_INTEGRATION_ONLY; NOT_ACCEPTED_AS_NATIVE_PRODUCT_ACCEPTANCE

## C2 / NEXT44-AC26

原AC（verbatim）：[MUST] desktop bilingual labels and accessibility: keyboard, icon+word statuses, light/dark controls.

原Reviewer：**NOT_VERIFIED**；MUST=true。来源 FirmwareSight_NEXT_44_Visual_Report_Export_Integrated_Product_Batch_2026-10-14.txt:344。

原Reviewer理由（verbatim）：Localized React labels, named buttons, icon+word statuses and frozen theme CSS are present/tested. Actual native keyboard/focus/accessibility/light-dark/min-width interaction has not been observed; DOM/source checks are insufficient for full native workflow acceptance.

自动化覆盖边界：React i18n、语义按钮/状态 icon+word、aria hooks、token/theme source 与组件测试；代码/DOM 层，不能证明 Windows 焦点、屏幕阅读器、主题或1024×720真实可用性。

用户后果与未观察动作：键盘或辅助技术用户可能无法到达/退出导出控件，焦点丢失或被困；深浅主题或最小窗口可能使重要状态/恢复动作不可读或不可操作。

影响区域：Analyze / Compare / Gate / Release visual export and native dialog/controls。影响程度：潜在中高影响：可访问性与导出可用性，可能对键盘/辅助技术用户完全阻断；当前原生未观察，不编造严重性已验证结论。

可恢复性：若仅视觉/布局失败可尝试扩大窗口或切换主题；严重键盘/focus失败不能假定鼠标可替代辅助技术，必须修复并重测，期间禁止对外分发。

最小复测环境：Real Windows native app in dedicated isolated non-Owner user/profile; synthetic project/outputs; prove resolved store paths before launch. No VM or Sandbox prepared in NEXT47.

真实用例（计划，NOT_RUN）：

1. 在授权的隔离 Windows 原生用户/profile 中，先证明 data/config/cache/temp 全部在合成 namespace；Owner AppData、生产项目、凭据、V1均不可达。登记精确代码SHA、binary hash、若使用安装包则installer SHA256和来源；不调用Owner现有桌面进程。
2. 创建合成 ELF/MAP 项目及各状态输入，只用公开repo fixtures/人工合成数据。记录操作人、Windows版本、DPI、字体、语言、theme、窗口尺寸和空输出目录列表。对任何覆写目标先建立独立sentinel备份与hash。
3. 按 Analyze、Compare、Gate/Release 中实际提供的visual export入口和 PNG/SVG/HTML各格式重复适用用例，记录显示主体的snapshot/diff/run身份。
4. 正常 Save：鼠标启动真实OS Save，验证格式filter/default扩展名/目标，保存到合成可写目录；独立解码PNG、parseSVG和浏览HTML，核对精确bytes、Unknown/五态、中文长标签与当前身份。
5. Cancel：打开真实Save后取消；核对输出目录entries/hash和既有sentinel保持原样，UI没有虚假success，控件可再次使用。
6. 拒绝覆盖：选择预先存在sentinel目标，实际拒绝系统/应用确认；核对sentinel字节/hash不变，无临时/旁路写入，UI可恢复。
7. 确认覆盖：另一个有备份的合成sentinel，实际核对确认文案和完整目标再同意；只替换指定目标，输出完整可解码/核验，邻近sentinel不变。
8. 无权限写入：仅在合成namespace中设置并独立证明不可写的测试目录（不能用Owner目录/系统目录），触发真实保存失败；检查目标/旁文件无破坏，错误和diagnostics/retry可见。
9. Stale selection：保持Save流程或长导出未完成时更换合成选中文件/成功分析或diff/run；旧结果不能显示为新主体success/error，source stale应拒绝且无误写。分别对Analyze/Compare/Gate主体执行。
10. 重新分析与retry：从失败/stale状态返回、重新生成当前成功主体，保存到可写新目标；验证错误清除和按钮状态、输出当前身份，旧异步完成不会污染结果。
11. Keyboard-only：Tab/Shift-Tab、Enter/Space、Escape贯穿入口、格式、真实对话框、确认/取消、错误/retry；focus可见、顺序合理、关闭后回到正确发起控件，无trap。用实际Windows辅助技术检查name/role/state及五态词，不只看DOM。
12. 在System/Light/Dark及中文/英文分别重复入口/Save/拒绝覆盖/错误恢复；检查真实dialog title、filter、自定义确认copy和结果提示，不把OS自动本地化或英文技术detail当全UI本地化。
13. 在1024×720及正常窗口、代表DPI重新执行keyboard/focus/错误/retry/主题；记录重要表格/控件无不可达遮挡、无纯颜色状态；完整录像/截图加步骤结果和异常，不摆拍点击。
14. 退出并再次启动同一隔离构建，证明新合成store行为；记录所有目录/目标hash变化，核查无Owner访问。发现S0/S1/错误覆盖/泄漏/可重复流程缺陷立即停止分发，形成缺陷并最小修复后以新SHA重新测试。

有效证据：

- exact full build SHA and clean source/tree
- binary SHA256; installer SHA256+CI origin if installed, otherwise NOT_RUN and no installed acceptance claim
- identified human operator, UTC timestamps, Windows/DPI/font/language/theme/window
- continuous actual native UI video/screenshots, individual steps/expected/actual results
- before/after synthetic outputs/sentinels hashes, independently decoded outputs
- defect IDs/severity, repair SHA and repeat observations

解除条件：All applicable original native MUST scenarios actually observed PASS; serious defects fixed/retested and independent native acceptance signed. Integration CI or component tests alone cannot close.

强制截止：**MUST close BEFORE any Developer Preview external distribution OR Private Beta OR RC, whichever is first; no date guessed.**

风险处置：OWNER_ACCEPTED_FOR_ENGINEERING_INTEGRATION_ONLY; NOT_ACCEPTED_AS_NATIVE_PRODUCT_ACCEPTANCE

## C3 / NEXT45-AC20

原AC（verbatim）：UI localized and typed: controls, loading, error, cancellation, file save and overwrite protection.

原Reviewer：**NOT_VERIFIED**；MUST=true。来源 FirmwareSight_NEXT_45_Visual_Report_Export_Scoped_License_Admission_Continuous_Completion_2026-10-14.txt:325。

原Reviewer理由（verbatim）：Localized source and typed IPC plus injected cancellation/overwrite/write/stale/retry tests pass. Actual OS Save dialog and overwrite controls have not been observed in an isolated native app; cannot promote DOM/Rust proof to native UI evidence.

自动化覆盖边界：中英 React catalog、typed IPC、loading/error/cancel/overwrite 回归；既有 Rust native title/custom confirmation 英文 literal 是已观察源码限制，不能据 React 本地化宣称 native 已本地化。

用户后果与未观察动作：中文用户可能误解原生英文 Save/overwrite 提示，错误确认目标、误覆盖或无法理解失败/取消；React 和原生提示不一致可能导致恢复失败。

影响区域：Analyze / Compare / Gate / Release visual export and native dialog/controls。影响程度：潜在中高影响：本地化决策和文件保存/覆盖；英文literal为已知代码限制，原生行为/理解后果尚未观察。

可恢复性：取消后选择清晰目标、重新生成并独立核验；误覆盖仅能依赖备份，不能保证恢复；若实际本地化不能满足原AC，登记缺陷、修复/重测后才能解除。

最小复测环境：Real Windows native app in dedicated isolated non-Owner user/profile; synthetic project/outputs; prove resolved store paths before launch. No VM or Sandbox prepared in NEXT47.

真实用例（计划，NOT_RUN）：

1. 在授权的隔离 Windows 原生用户/profile 中，先证明 data/config/cache/temp 全部在合成 namespace；Owner AppData、生产项目、凭据、V1均不可达。登记精确代码SHA、binary hash、若使用安装包则installer SHA256和来源；不调用Owner现有桌面进程。
2. 创建合成 ELF/MAP 项目及各状态输入，只用公开repo fixtures/人工合成数据。记录操作人、Windows版本、DPI、字体、语言、theme、窗口尺寸和空输出目录列表。对任何覆写目标先建立独立sentinel备份与hash。
3. 按 Analyze、Compare、Gate/Release 中实际提供的visual export入口和 PNG/SVG/HTML各格式重复适用用例，记录显示主体的snapshot/diff/run身份。
4. 正常 Save：鼠标启动真实OS Save，验证格式filter/default扩展名/目标，保存到合成可写目录；独立解码PNG、parseSVG和浏览HTML，核对精确bytes、Unknown/五态、中文长标签与当前身份。
5. Cancel：打开真实Save后取消；核对输出目录entries/hash和既有sentinel保持原样，UI没有虚假success，控件可再次使用。
6. 拒绝覆盖：选择预先存在sentinel目标，实际拒绝系统/应用确认；核对sentinel字节/hash不变，无临时/旁路写入，UI可恢复。
7. 确认覆盖：另一个有备份的合成sentinel，实际核对确认文案和完整目标再同意；只替换指定目标，输出完整可解码/核验，邻近sentinel不变。
8. 无权限写入：仅在合成namespace中设置并独立证明不可写的测试目录（不能用Owner目录/系统目录），触发真实保存失败；检查目标/旁文件无破坏，错误和diagnostics/retry可见。
9. Stale selection：保持Save流程或长导出未完成时更换合成选中文件/成功分析或diff/run；旧结果不能显示为新主体success/error，source stale应拒绝且无误写。分别对Analyze/Compare/Gate主体执行。
10. 重新分析与retry：从失败/stale状态返回、重新生成当前成功主体，保存到可写新目标；验证错误清除和按钮状态、输出当前身份，旧异步完成不会污染结果。
11. Keyboard-only：Tab/Shift-Tab、Enter/Space、Escape贯穿入口、格式、真实对话框、确认/取消、错误/retry；focus可见、顺序合理、关闭后回到正确发起控件，无trap。用实际Windows辅助技术检查name/role/state及五态词，不只看DOM。
12. 在System/Light/Dark及中文/英文分别重复入口/Save/拒绝覆盖/错误恢复；检查真实dialog title、filter、自定义确认copy和结果提示，不把OS自动本地化或英文技术detail当全UI本地化。
13. 在1024×720及正常窗口、代表DPI重新执行keyboard/focus/错误/retry/主题；记录重要表格/控件无不可达遮挡、无纯颜色状态；完整录像/截图加步骤结果和异常，不摆拍点击。
14. 退出并再次启动同一隔离构建，证明新合成store行为；记录所有目录/目标hash变化，核查无Owner访问。发现S0/S1/错误覆盖/泄漏/可重复流程缺陷立即停止分发，形成缺陷并最小修复后以新SHA重新测试。

有效证据：

- exact full build SHA and clean source/tree
- binary SHA256; installer SHA256+CI origin if installed, otherwise NOT_RUN and no installed acceptance claim
- identified human operator, UTC timestamps, Windows/DPI/font/language/theme/window
- continuous actual native UI video/screenshots, individual steps/expected/actual results
- before/after synthetic outputs/sentinels hashes, independently decoded outputs
- defect IDs/severity, repair SHA and repeat observations

解除条件：All applicable original native MUST scenarios actually observed PASS; serious defects fixed/retested and independent native acceptance signed. Integration CI or component tests alone cannot close.

强制截止：**MUST close BEFORE any Developer Preview external distribution OR Private Beta OR RC, whichever is first; no date guessed.**

风险处置：OWNER_ACCEPTED_FOR_ENGINEERING_INTEGRATION_ONLY; NOT_ACCEPTED_AS_NATIVE_PRODUCT_ACCEPTANCE

## 强制产品边界

本轮集成不批准 Developer Preview 外发、Private Beta、RC/GA、公开Release、签名或license选择；CI package构建也不能替代安装/原生产品验收。Keil/IAR厂商cohort与MAP支持仍保持未验证/不支持。

原 NEXT46 AC34 的完整原合同验收前提 FAIL 仍为历史结果。本轮独立 Reviewer 必须另行判断 CODE_INTEGRATION_GATE，不能改写该 FAIL 或使用 ACCEPT_WITH_LIMITS 模糊批准。
