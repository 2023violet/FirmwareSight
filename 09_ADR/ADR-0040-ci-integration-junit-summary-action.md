---
title: "ADR-0040: CI integration — JUnit, PR summary and a composite GitHub Action"
doc_id: "FS-ADR-0040"
product: "FirmwareSight"
version: "0.8.0"
status: "BASELINE"
owner: "Engineering"
last_updated: "2026-11-09"
---

# ADR-0040: CI integration — JUnit, PR summary and a composite GitHub Action

## Status

**BASELINE** (2026-11-09).
- This is batch D9, authorised by the Owner's written exception for the "too monotonous" round (`.ai/DECISIONS.md` F1, F3).
- It builds on ADR-0023 (Gate states), ADR-0037 (release provenance) and ADR-0039 (growth waterfall).
- The plan named this record ADR-0042. D9 landed before D8, so it takes the next free number, and D8's records follow it.

## Context

- `fwsight gate` already exits 0 / 4 / 5 for PASS / REVIEW / BLOCK, and `fwsight diff --markdown` writes a full report. So a CI job can already fail on a bad build.
- What CI cannot do is *show* the answer where people look:
  - the test tab of GitHub, GitLab or Jenkins reads JUnit XML;
  - a pull request is read as one comment;
  - the full comparison report is several screens long, so it is too long for that comment.
- Every user who wants this has to write the same download-run-comment script.

## Decision

1. **`fwsight gate --junit FILE`** writes one JUnit `<testsuite>` with one `<testcase>` per Gate finding, using `classname="firmwaresight.gate"` and the rule id as the name. A finding maps by its factual state and its effective severity:

   | Finding | JUnit |
   |---|---|
   | `N/A` | `<skipped message="not applicable: …">` |
   | effective severity **BLOCK** (a BLOCK, or an UNKNOWN that policy blocks) | `<failure type="BLOCK" message="…">` |
   | effective severity **REVIEW** (a REVIEW, or an UNKNOWN that policy sends to review) | `<skipped message="needs review: …">` |
   | PASS | passed (no child) |

   - REVIEW is *skipped*, not failed. A review is a decision a person owes, not a broken test. The CI job still exits 4, so a pipeline that does not accept reviews still stops.
   - The suite carries `tests`, `failures`, `skipped` and `errors="0"`, plus the run id, the snapshot id and the overall verdict as `<properties>`.
   - **No timestamps, no host name, no paths.** The same run writes the same bytes.
   - Text is XML-escaped. A character that XML 1.0 forbids is written as its `\u{…}` code, never dropped.
2. **`--summary-md FILE`**, on `fwsight diff` and on `fwsight gate`, writes a short Markdown block for a PR comment or `$GITHUB_STEP_SUMMARY`:
   - **diff:**
     - Flash and RAM base → target and Δ, with comparability;
     - the largest five object movers from ADR-0039's growth waterfall, or the largest five section growths when the MAPs give no object attribution;
     - the warning codes.
   - **gate:**
     - the verdict with its glyph and word;
     - one line per non-PASS finding.
   - It starts with an HTML comment, `<!-- firmwaresight-summary -->`, so a script can find its own earlier comment and update it instead of adding another.
   - Names are cut to 120 characters, so the block stays far below GitHub's 65,536-character comment limit whatever the input.
   - Unknown is written as *unknown*, never 0. An absent side is written as *not present*.
3. **Both are export files, so they follow the existing export rules:**
   - they never replace an existing file (`ERR-EXPORT-6001`, exit 6);
   - two exports of one run that name the same file are refused before any work;
   - nothing changes on stdout, so the `--json` contract is unchanged.
4. **A composite GitHub Action at the repository root (`action.yml`).**
   - **Input:** a version, an artifact and its MAP, an optional baseline and its MAP, a project directory, and an optional PR comment.
   - **Get `fwsight`:** download the release's CLI archive for the runner's OS, then **verify its build provenance with `gh attestation verify --repo 2023violet/FirmwareSight`** (ADR-0037) before running it. A file that fails verification is never executed.
   - **Run:** `gate --junit --summary-md`, and `diff --summary-md` when a baseline is given.
   - **Report:**
     - append the summaries to `$GITHUB_STEP_SUMMARY`;
     - when `comment: true` on a pull request, create or update one comment found by the marker.
   - **Outputs:** the verdict, the exit code and the JUnit path.
   - **`fail-on: block | review | never`** decides whether the step fails.
   - A `fwsight-path` input runs a binary the caller built, and skips the download. This repository's self-test uses it.
5. **The product stays offline.**
   - Downloading, verifying and commenting are done by the Action's shell steps with the runner's `gh`. The `fwsight` binary opens no socket, as before (AGENTS.md §2).
   - The Action needs `pull-requests: write` only when `comment: true`. Without it, it needs no write permission.
6. **No new dependency:** the JUnit writer is hand-written XML in `firmwaresight-report`, and the summary reuses `markdown.rs`'s escaping.

## Consequences

- `04_TECH/07_CLI_SPEC.md` gains the two flags. `docs/ci/` gains the following, with a note on what each exit code means:
  - a GitHub Actions recipe;
  - a GitLab CI recipe;
  - a Git pre-push hook.
- The analysis, diff, gate and release JSON schemas are unchanged.
- The `p0-check.yml` workflow gains a self-test job:
  - it builds `fwsight`, then runs the Action against the `d4-armlink` fixtures through `fwsight-path`;
  - it checks the outputs, the JUnit file and the summary marker;
  - it does not comment.
- **Deferred:**
  - a GitLab merge-request comment (the recipe writes the JUnit report and the job log only);
  - SARIF output.

## Alternatives rejected

- **REVIEW as a JUnit failure:** this would make an accepted-by-policy review look like a regression in every test dashboard. The exit code already stops the pipeline.
- **A JavaScript Action:** it needs a bundled Node dependency tree to audit and keep current. A composite Action has only shell steps and `gh`, which runners already carry.
- **Comment from the binary:** this would give the product a network client, which AGENTS.md §2 forbids.
