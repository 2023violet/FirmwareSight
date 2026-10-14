# 前检与调用更正记录

- 初次检查时 NEXT46 证据目录缺失，没有因此宣称复审/测试通过或推送。Owner 随后恢复原目录，374 文件内层清单及 Reviewer/矩阵/报告已知 hash 全部核验。可选外层交付清单仍缺失，明确标记；合同仅要求存在时读取它。
- 原 main 新出现一份未跟踪的 Playwright console 日志，原始 status 差异完整保留。九个 staged 项、OID、mode、patch、HEAD 和 tracked status 均精确匹配。没有读取或删除该日志，没有把 raw status 描述成完全一致。
- 首次读取猜测的 CI 文件名 ci.yml 失败；随后用 rg --files 定位权威 .github/workflows/p0-check.yml。这是读取路径错误，不是测试或 CI 失败。

- 首次推送前闸门发现增强矩阵的 NEXT45 AC26 requirement 被自动替换编码异常字符为正常破折号。闸门在实际 git push 调用与 attempt 文件创建之前终止，实际推送次数0。原历史矩阵未改；独立 Reviewer 保留初版、只修外部新增矩阵字段、单列更正并对同一 SHA 重新签署。

- 对前一条“自动替换字符”的诊断更正：真正原因是 Reviewer signing 脚本按 Windows 默认 GBK 读取原 UTF-8 JSON，导致增强矩阵一个 requirement 字段乱码。原件是正常 UTF-8 en dash，始终未改；Executor 闸门读取双方均显式 UTF-8，比较正确，只有默认 stdout 到工具的显示造成初次描述反转。码点差异在 PRE_PUSH_ARTIFACT_GUARD_ABORT.json，最终更正由 Reviewer 单列并重新签署。
