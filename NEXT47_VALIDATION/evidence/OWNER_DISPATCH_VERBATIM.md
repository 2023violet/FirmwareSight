# NEXT-47 actual human dispatch — verbatim transcription

Source: the latest human Owner message authorizing NEXT-47 in this current chat, not the attached contract. Transcribed by the executor for independent review; a provider message ID is unavailable. This is a transcription of already-granted authorization, not an additional approval.

```text
FirmwareSight NEXT-46 已形成完整本地候选，但原生桌面交互验收缺少实际观察，正确停止推送。
架构决定调整工程集成与原生产品验收的边界。
现明确授权执行附件：
**《NEXT-47｜Engineering Integration Gate & Native QA Deferral》**
待审候选 SHA：
`76b5c2c006f1ca0850af86da6d1e4b957227a025`
远端起始基线：
`4ac1e4f709396fac7a37a1d92b8095a070b3effa`
Owner 本次明确授权：
`OWNER_NEXT47_ENGINEERING_GATE_RECLASSIFICATION=YES`
`OWNER_NEXT47_CONDITIONAL_FF_PUSH=YES`
授权范围仅限 NEXT-46 的工程代码集成。NEXT-44 AC24、AC26 与 NEXT-45 AC20 维持 NOT_VERIFIED，但可以在完整风险登记之后递延至真实原生桌面产品验收，不能伪报 PASS。
请在独立 clean detached worktree 上，对既有候选及原 106 项矩阵进行集中复审。重点核实代码、隐私边界、ELF/MAP 兼容声明、图表正确性、异步状态、回归测试和完整工程门禁。
本轮不要重写已完成的功能，不要重复准备虚拟机，不要要求我逐项审批。
如果没有实际源码缺陷，优先保持现有候选 SHA 不变。
只有独立 Reviewer 明确签署工程集成门禁 PASS、无 S0/S1 或未受理严重缺陷、原 main 九个 staged 文件不变、远端基线不变时，才允许将精确候选普通非强制 Fast-forward Push 到 origin/main。
随后核验精确 SHA 的全部远端 CI Jobs、Check Runs、cargo-deny 和基线完整性。
不得将此次工程集成解释为原生桌面验收、Keil/IAR 正式兼容或产品公开发行授权。
请一次性交付完整复审、递延 QA 风险清单、最终 SHA 和远端 CI 报告。
```
