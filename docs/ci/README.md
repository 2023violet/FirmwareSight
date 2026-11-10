# FirmwareSight in CI

FirmwareSight runs in your pipeline the same way it runs on your machine:
- **Local:** the binary reads the build and its MAP, and writes files. It opens no network connection.
- **The wrapper:** only the wrapper (the GitHub Action below, or your own script) downloads the CLI and posts comments.

Background: ADR-0040 (`09_ADR/ADR-0040-ci-integration-junit-summary-action.md`), and the "CI exports" section of `04_TECH/07_CLI_SPEC.md`.

## What to read: the exit code

| Exit | Meaning | Typical CI handling |
|---:|---|---|
| `0` | Gate **PASS** | pass |
| `4` | Gate **REVIEW**: a person owes a decision | fail, or allow with `fail-on: block` |
| `5` | Gate **BLOCK** | fail |
| `2` | usage or `firmwaresight.toml` error | fail (fix the job) |
| `3` | the artifact or MAP could not be read | fail (fix the build) |
| `6` | an export could not be written (for example, the file already exists) | fail (use a fresh path) |

Never decide by parsing the text output: the exit code is the contract.

## Files for people

- `fwsight gate … --junit gate.xml`: one test case per Gate rule.
  - BLOCK is a failure.
  - REVIEW is skipped as "needs review".
  - N/A is skipped as "not applicable".
  - GitHub (through a JUnit reporter action), GitLab (`artifacts:reports:junit`) and Jenkins (`junit`) all read it.
- `fwsight gate … --summary-md gate.md` and `fwsight diff … --summary-md size.md`: short Markdown blocks for a PR comment or `$GITHUB_STEP_SUMMARY`. Each starts with `<!-- firmwaresight-summary -->`.

Exports never replace an existing file, so write them to a fresh directory on each run.

## The Action's PR comment

- **One per pull request.** The Action updates its own earlier comment, the one by a bot that starts with the marker. A second invocation in the same PR (for example a matrix of targets) updates that same comment.
- **Fork pull requests:** the token is read-only, so no comment is posted. The step prints a warning, and the job summary still shows the result. `fail-on` alone decides whether the job fails.

## Recipes

- [`github-actions.yml`](github-actions.yml): the composite Action with a baseline from the target branch, the job summary and one PR comment.
- [`gitlab-ci.yml`](gitlab-ci.yml): the CLI in a GitLab job, with the JUnit report shown in the merge request.
- [`pre-push`](pre-push): a Git hook that runs the Gate before every push.
