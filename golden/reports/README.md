# `golden/reports/`

Reserved by `05_ENGINEERING/08_CODE_AREA_PLAN.md` section 6 for golden output of the report and
Release Bundle layer.

It is intentionally empty in P0. The only deterministic output the P0 slice produces is the
`analyze` payload, and its goldens already live in `golden/cli/` (whole document) and
`golden/core/` (the memory projection). A portable report schema is a later-phase contract -
`04_TECH/26_PORTABLE_SCHEMA_POLICY.md` keeps `release-manifest v1` unpublished until the Gate and
Bundle phases exist - so a "report golden" written now would either duplicate `golden/cli/` or
freeze a shape that has no owner.

`golden/reports/` gets its first file when a real report renderer exists, together with a test
that reads it.
