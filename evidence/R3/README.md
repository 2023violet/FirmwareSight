# R3 Compare workbench: native smoke

Plan: [06_DELIVERY/17_R3_COMPARE.md](../../06_DELIVERY/17_R3_COMPARE.md) §4.

- **Build:** the QA-identifier build (`com.firmwaresight.desktop.qa`) of the final R3 code. Binary SHA-256 `2a05bbac154697139940d6306cfb38077f6b13bafd0cab7b9fb90f4f4301e75f`.
- **Operator:** Claude, as an automated driver on the Owner's Windows 10 host (zh-CN, WebView2).
  - Page input went through the WebView2 DevTools protocol.
  - The real native Open and Save dialogs were driven with Win32 messages, against the QA process only.
  - Window sizes were set with `MoveWindow`.
- **Input:** `fixtures/elf/m2-objects/{base,target}`, each an ELF plus its MAP, analyzed and then compared.

**Result:** 11 PASS, 1 NOT_RUN. See [AC_MATRIX.tsv](AC_MATRIX.tsv). The run covers:
- both languages, both themes, and 1440 and 1100 px;
- the Save dialog's Cancel path.

**Fixed during the run.** The smoke run exposed two defects. Both were fixed before this final run:
- The lead named `.symtab ▲96` as the largest growth source. It now names the largest **object** growth when the MAP attributes one, and only otherwise falls back to Core's section ranking.
- Next to the docked Inspector, table cells wrapped in the middle of a value (".symt / ab", "432 / bytes"). Figures now stay on one line, names break only between words, and the table scrolls inside its wrapper.

**Observation for Core (not changed in R3).** Core's top section growth ranks `.symtab` first: 432 → 528 in both its file size and its memory size. The ranking is meant to use runtime size, and `.symtab` is not loaded at run time. Whether a non-allocated section should carry a memory size, or take part in that ranking, is a Core and ADR-0021 question. The UI does not re-rank (D-R3-1).

**QA stores.** Each run's store is kept and none is deleted (Standing Owner rule: no deletion outside an approved scope):
- `com.firmwaresight.desktop.qa.r3-kept-234723`
- `com.firmwaresight.desktop.qa.r3-kept-235428`
- `com.firmwaresight.desktop.qa.r3-kept-000136` (final run)

The Owner's store was only hashed.

**Evidence ZIP.** `R3_NATIVE_SMOKE_EVIDENCE.zip` holds eight screenshots, `results.tsv` and the four driver scripts. Local paths were stripped before packing. SHA-256 `c6610b052ea91d3afee415a3eb9c01919a4743aa9c02c1930791b93c25ba7739`.
