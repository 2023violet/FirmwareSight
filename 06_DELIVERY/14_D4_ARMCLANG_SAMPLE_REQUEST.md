---
title: "D4 Sample Request — ArmClang / armlink"
doc_id: "FS-DEL-015"
product: "FirmwareSight"
version: "0.6.0"
status: "REQUEST"
owner: "Product / Delivery"
last_updated: "2026-10-16"
---

# D4 sample request — ArmClang / armlink

Prepared in D2C (C6) so that the slow part of D4, getting real samples, can start now. Standing Owner rule 3 applies: **no Keil or IAR support claim without real vendor samples.** Today armlink MAP files are `NOT_SUPPORTED / FIXTURE_BLOCKED` (`04_TECH/39`).

## What one usable sample is

A **same-origin set**, produced by one build of one project, unedited:

| File | Why |
|---|---|
| `.axf` (ELF image) | ELF read and memory accounting |
| armlink `--map` output with `--list`, ideally also `--info=sizes,totals` | the MAP grammar the adapter must read; the size totals are the cross-check |
| scatter file, or the linker options that set regions | region names, bases and lengths |
| the exact compiler and linker versions (`armclang --version`, `armlink --version`) and the build command line | provenance and grammar version |
| one object file and its source, or permission to name which object holds a known symbol | an independent check of object attribution |

Wanted beyond that:
- a second build of the same project, after a real change, for Compare;
- a build that links a static library (`.lib` / `.a`), for library attribution;
- one build from the Keil MDK IDE and one from a plain command line, if both are used.

## Provenance and privacy

- Prefer a **public or synthetic** project built with a licensed toolchain: vendor examples, a CMSIS pack example, or a small project written for the purpose. Its files can then be committed as fixtures, with a generator script where possible (as in `scripts/gen_m2_object_fixtures.py`).
- A participant's or customer's real build is used **only locally**, with permission, and never committed. It can confirm an adapter, but cannot be the fixture.
- Every committed MAP is scanned for host paths and user names before commit, as the M2 fixtures were.

## How D4 will use it

1. Write an ADR for the armlink adapter. The grammar, rows, charges and unknowns follow the D1–D8 pattern of `ADR-0033`.
2. Add the fixtures, plus a test that the adapter's totals equal armlink's own `--info=totals`.
3. Report support **per layer** in `04_TECH/39`: ELF read, memory accounting, object attribution and Compare, each with its fixture. One layer can be supported while another is not.

## Open questions for the Owner

1. Is a licensed Arm Compiler 6 / Keil MDK available to produce a synthetic sample, or must samples come from someone else?
2. Which Cortex-M family and which RTOS should the first sample represent?
