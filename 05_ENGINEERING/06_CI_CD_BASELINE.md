---
title: "CI/CD Baseline v0.4"
doc_id: "FS-ENG-007"
product: "FirmwareSight"
version: "0.5.1"
status: "BASELINE"
owner: "Engineering"
last_updated: "2026-10-03"
---

# CI/CD Baseline

## Event matrix

| Event | Windows | Linux | macOS | Packaging |
|---|---|---|---|---|
| Pull Request | required core + UI | required core | optional/cost-controlled smoke | no |
| main push | required | required | **required core smoke** | no |
| nightly | required | required | required | optional package smoke |
| release tag | required | required | required | **all declared release targets** |

这解决 v0.3.0 中“PR macOS optional”与“main required macOS”未说明事件边界的问题。

## PR checks

Rust:
- fmt
- clippy
- workspace tests
- fixture/golden tests
- schema validation

Frontend:
- frozen pnpm install
- TS typecheck
- lint/test/build
- generated IPC type clean check
- UI design checklist mechanical subset when available

## Security/dependencies

- cargo-deny
- dependency update bot
- license/source/advisory policy
- no automatic major merge

## Release

Protected tag/manual approval:
1. version/tag validation
2. full test
3. build matrix
4. code signing/notarization where required
5. bundle/installers
6. package smoke
7. checksums
8. draft release
9. updater artifacts only after updater ADR conditions are active

## Secrets

Signing/updater/notarization secrets never exposed to untrusted fork PRs.

## P5 package jobs (2026-10-03)

The event matrix above said `main push → packaging: no`, and it was accurate for every stage before
P5: prompt section 41 makes package production part of this stage's authoritative CI, and this
repository has no scheduled workflow at all, so the "nightly → optional package smoke" row that the
same matrix offers as packaging's home has never run and cannot run. A requirement to produce install
readiness with no CI that builds an installer resolves to documentation about a build nobody has
made, so P5 adds three jobs to a push on `main`:

| # | Job | Runner | Produces |
| --- | --- | --- | --- |
| 8 | Package Windows | `windows-latest` | NSIS installer + CLI companion zip |
| 9 | Package Ubuntu | `ubuntu-latest` | `.deb` + CLI companion tar.gz |
| 10 | Package macOS | `macos-latest` | `.app` + `.dmg` + CLI companion tar.gz |

Jobs 1-7 are unchanged and still authoritative. The authoritative set is therefore ten jobs for the
length of P5, and `P5_VALIDATION/P5_CI_AUTHORITY.md` records the exact set and each job's run before
closure is claimed - prompt section 41 requires that file if the set differs from ten, and this
repository's rule is that a job count never drifts silently.

What the package jobs do and do not do:

- Each runs `python scripts/check.py --only package`, the same command a developer runs, so the
  workflow still cannot be green where the local command is red. The group builds the CLI companion
  and the platform package, then `scripts/verify_package_artifacts.py` verifies the packaged version
  against `[workspace.package] version`, verifies the frontend is embedded in the binary that
  shipped, and writes the distribution `SHA256SUMS.txt` and `artifact-metadata.json`.
- They upload to the workflow run with `if-no-files-found: error` and a 14-day retention. Nothing is
  published: no GitHub Release, no tag, no signing, no notarization, no updater artifacts.
- Pull requests do not package. They keep the seven correctness jobs, which is the cost control the
  matrix above was written for.
- `tauri-cli@2.12.1` is installed pinned and `--locked`, the same way this workflow installs
  `cargo-deny@0.20.2`. It is a CI tool, not a dependency: it is absent from `Cargo.lock`, so
  `cargo-deny`'s view of the product graph does not change.
- The Linux WebView prerequisite list, which used to be copied into two jobs, now lives once in
  `.github/actions/linux-tauri-prereqs` and is used by `rust`, `drift` and `package-ubuntu`. Its
  provenance - the two runs that proved each line necessary - moved into the action file with it.

