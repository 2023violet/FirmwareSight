---
title: "Pilot Moderator Run Sheet"
doc_id: "FS-PL-005"
product: "FirmwareSight"
version: "0.6.0"
status: "MATERIAL_NOT_USED"
owner: "Research"
last_updated: "2026-10-17"
---

# Pilot session run sheet (about 60 minutes)

**Not used yet.** This is the session-day checklist for [PILOT_PROTOCOL.md](PILOT_PROTOCOL.md). The protocol wins if the two differ.

## Before the day

- [ ] The register shows: final protocol approval, the build commit and installer SHA-256, the Pilot minimum QA passed (`evidence/D2C-R/HUMAN_QA_PACK.md` §5), and this participant's pool decision.
- [ ] Consent received (V1 items + Pilot sentence); recording consent separately if recording.
- [ ] The installer, its SHA-256, and the unsigned-build disclosure are ready. Nothing else is sent: no fixture, no document about the product.
- [ ] A session record copied from `sessions/TEMPLATE.md` to the external evidence root, not into Git.

## During (the moderator reads the task wording from the protocol, word for word)

| Minutes | Step | Record |
|---|---|---|
| 0–5 | Welcome; confirm consent; confirm they may run their own build here; start recording if consented | consent time, recording mode |
| 5–10 | Install from the shared installer; verify its SHA-256 together | install friction, SmartScreen or AV prompts, time |
| 10–20 | **PT1** First look | first action, path, MAP added unprompted, first fact read aloud |
| 20–30 | **PT2** Growth source | reached object/library rows; trust in shares and "not attributed" |
| 30–38 | **PT3** Two builds (if two builds) | how base and target were chosen; did the object changes answer it |
| 38–45 | **PT3b** Reference build (if two builds) | pin found unprompted; where they looked; time to first pin; next Compare on the pinned build; unpin understood |
| 45–52 | **PT4** Hand-off | format chosen; where it would go; if Markdown, opened before sending and remarks |
| 52–60 | Debrief questions 1–6, in order, after all tasks | quotes verbatim, in quotation marks |

Any hint is an intervention: log what was said and why. Never name a page, button, file type or feature, and never say "pin", "baseline", "Markdown" or "export" before the participant does.

## Stop rules

- Stop at once and file a defect (without participant data) on: any write to the participant's files without a confirmed Replace, a crash, a path or name shown where it should not be, or a keyboard trap.
- Stop if consent is withdrawn; keep nothing from that point.

## After

- [ ] Uninstall offered; the participant's files untouched.
- [ ] The session record completed the same day; `NOT_RECORDED` where nothing was noted.
- [ ] The scan in `CONSENT_PRIVACY.md` before any `git add` of the anonymized record.
- [ ] Register updated: counters, the session row, `IN_COHORT` / `OUT_OF_COHORT`.
