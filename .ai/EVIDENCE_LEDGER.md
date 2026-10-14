# Concise evidence ledger — NEXT-36..46, observed 2026-10-14

Provenance: Git objects, live remote main/Actions, physically present original prompt bytes, and Owner's NEXT-46 dispatch. Code and CI are not independent architectural acceptance. No historical reviewer words or commit messages are rewritten. An unavailable source/report remains unavailable; the table does not infer a closed verdict from a filename or green CI.

| Iteration | Task / exact code SHA | Acceptance / evidence boundary |
|---|---|---|
| 36 | DOC-1 Wave 1A, `4b8df070dde19b586fedc177c5640fdbb44779a2` | [Code integrated remotely](https://github.com/2023violet/FirmwareSight/commit/4b8df070dde19b586fedc177c5640fdbb44779a2); independent historic verdict not re-adjudicated here |
| 37 | V1 private preservation | Original prompt present; execution and private archive deliberately NOT_ASSESSED |
| 38 | Isolated installed Windows QA | Prompt present; original NEXT-25 environment limitation remains. No installed/runtime PASS inferred |
| 39 | Source prompt not physically found | UNKNOWN; no fabricated task, SHA or verdict |
| 40–41 | Project readiness, `3706cf0b29e3009b54894ddce6a06068c5f51973` | [Code](https://github.com/2023violet/FirmwareSight/commit/3706cf0b29e3009b54894ddce6a06068c5f51973), followed by privacy rework; original independent disposition not replaced |
| 42 | Privacy rework through `5eb0a2977324c891748898b4e02586a1a8da49de` | [Code lineage](https://github.com/2023violet/FirmwareSight/commit/5eb0a2977324c891748898b4e02586a1a8da49de); historic acceptance not re-issued |
| 43 | Bounded intake/workload, `f2fcabe1393cbb74f308ddf4ccd0fc4b8931f54e` | [Actions #111](https://github.com/2023violet/FirmwareSight/actions/runs/38507903050), remote CI success; two 100K tests are opt-in, not added to default totals |
| 44–45 | Visual export, `4ac1e4f709396fac7a37a1d92b8095a070b3effa` | [Actions #112](https://github.com/2023violet/FirmwareSight/actions/runs/38535378887), 10 jobs success. Owner: ENGINEERING_DELIVERED / REMOTE_CI_PROVEN / REVIEW_EXCEPTIONS_OPEN. Prior ACCEPT_WITH_LIMITS and conditional-push nonconformance retained |
| 46 | Export privacy/SVG + ELF/MAP foundation, descendant of exact `4ac1e4f709396fac7a37a1d92b8095a070b3effa` | Isolated LOCAL candidate; final exact SHA, 106-item independent matrix and verdict are external delivery records. New CI NOT_RUN until permitted push. Native dialogue/runtime gaps may bar push |

Original prompt SHA-256, measured from bytes (prompt availability does not grant authority):

| NEXT | SHA-256 |
|---|---|
| 36 | `14d308ff417bf388641738e6510e4f65bc6cf79ed68a3ac3ace2309d1a7a8418` |
| 37 | `9838f142c9ca56bfa9f7a2300e89dac0d4b00fb24c2ed12db4c92ece8ff9ecc8` |
| 38 | `2c647e80647829413cd76305c5a9573f5f1d7edc9a325c8b92946ac4604b3e86` |
| 39 | NOT_FOUND |
| 40 | `a435ecaa88081bd75557820ce0871b1161a00e625c31fbffcaa2dbb50901c77a` |
| 41 | `c5d62893f56e3b6125c3997141f57330d750d8ed19d06221541a9ffda732a5ca` |
| 42 | `1fc91db1f28e75a8348011f2ddb9b561e1c9d6491af7a10e70a19c60f1463c8d` |
| 43 | `ed5c2863f5f8a49f2ab84f21076028a6819a79daeacfe755847564c4b895212f` |
| 44 | `4b651a2f2487f3b7b1b7c64c9abf69b1962a953bcb27fe1afd6d895ca769a1fd` |
| 45 | `7f5b979614761bd9ea9ac1bd639f406d05d67f4a001be97307ef25333df1acae` |
| 46 | `0280db1a8ebe3a739c92615a260a07146a67f0ee8aec9830613f76190996d9fa` |

NEXT-45 count addendum: one complete Windows `cargo test --workspace` invocation returned **1,154 pass / 0 fail / 2 ignored / 54 result lines**. The committed 1,448/68 claim included overlapping runs; it remains historical text, not the current total. Actual NEXT-46 counts are recorded from one fresh final workspace invocation, separately from IPC drift/Core smoke reruns.

Existing historical executor/reviewer test, license-negative-control and print files were located externally. A submitted NEXT-45 final reviewer report/matrix artifact was not physically recovered during scoped search; only its Owner-declared verdict is recorded. All 40 NEXT-44 and 30 NEXT-45 authoritative AC are physically available and must be checked independently on the new final SHA. No missing report is reconstructed as an invented acceptance.

## NEXT-47 scoped engineering integration and subsequent evidence upload — 2026-10-14

The dated NEXT-46 row above records its earlier local/evidence-blocked state and is retained. Under the Owner's explicit NEXT-47 reclassification and conditional-push authorization, independent clean detached review signed `CODE_INTEGRATION_GATE = PASS` on unchanged candidate `76b5c2c006f1ca0850af86da6d1e4b957227a025`, parent `4ac1e4f709396fac7a37a1d92b8095a070b3effa`. One ordinary exact-SHA FF integrated the code. [Actions #113](https://github.com/2023violet/FirmwareSight/actions/runs/38595778036), attempt 1, proves all 10 Jobs and 10 Check Runs on that SHA, including independently read actual cargo-deny and baseline/drift logs. Final local gate: 18/18, zero SKIP; Rust workspace 1,160 passed / 0 failed / 2 preexisting opt-in ignored; frontend 397 tests / 13 files.

`NATIVE_USER_ACCEPTANCE_GATE = NOT_VERIFIED / BLOCKED_BY_ENV`. The original NEXT-44 AC24/AC26 and NEXT-45 AC20 MUST states remain NOT_VERIFIED; only their scoped integration exception is Owner-accepted deferred native QA. Their full risk records require closure before the earliest external Developer Preview distribution, Private Beta or RC. Original NEXT-46 `BLOCKED_INSUFFICIENT_EVIDENCE / NO_PUSH` and original matrix verdicts are not rewritten. Keil/IAR vendor compatibility remains unverified and ArmLink/IAR MAP unsupported.

The subsequent Owner message explicitly requests uploading this delivery to main. [NEXT47_VALIDATION](../NEXT47_VALIDATION/README.md) is a later documentation/evidence publication with a public derivative, host-path placeholders, per-file original/published SHA-256 mapping and omissions of execution helpers/protected staged patches. Local originals remain byte-identical. Review and CI for this upload commit are separate from the already proven code SHA; no earlier run is claimed to certify the later commit. This does not grant native/installed acceptance, signing, a public product release or a license change.
