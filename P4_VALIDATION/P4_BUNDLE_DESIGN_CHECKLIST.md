---
title: "P4 Release Bundle Design Checklist"
doc_id: "FS-P4-DESIGN"
product: "FirmwareSight"
version: "1.0"
status: "VALIDATED"
stage: "P4_RELEASE_BUNDLE"
owner: "Engineering"
last_updated: "2026-10-01"
---

# P4 Release Bundle — design checklist (prompt §63, AGENTS.md 11)

Every box below was settled by running the grep or the command it names, on the tree this pack
closes. The Bundle surface is an audit/export surface and was reviewed as one: dense lists, mono
identifiers, factual states, a quiet success line.

## Frozen assets

- [x] **No new design token.** `assets/design-tokens.json` still reads `meta.version 0.2.1`, and
  `git diff 6b7e23e HEAD -- assets/design-tokens.json assets/tokens.css` is empty: P4 added no token
  and changed none. `python scripts/check.py --only drift` re-derives `tokens.css` from the frozen
  file and reports `tokens.css matches the frozen token file.`
- [x] **No magic number in the new surface.** `apps/desktop/ui/src/Release.module.css` opens with the
  rule that every length, colour, radius and duration is a token, and a grep for `shadow` in that
  file returns nothing — the bundle screen casts no panel or table shadow.
- [x] **Banned vocabulary absent.** `grep -rniE "gradient|glassmorphism|backdrop-filter|purple|neon|
  confetti|celebrat|ready to ship|hero"` over `Release.tsx` and `Release.module.css` returns nothing.
  The success state is a plain `<h3>Bundle created</h3>` over a definition list — no exclamation, no
  celebration copy, no marketing hero.

## The five states and the mono rule

- [x] **States render as icon + label, never colour alone.** The Bundle surface moves only the
  aggregate the Gate computed; the exported report was read live in a browser and carries all five
  labels — `["PASS","REVIEW","N/A","UNKNOWN","BLOCK"]` — as the report layer renders them.
- [x] **UNKNOWN stays neutral.** The Release page reuses the frozen `StateBadge`; UNKNOWN keeps the
  neutral grey and hollow icon the tokens define, and the bundle's acceptance path offers no control
  for it (P3's rule, unchanged).
- [x] **Hashes, ids, versions, paths are mono.** `styles['mono']` appears 30 times in `Release.tsx`:
  the release id, both snapshot ids, the gate run id, every digest in the preview table, the proposed
  folder name, the notes path and the diagnostics id all render in the mono face.

## Audit-surface behaviour

- [x] **Dense file list.** The preview is one table of ten rows in Core's bundle-path order — file,
  role, size, SHA-256 — with no per-row expansion and no sorting of its own; the order is the plan's.
- [x] **Clear status, quiet failure.** Every bundle failure renders the shared `ErrorPanel` with
  *What happened / Code / What to do / Diagnostics ID*, and the confirmation block states the
  consequence in plain sentences: "Replacing it moves that bundle aside and removes it only once the
  new one is written and verified; keeping it writes nothing."
- [x] **No path crosses the surface.** The destination dialog returns a token; the page shows the
  proposed folder *name*, never the chosen root, and the smoke's host-path sweep over the page and
  over every composed document found none.
- [x] **Readiness is not certification.** `Release.tsx`'s header states that the Gate answers one
  policy and the wording stops there, and the exported report closes with the heading
  `9. Known capability limits of this release` rather than any shipping claim.

## Navigation and scope

- [x] **No fifth navigation verb.** The rail assertion in `apps/desktop/ui/src/release.test.tsx`
  still pins exactly Analyze / Compare / Release; `Bundle` is a section under Release (§48, §58), and
  the test's word-ban list names the verbs a fifth page would introduce, not the word this stage
  earned.
- [x] **UI re-implements nothing.** The preview table, the digests and the result block are all
  rendered from DTOs the engine produced; `Release.tsx` contains no hash, diff or gate logic — the
  same boundary P1–P3 held.
- [x] **DESIGN.md read before the first pixel.** The section follows `DESIGN.md`'s audit-surface
  pattern (definition lists, `rows`/`row`/`term`/`value` classes) rather than inventing layout, and
  `templates/DESIGN_CHECKLIST_TEMPLATE.md`'s items are the boxes above.
