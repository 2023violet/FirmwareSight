# NEXT-47 Owner 调度来源

授权来源为当前 chat 中人类 Owner 在 2026-10-14 实际发出的调度，附件自身不构成授权。实际调度明确包含：

```text
OWNER_NEXT47_ENGINEERING_GATE_RECLASSIFICATION=YES
OWNER_NEXT47_CONDITIONAL_FF_PUSH=YES
```

范围仅为 NEXT-46 候选 `76b5c2c006f1ca0850af86da6d1e4b957227a025` 的工程集成，远端起始基线为 `4ac1e4f709396fac7a37a1d92b8095a070b3effa`。三项原生 MUST（NEXT44 AC24、AC26，NEXT45 AC20）保持 NOT_VERIFIED，允许完整风险登记后的明确递延。没有真实源码缺陷时保持现有 SHA。

只有独立 CODE_INTEGRATION_GATE 明确 PASS、没有 S0/S1 或未受理严重实际缺陷、原 main 九个 staged 项和远端基线不变，才可普通非强制推送精确 SHA 一次，随后验证新的远端 CI 全部 Jobs、Checks、依赖门禁和基线实际日志。不逐项请求审批，不准备 VM；不授权原生验收、厂商正式兼容、签名、tag、公开 Release、Beta/RC/GA 或产品外发。

附件原文已逐字节复制至此仓库外证据目录，hash 在 01_NEXT47_PRECHECK.json。历史 NEXT44/45/46 原件、Reviewer 原文和 106 项矩阵不修改，所有 NEXT47 新裁决只追加在本轮外部证据中。
