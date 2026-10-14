# Fresh exact-SHA visual inspection

Candidate `76b5c2c006f1ca0850af86da6d1e4b957227a025`; all artifacts were generated in NEXT47 by the reviewer, not copied from NEXT46 results. Existing NEXT46 scripts were explicitly adapted outside Git as starting points; `adaptation-provenance.md` records that distinction.

All six fresh 1600×900 PNG images were directly viewed: Analyze known, Compare ranked growth, Compare Unknown, Release REVIEW/BLOCK/UNKNOWN, privacy/CJK/long label, and large-u64/CJK region. Integer values, signed deltas, Unknown labels, safe withheld text and evidence limitations were legible in this host's font environment. No host canary was visible and no original private input was substituted. This establishes these artifact renderings, not native desktop UI.

Fresh offline HTML files were opened in isolated headless Edge contexts, with no HTTP requests or script errors. SVG text bounds were measured against the 1600×900 scene. Actual A4 PDFs were generated from all six summary HTML files, the newly generated detailed Compare, the exact-SHA frozen release golden and a newly generated real CLI Release bundle. Independent pypdf checks confirm A4 boxes, nonblank last pages and every source table-cell text across all pages. Summary files are 2 pages each; detailed Compare is 16; frozen Release 12; the new synthetic CLI Release is 10 pages with 194 table cells covered.

Poppler rasterized every page of the newly generated detailed Compare and CLI Release PDFs. Both complete contact sheets were directly viewed; table headers repeat and all final pages contain substantive evidence/limitations. The Release REVIEW/BLOCK/UNKNOWN full HTML screenshot was viewed, including its detailed Unknown/Review rows and stated limitations. Summary cards explicitly name their row limits and refer to detailed evidence rather than implying all findings appear in seven rows.

Pillow 12.3.0 independently decoded all six PNGs, checked actual dimensions, white background, substantial nonwhite pixels and variation. ElementTree independently parsed SVG XML and checked static tags/attributes/references, positive rectangle widths and absence of privacy canaries/NaN/Infinity. Two generations produced identical PNG/SVG/HTML bytes on this host. Cross-host font/raster byte equality was not tested or claimed.

These are artifact/HTML/browser-print observations. No native Windows Save dialog, keyboard/focus/accessibility/theme/minimum-window session, installed application, installer or native PDF feature was observed. C1–C3 remain NOT_VERIFIED.
