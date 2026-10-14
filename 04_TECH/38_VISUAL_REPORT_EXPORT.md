# Visual report export

Analyze, Compare and Release offer PNG, SVG and offline HTML through native Save dialogs. PNG is a 1600 × 900 share image. SVG carries the same scene as editable vector markup. Exports use the light design baseline regardless of the desktop theme. Exported headings are English; safe Chinese artifact/region names remain readable with local font support.

The HTML overview is separate from complete source evidence. Its exact-value tables and limitations remain available below the image; use the original analysis JSON, detailed Compare HTML or verified Release Bundle for all engineering facts. Open HTML in a browser and use **Print → Save as PDF**, selecting A4 paper. FirmwareSight does not generate native PDF files. Detailed Compare and Release HTML also include an overview and A4 pagination styles.

## CLI and reproducible examples

```powershell
cargo run --bin fwsight -- analyze fixtures/elf/p0-dual-region/firmware.elf --map fixtures/elf/p0-dual-region/firmware.map --png analyze.png --svg analyze.svg --report-html analyze.html
cargo run --bin fwsight -- diff fixtures/elf/p2-diff/base/firmware.elf fixtures/elf/p2-diff/target/firmware.elf --png compare.png --svg compare.svg --report-html compare.html
cargo run -p firmwaresight-report --example visual_samples -- <new-output-directory>
```

The sample generator reads synthetic Core facts only and produces known Analyze, mixed/Unknown Compare, ranked growth, Gate REVIEW/BLOCK/UNKNOWN, long Chinese labels, private-path canaries and large-integer cohorts. It refuses to replace existing files. CLI images are written only to explicitly selected paths; `--json` still writes the original JSON to stdout. Existing `diff --html` remains the detailed report; `--report-html` is the visual overview.

Desktop cancellation writes nothing. A denied overwrite retains the existing file. Choose a writable directory and retry after a write error. Analyze export is available only for a matching successful selection; Compare is bound to the displayed diff; Gate/Release summaries retain their own run and plan identities. Standalone images are separate from the Release Bundle file set.

## Reading the evidence

- FLASH/nonvolatile footprint and RAM/runtime footprint are different Core facts. Unknown stays Unknown; observed zero is displayed as zero.
- Region bars show mapped interval occupancy against the region's declared capacity, only when Core reports exact accounting and positive capacity. They do not establish footprint utilisation, allocator headroom or stack/heap safety.
- Compare follows A → B and shows Core's signed byte delta and comparability. Its Top 5 changed-symbol growth comes from Core; bar lengths are relative to the largest listed contributor. Added/Removed counts do not imply a measured zero baseline. Section/symbol sizes do not add up to reliable FLASH/RAM attribution.
- Gate findings retain PASS / REVIEW / BLOCK / UNKNOWN / N/A with a glyph and word. Original severity and disposition after recorded acceptances are separate. PASS is a policy result, not shipping permission or security certification. SHA-256 checks byte integrity, not publisher identity or build provenance.
- Identity labels distinguish Snapshot ID, observed artifact SHA-256, FirmwareSight version, Gate Run ID and real Release ID/version. Workspace Git observations do not prove the firmware build source.

## Privacy and portability limits

Labels are bounded and projected for display, never written back to source configuration or storage. Control/bidi characters, markup and host-path patterns are withheld. Overview labels may be ellipsized; numeric values stay exact decimal integers. Full source evidence remains in the original JSON; a visual export intentionally does not carry raw release notes or config contents.

SVG and HTML contain static markup with no scripts, network references, external fonts or user CSS. PNG rasterizes that SVG offline through resvg 0.48.1 with text/system-font support only. Fonts are read locally and not redistributed. Identical inputs in the same font environment produce identical bytes; PNG font fallback and hashes can differ between hosts. A missing font environment causes an explicit error rather than a substituted preview. HTML/PDF appearance depends on browser pagination and installed fonts.

See [third-party renderer notices](../07_COMPLIANCE/THIRD_PARTY_VISUAL_REPORT_NOTICES.md) for the exact arrayref 0.3.9 license admission and redistribution duties. Public distribution, FirmwareSight licensing, signing and installed desktop QA are separate decisions.

## NEXT-46 boundary addendum — 2026-10-14

CLI visual and legacy detailed HTML export failures use a shared output boundary: exit 6, `ERR-EXPORT-6001`, operation ID and recovery advice remain inspectable; destination names and raw OS errors are not emitted. Only FirmwareSight's typed operation tracing reaches CLI stderr, so dependency font-path diagnostics cannot bypass the boundary even under `RUST_LOG=trace`. A visual destination is never replaced implicitly; the legacy `diff --html` replacement behavior is unchanged.

Rounded-zero decorative region/ranking bars are omitted. Exact positive integer bytes remain in the row/table; no 1-pixel minimum or fabricated percentage is introduced. Independent PNG decode, SVG XML checks and browser A4 samples are separate evidence from native desktop Save-dialog interaction. Native dialog/keyboard/theme runtime acceptance remains NOT_VERIFIED unless independently observed; code and DOM tests do not establish it.

Late export outcomes are scoped to the displayed analysis selection/snapshot, comparison handle or Gate run/snapshot. Changing that subject clears the old export state; an older asynchronous response cannot report success or error under the new subject. Dedicated delayed-response UI regressions cover all three surfaces. Existing native-side source/destination rechecks remain unchanged.

NEXT-45's dated engineering/CI record remains intact. Its corrected Windows nonduplicated `cargo test --workspace` total is 1,154 passed, 0 failed, 2 opt-in ignored tests, 54 result lines. The historical commit message's 1,448/68 totals counted overlapping runs and are not amended. Owner's NEXT-46 adjudication is `ENGINEERING_DELIVERED / REMOTE_CI_PROVEN / REVIEW_EXCEPTIONS_OPEN`; the earlier push with reviewer `ACCEPT_WITH_LIMITS` is retained as a conditional-push governance nonconformance. New candidate acceptance and push require the fresh exact-SHA independent matrix.
