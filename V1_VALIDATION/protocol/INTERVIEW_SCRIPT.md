---
title: "V1 Interview Script"
doc_id: "FS-V1-014"
product: "FirmwareSight"
version: "0.6.0"
status: "EXECUTION_RECORD"
owner: "Product / Research"
last_updated: "2026-10-06"
---

# V1 Interview Script

Continues `V0_VALIDATION/protocol/INTERVIEW_SCRIPT.md` and the baseline question bank
`06_DELIVERY/09_USER_INTERVIEW_QUESTION_BANK.md`, with two changes: the Gate comprehension rubric is now a
scored metric (M4), and the commercial section is explicitly secondary (V0's price-anchor work was withdrawn
before it ran, and §22 forbids inventing anchors now).

Ask everything here **after** the tasks, never before.

## M4 — Gate meaning comprehension rubric (§19)

Neutral framing first: "I'd like to ask what a few of the words in the tool mean to you — there is no product
vocabulary test here, I want to know what you would conclude."

Present the five states the participant encountered and ask, for each, what it would let them conclude and what
it would not. Comprehension **passes** only with all five:

| | The distinction the participant must make |
|---|---|
| A | **PASS** is a policy/evidence result, *not* universal release approval |
| B | **UNKNOWN** is insufficient or unresolved evidence, *not* "clean" |
| C | **REVIEW** is accountable human judgment, *not* automatic proof |
| D | **BLOCK** prevents qualification |
| E | **N/A** means the rule does not apply in this context |

Record per-concept misses, not just the pass/fail, and report them in the aggregate. If the participant
expresses that PASS certifies legal, compliance, security or safety matters, code it **Critical** (C1/M13) and
flag it for Interim Review the same day.

Follow-up, still neutral: "Suppose a colleague said 'the tool says it can ship'. What would you ask them?"

## M5 — Real new information (§20)

"Of what you saw in your own build tonight, was there anything you did not know, had not noticed, or would
otherwise have gone and checked separately?"

Then: "Which specific thing was it, and how do you know it is about your build rather than the tool's
example?" `NEW_INFORMATION = YES` requires a **concrete fact about their own build**. Generic praise
("this looks useful") is recorded as an observation, not as M5. Note whether they would have found it another
way, and how long that would have taken.

## M6 — Return intent (§21)

Ask in these words and code only `YES` / `MAYBE` / `NO`:

> "Thinking about a real release/review like the one you work on, would you intentionally open FirmwareSight
> again for a future release?"

Follow-ups that do not change the code: "what would the trigger be?", "who else would need to see it?",
"what would have to be true for the answer to be YES instead of MAYBE?"

## Current workflow and value

- How do you do these checks today, in the last week before a release?
- Which step costs the most time, and which step carries the most risk?
- What did this replace tonight, precisely? What would you go back to without it?
- Which parts felt merely convenient, and which felt like they lowered release risk?

## Handoff and export

- Who else consumes this analysis, and what format do they actually open?
- Would a self-contained file be more useful than a link here? Why?
- (No promise that any export exists: V1 may recommend export formats and may not implement or imply them.)

## History and repetition

- How many times do you repeat these checks in one release cycle, and what event triggers a re-check?
- Recall a time you needed a build's evidence months later. How did you find it?
- Would an automatic watch save you anything, or create noise?

## Evidence limits and toolchain expectation

- Which source is the dependency-version authority in your project?
- Did anything look wrong, thin, or unexplained in the evidence you were shown? (Ask before explaining what
  `Unknown` means, or the answer is contaminated.)
- Which toolchain and linker do you actually use, and does the tool cover your case as you understand it?

## Support, failure and trust

- If the tool behaved unexpectedly and you needed support, what would you look for first?
- What would make you distrust a number it gave you about your own build?
- What would you need to see to allow this on a machine that builds release firmware?

## Commercial and pilot signal — secondary only (§22)

Qualitative questions only. No pricing model, no paid plans, no invented anchors, and no claim that any anchor
is a frozen product price.

- Who owns tool approvals like this on your team, and how does one get bought?
- Would you run one real release through it as a pilot if that were possible? What would need to be true?
- Is there any external pressure today (customer review, regulation, an audit) that touches release evidence?
  Ask what it actually requires; do **not** ask "do you need FirmwareSight for CRA compliance".

If no Architect-provided anchors exist, the price question is not asked at all.

## Debrief

Anything they want to ask, answered **after** all coding is done. Then record: verbatim quotes separated from
interpretation, product findings classified by §31, participant recommendations, and the moderator's
interpretation in its own clearly-labelled section.
