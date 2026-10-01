---
title: "P4 Release Bundle Execution Report"
doc_id: "FS-P4-EXEC"
product: "FirmwareSight"
version: "1.0"
status: "VALIDATED"
stage: "P4_RELEASE_BUNDLE"
owner: "Engineering"
last_updated: "2026-10-01"
---

# P4 Release Bundle — execution report (prompt §56–§71)

## 1. What was built, commit by commit

| Commit | Content |
| --- | --- |
| `9e421a5` | stage opened: prompt registered with its SHA-256, `ACTIVE_TASK` set, governance activated |
| `b12961c` | Core release domain model, release-id fingerprint, portable `analysis:1`, manifest DTO, report HTML, the five bundle contracts |
| `ba0ac9a` | storage migration `0004_release_records.sql`, `SCHEMA_VERSION` 4, immutable release records |
| `cbd44a0` | verified bundle builder, staging/publish engine, standalone verifier |
| `01af703` | `fwsight release prepare` on the command line |
| `a786595` | desktop bundle IPC: bounded session registry, tokens instead of paths, the Release page's Bundle section |
| `6b7e23e` | desktop shell wiring, preview table, destination dialog, confirmation and result blocks |
| `ee27416` | fixtures, the golden bundle, `apps/cli/tests/p4_golden.rs`, `scripts/verify_bundle_portability.py` |
| `e799f2f` | the portability checker's clock rule narrowed to the one clock a bundle may carry |

No sixth first-party crate: the workspace still holds five library crates plus `apps/cli` and
`apps/desktop`. No new third-party dependency entered `Cargo.lock`; the portable reader is Python
stdlib with `jsonschema` optional, and Core validates contracts through its own dependency-free
`schema_check` subset. Filesystem assembly lives outside Core: Core owns release semantics, ids and
the byte plan; the engine in `apps/desktop/src-tauri` and the CLI own staging, rename and rollback.

## 2. The release subject and the golden

Every bundle in tests, smoke and goldens was produced from a throwaway project, never this
repository. The golden subject `fixtures/project/p4-release/` pins its Git identity (author and
committer name, email, both dates, message), which is what makes a golden possible at all: two
independent temporary repositories initialized with those facts produce the same HEAD
`585dafd3eb59592563923818cdfec4e9a370a83b`, and the test asserts that HEAD so a Git object-format
drift fails as a broken subject rather than as a changed bundle.

`golden/reports/p4-release/` holds the eight composed documents of one whole bundle — release id
`release-974f661d3a6def9d7e7393e940e907a3c462710df2d071ad8140fe8894742ab0`, folder
`brake-node-1.2.3-974f661d3a6d`, gate run
`gate-a17e9861ce643da5e7061ac349793f80d1be87736ad52b2daf3433dc8cb8bc26`, 10 files / 148,890 bytes —
compared byte for byte by `apps/cli/tests/p4_golden.rs`. The two artifacts are deliberately absent
from the golden: they are copies of committed fixtures whose hashes the manifest already pins, and
storing a third copy would only age.

## 3. CLI release smoke (§60)

`target/release/fwsight.exe` over a scratch subject under `/tmp`: **23 ok, 0 failed** across eleven
sections — PASS prepare, layout, `--json` manifest equality with the on-disk manifest, REVIEW
refusal, acceptance then prepare, stale-context refusal, occupied destination `ERR-BUNDLE-6106`,
`--force` against a stranger directory (`ERR-BUNDLE-6107`, exit 6, the stranger's file surviving) and
`--force` against a recognized bundle (replaced, byte-identical to the same release). Transcript kept
under gitignored `target/p4-cli-smoke-transcript.txt`; scratch `/tmp/p4-cli-smoke.DYhl9P`. The CLI
writes no release record and says so: "this run is not stored; the desktop Release page writes the
release record".

## 4. Portability / independent reader (§61)

Two readers that share no code with the producer:

- `apps/cli/tests/p4_golden.rs` relocates a freshly prepared bundle, deletes its inputs, and verifies
  it with the crate's own standalone verifier — release id, file count, artifact count, comparison
  and notes present — then answers the nine §61 questions from the bundle alone.
- `scripts/verify_bundle_portability.py`, Python stdlib with optional `jsonschema`, never importing
  first-party code: SHA256SUMS, the manifest/sums coverage relation
  (`listed − sums = {SHA256SUMS}`), schema validation, self-contained report, host-path and clock
  rules, and the same nine questions.

Measured on the desktop-exported bundle: **64 of 64 with `--schemas schemas`, 59 of 59 without**; on
the relocated copy with project, artifacts and (after the clean close) the database unavailable:
**59 of 59**. The nine answers it prints end with the integrity model in one sentence: no file
carries its own digest; `SHA256SUMS` hashes the payload, `release-manifest.json` hashes the payload
plus `SHA256SUMS`.

## 5. Validation commands (§68), measured on the final tree

Run separately, each recorded with its exit code in `target/p4-validate.log` (gitignored):

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | exit 0 |
| `cargo test --workspace` | exit 0 — **769 passed, 0 failed, 0 ignored** across 41 executable suites |
| `corepack pnpm install --frozen-lockfile` (in `apps/desktop/ui`) | exit 0 |
| `corepack pnpm typecheck` | exit 0 |
| `corepack pnpm lint` | exit 0 |
| `corepack pnpm test` | exit 0 — **155 passed in 6 files** |
| `corepack pnpm build` | exit 0 — `dist/assets/index-*.js` 314.49 kB, gzip 90.37 kB |
| `python scripts/check.py --only drift` | exit 0 |
| `python scripts/check.py --only deny` | exit 0 — `advisories ok, bans ok, licenses ok, sources ok` |
| `python scripts/check.py --only core-smoke` | exit 0 |
| `python scripts/check.py` | exit 0 — **15/15 steps passed** |
| `git diff --check` | exit 0, no output |

The 769 is the direct `cargo test --workspace` count and nothing else: no drift or core-smoke rerun
is summed into it. The retired 715 does not appear anywhere in this pack.

## 6. Release build (§69)

One chained command built both shipping binaries; both printed `Finished release profile
[optimized]` (54.87 s, then 3 m 49 s) and the `sha256sum` stage that followed printed

```
80feaff561cd4528f432bc047ecb4ed4427d1044bb636828941485ddebcc4a10 *target/release/fwsight.exe
54a87e341917ada3955939496301163f74eb35691b605bea09aa690d3f6244c9 *target/release/firmwaresight-desktop.exe
```

The desktop smoke walked the second binary. Whether it is the binary of the final tested tree is a
measured claim, not an assumption: the last commit touching any shipped source path is `6b7e23e`
(04:05:06), the binaries were written at 05:22:06 and 05:25:56, and
`git diff --stat 6b7e23e HEAD -- '*/src/*'` is empty — the commits after the build (`ee27416`,
`e799f2f`) moved only a CLI test, fixtures, goldens and Python scripts. Nothing in the shipped
surface changed after the smoke began, so nothing required a re-walk; had it, this report would name
what was and was not re-smoked.

## 7. Remote CI (§73, §74)

Every P4 commit was pushed and its run read back by exact head SHA:

| headSha | run | conclusion |
| --- | --- | --- |
| `cbd44a0` | 36826182802 | success |
| `01af703` | 36837949643 | success |
| `a786595` | 36842591014 | success |
| `6b7e23e` | 36853309527 | success |
| `ee27416` | 36861023043 | success |
| `e799f2f` | 36872456446 | success — **7 of 7 jobs**: Generated output drift, Desktop UI (windows-latest), Rust (windows-latest), Dependency policy, Rust (ubuntu-latest), macOS Core Smoke, Desktop UI (ubuntu-latest) |

`e799f2f` is the final implementation HEAD and its run is the implementation CI this stage cites. No
intermediate run was red, so there is no remote failure to record; had one appeared, it would be
named here rather than folded into a later green.

## 8. Integrity artifacts (§70)

The tracked path set changed with this stage, so `DIRECTORY_TREE.txt` was regenerated for the new
layout and `SHA256SUMS` regenerated last, in the closure commit that also carries this pack. Both are
verified by `scripts/verify_baseline_artifacts.py` (which re-derives the tracked set from
`git ls-files` and hashes off disk, sharing no code with the generator) and by `sha256sum -c
SHA256SUMS`: full coverage, manifest self-exclusion, 0 mismatch, 0 missing, 0 stale, no duplicate
path, LF endings, no host path. The repository manifest and the bundle's `SHA256SUMS` remain separate
concepts with separate generators.

## 9. License gap (§76)

Unchanged and untouched: `license = "Proprietary"` in the root `Cargo.toml`'s `[workspace.package]`
table, no root `LICENSE`, `OPEN_SOURCE_LICENSE_DECISION_PENDING_OWNER_CONFIRMATION` standing. The
decision is the owner's under `AGENTS.md` 9 and does not block technical MVP completion.

## 10. Not measured, not run

Peak RSS and any bundle-size throughput number: `NOT MEASURED`. Fuzzing, a macOS or Linux window,
keyboard-only traversal of the shipped binary, a second Windows host and any DPI other than 100%:
`NOT RUN`. The report's offline claim was observed in a browser at the bundle's `file://` URL, not by
an Explorer double-click.
