---
title: "R4 Design Checklist"
doc_id: "FS-EV-R4-001"
product: "FirmwareSight"
status: "SELF_REVIEW"
owner: "Engineering"
last_updated: "2026-10-17"
---

# R4 design checklist

Template: `templates/DESIGN_CHECKLIST_TEMPLATE.md`.

Scope: what R4 touches.
- Release (`Release.tsx`);
- the new `Sheet`;
- the R1 components as R4 extends them: BuildPicker's none entry, `SubjectBuild.map` made optional, LeadAnswer's `label` and `focusRef`, and two Gate freshness words.

Evidence:
- source reading, plus a mechanical grep of `git diff 465fe5f` over the UI sources;
- 501 component and page tests;
- the native smoke in [README.md](README.md).

## Don'ts

| Item | Result | Basis |
|---|---|---|
| No gradient, glass, glow or neon | PASS | The diff adds no `gradient`, `backdrop-filter` or `text-shadow`. |
| No purple; blue only as accent | PASS | No literal colour was added. |
| No shadow on panels or tables | PASS | The only added `box-shadow` is `overlay-sm` on the Sheet, which is an overlay. |
| No marketing surface | PASS | The verdict is a badge, one sentence, the state strip and a standing note on a hairline. |
| State is never colour alone | PASS | StateBadge everywhere. Growth deltas are neutral ▲▼ with spoken words. |
| Dual theme | PASS | Semantic tokens only. Dark was checked natively (R12). |
| No invented values | PASS | The diff adds no px, colour or font-size literal. |
| UI does not re-implement Core | PASS | Every state, count, margin, delta and aggregate is the shell's. The verdict sentence counts rows by Core's own `effectiveSeverity` field. No request changed (D-R4-1). |
| Banned words | PASS | No "safe to ship", "certified", "compliant" or "production ready". The tests pin this. |

## Do's

| Item | Result | Basis |
|---|---|---|
| One answer first | PASS | The verdict leads, with the not-a-certification note. |
| Unknown: neutral, with a reason; never 0 | PASS | An unmeasured growth delta reads "Unknown", never "not present". A missing budget figure reads Unknown. An UNKNOWN finding is classed Unknown in the Inspector. Each has a mutation-checked test. |
| One primary action per state | PASS | Run Gate. In the Bundle view, Prepare (disabled with its reason unless PASS). |
| Numbers, hashes and paths in mono | PASS | The settings line, the run record, the tables' numeric columns and the picker values. |
| Tables have semantic headers | PASS | DataTable: `caption`, `th scope`, row headers. Budget and growth rows open the Inspector. |
| Esc closes overlays; focus returns | PASS | The Sheet traps Tab, closes on Esc or a scrim click, and returns focus to its opener (foundation tests; smoke R08). The Inspector does the same (R04, R05). |
| Key figures have context | PASS | The verdict carries the state strip. The settings line names the build, baseline, policy and attachments the next run will judge. |
| Errors have their four parts | PASS | ErrorPanel is unchanged. A refused export (6002) is the stale note on every path. |
| Window minimum 1024×720, checked by a human drag | NOT_VERIFIED | Carried from R1 A10. |
| Screen reader pass | NOT_VERIFIED | Belongs to R7. |
