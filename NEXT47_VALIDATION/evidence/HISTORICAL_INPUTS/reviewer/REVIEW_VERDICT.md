# Independent final-SHA review — NEXT46

**Verdict: BLOCKED_INSUFFICIENT_EVIDENCE / NO_PUSH.** This is not unconditional acceptance and does not satisfy `OWNER_NEXT46_CONDITIONAL_FF_PUSH=YES`. No candidate code defect was found in the independently executed source, CLI, parser, report, image, browser-print and regression checks. Three applicable mandatory native workflow requirements lack actual observations; successful Rust and DOM tests cannot replace them.

Reviewed Git object: `76b5c2c006f1ca0850af86da6d1e4b957227a025`, direct parent `4ac1e4f709396fac7a37a1d92b8095a070b3effa`, tree `55ecc38ae7e45976e60845249dee8830d35e054d`. The reviewer used a new managed clean detached worktree at `LOCAL_USER/.codex/worktrees/next46-final-review/FirmwareSight`, with a fresh worktree-local target. The reviewer is a separate context, did not author the candidate, and made no candidate code/document changes or push. Final status is clean and unchanged (`git-after.json`).

The original NEXT44, NEXT45 and NEXT46 acceptance texts were read directly. Their byte digests and exact source-line requirements are included in `AC_MATRIX_106.json`, with the original NEXT44 digest matching `4b651a2f2487f3b7b1b7c64c9abf69b1962a953bcb27fe1afd6d895ca769a1fd`. `AC_MATRIX_106.tsv` is the flat 106-row form. Historical acceptance is never reissued by this matrix: original stage baselines are reviewed as lineage, with the current NEXT46 parent governing this candidate.

| Classification | Rows |
|---|---:|
| PASS | 96 |
| NOT_VERIFIED | 3 |
| FAIL — unmet acceptance prerequisite | 1 |
| CONDITIONAL N/A — no push permitted | 3 |
| NOT_RUN — final-SHA remote CI | 3 |
| Total | 106 |

## Mandatory unresolved items

| Requirement | Actual missing evidence | Classification |
|---|---|---|
| NEXT44 AC24 | Native OS Save dialog cancellation, denied overwrite, write failure, stale session and retry workflow in a safely isolated actual app. Injected dialog/file tests verify code behavior, but no native dialog sequence was observed. | NOT_VERIFIED |
| NEXT44 AC26 | Actual native keyboard/focus/accessibility and light/dark controls, including the 1024×720 usability path. Localized source, status semantics and DOM tests exist, but native interaction has not been observed. | NOT_VERIFIED |
| NEXT45 AC20 | Actual native file-save and overwrite interaction with localized controls and outcomes. Typed IPC and source/DOM tests cannot establish native UI behavior. | NOT_VERIFIED |
| NEXT46 AC34 | Every applicable mandatory original-contract requirement must fully pass. The three items above do not. | FAIL — prerequisite unmet |

NEXT44 excludes **installed** desktop QA for this batch. That does not remove its native workflow, keyboard/theme and localized-control requirements; an isolated development native runtime would also need actual observations. Native Windows app control is disabled here, and no isolated desktop application store was established. No native app was launched, no Owner appdata was opened, and no native interaction was inferred from tests. See `native-evidence-boundary.md`.

NEXT44 AC39, NEXT45 AC28 and NEXT46 AC35 are CONDITIONAL N/A for the new candidate: the full-acceptance antecedent is false, so the Owner push switch cannot permit a push. NEXT44 AC40, NEXT45 AC29 and NEXT46 AC36 are NOT_RUN: no final-SHA push/remote CI exists. Baseline Actions #112 is not final-SHA proof. A subsequent exact-SHA review must establish all missing MUST evidence before any conditional push; no waiver or historical `ACCEPT_WITH_LIMITS` can substitute.

## Independently executed verification

The complete fresh `python scripts/check.py` invocation exited **0**, **20/20 steps, zero SKIP** (`full-gate.json`, `full-gate.log`). This clean worktree required two frontend-assets preparation steps, so its count is 20 rather than the executor's 18 in a worktree with existing frontend assets. One nonduplicated `cargo test --workspace` segment contains **1,160 passed / 0 failed / 2 ignored / 56 result lines**, with the two preexisting opt-in 100K ignores kept separate (`workspace-count.json`). Frontend: **397 passed in 13 files**, with typecheck, lint and build green. Rust fmt and clippy pass. Separate Core smoke exits 0, **3/3 steps** (`core-smoke.json`).

Four separate fresh online `cargo deny check` categories — licenses, bans, sources, advisories — each exited **0**. External no-exception, wrong-package-name and wrong-exact-version policies each exited **4** with a genuine `arrayref` license refusal. The global allowlist and all other policy settings remain unchanged. Actual cargo tree and fetched license notice were independently checked (`arrayref-tree.log`, `resvg-features.log`, `license-notice-proof.json`). No claim of advisory-free dependencies or authorized public distribution is made.

Two required source mutations were independently executed in a disposable separate source copy with its own target, never the candidate. Bypassing safe display caused the named privacy assertion to fail; adding one displayed byte caused the named exact-value assertion to fail. Both final controls exited **101** with **test result: FAILED**, and original disposable source bytes were restored (`negative-privacy-correct.log`, `negative-chart-fact-correct.log`, `negative-controls-summary.json`). Initial Windows encoding-error runs were not assertion proof and are explicitly excluded; their logs remain for provenance.

Final ADR-0029 verifier: **894 tracked paths / 892 sum entries**, **0** index mismatch, unlisted/duplicate/unmerged entries or artifact/blob disagreement, **RESULT PASS** in the full gate. Public schemas, Core, storage, identity code, Tauri capabilities, locks, policy and goldens are unchanged by NEXT46. The inherited visual HTML golden refresh only changes HTML and required hash/size index fields; Release identity and other portable truth bytes remain unchanged. Source findings are documented in `source-review.md`.

## Independently generated and observed artifacts

Six synthetic Core cohorts were generated twice from this exact candidate. All 18 PNG/SVG/HTML pairs were byte-identical on this host (`sample-manifest.json`). Pillow **12.3.0**, independently of the encoder, decoded each real **1600×900 PNG**, with substantial nonwhite pixels, white background and meaningful text/graphics. ElementTree independently parsed each standalone SVG and rejected risky tags/attributes, external references, canary text and invalid rectangle geometry (`independent-artifacts.json`). All six actual PNGs were directly viewed.

An isolated headless Edge context opened the real offline HTML files with no HTTP requests or script errors, checked SVG text bounds and produced actual A4 browser-print PDFs. Summary artifacts are **2 pages** each; detailed Compare is **16 pages**, detailed Release **12 pages**. Independent pypdf reading verified A4 boxes, nonblank final pages and **every source table cell's text** across pagination. Poppler rendered all detailed pages; both complete contact sheets and summary pages were viewed. This establishes browser print, not native in-app PDF or native desktop Save behavior (`browser-review.json`, `print-review.json`, `visual-inspection.md`).

Four genuine Clang relocatable objects regenerated byte-identically using the existing compiler and were independently inspected with LLVM and raw header checks: ARM32/AArch64, little/big endian. They establish basic metadata only, not vendor compiler support or executable memory accounting (`fixture-proof.json`, `readobj/`). Foreign MAP banners outrank incidental GNU text and are explicitly refused; ELF-only recovery remains useful. ArmLink and IAR adapters remain NOT_SUPPORTED / FIXTURE_BLOCKED, with vendor compiler cohorts UNVERIFIED. See `vendor-semantics-review.md`.

## Workspace protection and history

Official remote `https://github.com/2023violet/FirmwareSight.git` and remote main `4ac1e4f709396fac7a37a1d92b8095a070b3effa` were independently read via `git ls-remote` and GitHub REST. Protected main before/after matches `e34f9e69e515a330a3705470aeba6cfab073634d`, exactly nine modes/OIDs/status entries and the **19,794-byte raw patch SHA256 `05f86c0313040899dca1156f538186d686377906e1a62e755fcb029c1eb96d2b`**. All protected Git reads used `GIT_OPTIONAL_LOCKS=0`; no stage/reset/checkout/stash/commit/reconciliation occurred there. Owner data, credentials, private firmware and V1 archives were not accessed. Tests used repository synthetic fixtures and bounded temporary stores/outputs only.

The earlier NEXT45 count was independently extracted from the actual external executor log as **1,154 / 0 / 2 / 54** (`historical-count-verification.json`). It is historical evidence, not final-candidate evidence. Historical Owner status remains **ENGINEERING_DELIVERED / REMOTE_CI_PROVEN / REVIEW_EXCEPTIONS_OPEN**, and the earlier push despite reviewer `ACCEPT_WITH_LIMITS` remains a conditional-push governance nonconformance. The recovered historical text is only the NEXT44 dependency blocker review. The full submitted NEXT45 final review was not physically recovered and has not been reconstructed or retroactively accepted.

Existing executor release-size artifacts and corrected comparable-build logs are provenance-qualified external evidence: baseline 7,221,248 bytes, candidate 7,222,784 bytes. No new reviewer release size build, cross-host PNG byte equality, performance improvement, installed QA, native PDF, signing, license choice, public release or V1 research claim is made.

The exact local candidate and stable evidence are ready for Owner/Architect review. **No push is permitted under the current full-acceptance condition.**
