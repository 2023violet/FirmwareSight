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

Both are regenerated only by an explicit, reviewed step:

    python scripts/update_goldens.py             # prints the semantic diff, writes nothing
    python scripts/update_goldens.py --confirm   # writes it

A failing test never rewrites its own golden (`05_ENGINEERING/02_TEST_STRATEGY.md`), and the script
compares JSON goldens by content rather than by bytes so that a serializer's key order can never
reformat another stage's evidence.
