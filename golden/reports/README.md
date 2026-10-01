# `golden/reports/`

Holds the golden output of the report layer, the directory `05_ENGINEERING/08_CODE_AREA_PLAN.md`
section 6 reserves for it.

P2 filled the placeholder: `p2-diff.html` is the complete self-contained HTML export of
`fwsight diff` over the dedicated fixture pair `fixtures/elf/p2-diff/{base,target}`, both sides
MAP-backed. It is compared **byte for byte** by `apps/cli/tests/p2_golden.rs`, because byte
stability is the property under test: the export carries no timestamp, no script and no remote
reference, so the same two builds must produce the same file forever.

Its JSON counterpart is `golden/core/p2-diff.json` - the whole portable diff document, not an
excerpt, compared by content the way the P0 analyze goldens are.

## `p4-release/` - one whole release bundle

`golden/reports/p4-release/` holds the eight documents one release writes
(`apps/cli/tests/p4_golden.rs` compares all eight **byte for byte**), produced by the shipped
`fwsight release prepare` over the release subject `fixtures/project/p4-release` and the same P2 pair:

    accepted-reviews.json  analysis.json  diff.json  gate-results.json
    release-manifest.json  release-notes.md  release-report.html  SHA256SUMS

Two of the ten files a bundle holds are deliberately **not** here: `artifacts/firmware.elf` and
`artifacts/firmware.map`. Their bytes are already committed as fixtures and already hashed in
`fixtures/manifest.json`, so duplicating 22 KiB of image into `golden/` would add a copy that can
disagree with the original. `the_shipped_artifact_bytes_are_the_committed_fixtures` is what closes
that gap: the copies inside a freshly published bundle are hashed and checked against the fixture
record.

The JSON documents are compared by bytes rather than by content, unlike the P0 and P2 goldens,
because a digest was taken over exactly these bytes. A re-indented or re-ordered golden would no
longer be the file `SHA256SUMS` describes, and the mismatch it should report is the one it would hide.

The subject's single commit has a pinned author, committer, both dates and message
(`release@example.invalid`, `2026-09-30T07:00:00+00:00`, `"release candidate"`, tag `v1.2.3`), recorded
in `fixtures/project/p4-release/fixture.toml`. Without that, the workspace hash would differ on every
machine, and the release id - which is derived from the build, the policy and the version - would
follow it, taking every digest in the bundle with it.

Both this folder and the P2 pair above are regenerated only by an explicit, reviewed step:

    python scripts/update_goldens.py             # prints the semantic diff, writes nothing
    python scripts/update_goldens.py --confirm   # writes it

A failing test never rewrites its own golden (`05_ENGINEERING/02_TEST_STRATEGY.md`), and the script
compares the older JSON goldens by content rather than by bytes so that a serializer's key order can
never reformat another stage's evidence.

Nothing in `p4-release/` needs FirmwareSight to read. That is the point of the bundle, and
`python scripts/verify_bundle_portability.py <bundle>` is what says so out loud: standard library plus
`jsonschema`, no first-party code in the path.
