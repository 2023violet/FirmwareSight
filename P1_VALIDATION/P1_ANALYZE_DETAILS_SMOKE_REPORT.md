---
title: "P1 Analyze Details Desktop Smoke"
doc_id: "FS-P1-004"
product: "FirmwareSight"
version: "0.1.0"
status: "EXECUTION_RECORD"
owner: "Engineering"
last_updated: "2026-09-29"
---

# P1 Analyze Details Desktop Smoke

A real window, real native dialogs, real clicks, on the binary this commit ships. This is the only
place in the P1 pack that reports what a person would see; the automated results live in
`P1_ANALYZE_EXECUTION_REPORT.md` §7.

## What was run

| Item | Value |
| --- | --- |
| Binary | `target/release/firmwaresight-desktop.exe`, 11,207,168 bytes, sha256 `82de3bbadebac5f8…`, built 17:01 |
| Build command | `cargo build --release --features custom-protocol -p firmwaresight-desktop`, after `corepack pnpm build` in `apps/desktop/ui` |
| Bundled assets | `dist/assets/index-CcxqINNK.css` 13.48 kB, `dist/assets/index-DAjTMXP2.js` 246.50 kB |
| Window | id `466734`, pid `2944`, title `FirmwareSight - Analyze`, outer 1056 x 799, client 1015 x 768 |
| Document origin | `http://tauri.localhost/` (packaged custom protocol, not a dev server) |
| Window width vs contract | 1015 px client, i.e. the smoke ran at the frozen `layout.desktop_min.width = 1024` size, not a comfortable one |
| Inputs | `%LOCALAPPDATA%\Temp\p1-details-smoke\ota-image.elf` (10,960 B, sha256 `c6d0feed0a1e4153d80592519f0bfe1a11be67d756f2ecd811799b4f7def4e62`), `ota-image.map` (3,149 B, `c4182c5bcc69b155e6a…`), `truncated.elf` (64 B, `178728395c4245af…`) |
| Fixture hashes | re-checked with `sha256sum` in the same shell session before the run |
| Driver | Qoder computer-use MCP: `launch_app`, `click` at window-relative coordinates, `type_text`, `press_key`, `scroll`, `get_window_state` |

Three binaries were driven during this round, and the order matters for reading the findings:

1. the binary built at 15:37 (`dist/assets/index-Ui9zGfZD.css` / `index-BwJz3iHb.js`, window `10164920`,
   pid `26580`) - the functional pass, and where the filter anomaly in §6 was seen;
2. `ae1120c48f55965a…` built at 16:39 (window `1056562`, pid `21696`) - the first build carrying the
   layout fixes, where the `Inspect` column was still off the right edge;
3. `82de3bbadebac5f8…` built at 17:01 (window `466734`, pid `2944`) - the shipped binary, everything below.

## 3. Intake through native dialogs

| Step | Observed |
| --- | --- |
| Fresh launch | `Analyze` heading, `Choose firmware artifact`, `Analyze` disabled, and `Nothing has been analyzed in this session yet. Choose an artifact and run Analyze.` |
| `Choose firmware artifact` | dialog window titled **`Choose the firmware artifact to analyze`** - the use-case wording, not a generic open panel; filter `All Files (*.*)` |
| Path typed, submitted | selection line becomes `ota-image.elf   MAP: Not provided`, `Add MAP` and `Analyze` enable |
| `Add MAP` | dialog window titled **`Choose the linker MAP for this artifact`** |
| MAP submitted | `ota-image.elf   MAP: ota-image.map`, and the buttons become `Replace MAP` / `Remove MAP` |
| `Analyze` | summary renders; no path appears anywhere on screen - only `ota-image.elf` and `ota-image.map` |

The dialog's starting directory was the executable's own folder (`D:\study\Software\FirmwareSight\target\release`).
That is the shell's default, not a location the WebView named.

## 4. Summary against the CLI, and the unit switch

Field by field, the same facts the CLI golden reports:

```text
File      ota-image.elf
SHA-256   c6d0feed0a1e4153d80592519f0bfe1a11be67d756f2ecd811799b4f7def4e62   (equals sha256sum output)
Size      10,960 bytes
Format    Arm 32-bit little
Kind      elf          Parser  object-elf/0.40
Entry     0x080000039  Build ID  -   no .note.gnu.build-id section is present
Nonvolatile / load image   Exact 160 bytes
Runtime RAM                Exact  72 bytes
Load evidence   Admissible for a hard limit - weakest basis map-memory-configuration+elf-load; region evidence came from the linker
Layout source   map        Accounting rule adr-0021-dual-budget
Dual-accounted  elf.section_header[3], elf.section_header[5]   2
Device metadata excluded   1,975 bytes
Capabilities  ELF supported | Sections available | Symbols available | Debug info available |
              MAP provided | Object attribution available | Git unknown
Counts        Sections 17 | Symbols 32 | Evidence 11 recorded - 11 observed, 0 derived, 0 declared, 0 unknown
Snapshot      snap-c6d0feed0a1e4153d80592519f0bfe1a11be67d756f2ecd811799b4f7def4e62-p0-normalize-1-c4182c5bcc69b155e6a9ccc641990a6d6d8649ebb6b57e3ea63a3fd6f0ce2a61
              firmwaresight.analyze/p0-internal-1 · p0-internal · normalization p0-normalize-1 · fwsight 0.1.0
```

The snapshot id is the artifact hash **and** the MAP hash joined through the normalization version, so
the companion MAP is sealed into the identity rather than merely displayed next to it.

After choosing `KiB`, every byte figure re-labelled and nothing else moved:

| Cell | Bytes | KiB seen | Independent check |
| --- | --- | --- | --- |
| Artifact size | 10,960 | `10.7 KiB` | 10960/1024 = 10.703125 |
| Nonvolatile | 160 | `0.156 KiB` | 0.15625 |
| Runtime RAM | 72 | `0.0703 KiB` | 0.0703125 |
| Excluded metadata | 1,975 | `1.93 KiB` | 1.92871 |
| `.text` file / memory size | 60 | `0.0586 KiB` | 0.05859375 |
| `.data` | 4 | `0.00391 KiB` | 0.00390625 - a nonzero count never rendered as zero |
| `.symtab` contributor | 528 | `0.516 KiB` | 0.515625 |
| `.debug_abbrev` contributor | 166 | `0.162 KiB` | 0.162109 |

Untouched by the switch, in the same screen: `Entry 0x080000039`, every virtual address, load address
and file offset, the SHA-256, the snapshot id, and the counts `17`, `32`, `11 recorded`. The Evidence
Inspector's `Value` (`ROM@0x080000000+0x100000,…`) also stayed verbatim: a recorded raw value is not a
byte count.

The switch issued no request. The snapshot id line was identical before and after, no `Analyzing…`
state appeared, and the evidence table kept its rows - so nothing re-parsed, no snapshot was created
and no SQLite row was written for a unit change.

## 5. The detail area

`Largest stored payload` (top contributors) rendered one row per section, name and role on the left and
the byte count right-aligned:

```text
.symtab       SymbolTable   528 bytes
.debug_info   Debug         294 bytes
.debug_str    Debug         275 bytes
.shstrtab     StringTable   175 bytes
.debug_abbrev Debug         166 bytes
```

with the hint that these are stored bytes and not what a device budget charges. `.symtab` leading the
list is host metadata by size, which is exactly why the sentence is there.

Sections tab, first rows as stored:

```text
1 .text       Code              A-X  0x080000000 0x080000000 0x00001000 60 bytes  60 bytes  ROM
2 .rodata     ReadOnlyData      A--  0x08000003c 0x08000003c 0x0000103c 32 bytes  32 bytes  ROM
3 .data       InitializedData   AW-  0x200000000 0x08000005c 0x00002000  4 bytes   4 bytes  RAM
4 .bss        UninitializedData AW-  0x200000004 0x080000060 Unknown     0 bytes   4 bytes  RAM
5 .ota        InitializedData   AW-  0x200000008 0x080000060 0x00002008 64 bytes  64 bytes  RAM
6 .debug_info Debug             ---  Unknown       0x000000000 0x00002048 294 bytes 294 bytes *default*
```

`.bss` has no file offset and the cell says `Unknown` rather than `0x00000000`. `.debug_info`'s virtual
address says `Unknown` followed by its own reason: `this section is host metadata and is not loaded at
runtime`. Neither cell was produced by the front end guessing.

Symbols tab, filtered by `task` then by `sensor`, then sorted by `Size`:

```text
filter task   ->  26  mqtt_task     0x08000001  32 bytes  Function  Global  Index(1)   Showing 1 to 1 of 1
filter sensor ->  30  sensor_fifo   0x08000002d 10 bytes  Function  Global  Index(1)   Showing 1 to 1 of 1
sort Size asc ->   27  g_scratch …  4 bytes | 29 main … 4 bytes | 31 g_threshold … 4 bytes      (ordinal tiebreak)
sort Size desc ->  23 staging_area 64 bytes | 26 mqtt_task 32 bytes | 28 cfg_table 32 bytes
```

Scrolling to the tail of the descending order shows every unknown-sized row after the known ones, each
with `Unknown  the symbol entry records size 0`, and mapping symbols (`$t`, `$d`) rendering `Unknown`
for kind rather than a guessed one. `main.c` renders as kind `File` with section `Unknown`. Unknown
stayed last in **both** directions, which is the contract the storage layer implements.

Evidence tab, `Inspect` in the first column and the inspector opening above the table:

```text
Field            Classification  Value                     Source
architecture     observed        Arm                       elf.e_machine
bitness          observed        Bits32                    elf.e_ident[EI_CLASS]
byte_size        observed        10960                     stat:ota-image.elf
dual_accounted   observed        nonvolatile=4 runtime=4   elf.section_header[3] + map:load-address
dual_accounted   observed        nonvolatile=64 runtime=64 elf.section_header[5] + map:load-address
endianness       observed        Little                    elf.e_ident[EI_DATA]
entry_point      observed        0x080000039               elf.e_entry
memory_regions   observed        ROM@0x080000000+0x100000, RAM@0x200000000+0x40000, *default*@0x0+0xffffffff   map:Memory Configuration
…
Pager: Showing 1 to 11 of 11 - Previous page disabled, Next page disabled
```

The `+ map:load-address` suffix on the two dual-accounted rows is the basis-aware provenance from the
P1-A0 closure: those rows name the MAP because the accounting basis genuinely came from it.

Inspecting `memory_regions`:

```text
memory_regions
Classification  observed
Source          MapFile
Locator         map:Memory Configuration
Value           ROM@0x080000000+0x100000,RAM@0x200000000+0x40000,*default*@0x0+0xffffffff
Rule            gnu_ld
Confidence      -
Evidence id     ev-map-regions
```

Labels and values sit in two aligned columns with no overlap, the locator is shown in full rather than
truncated, and `Confidence` renders `-` because there is genuinely no value rather than a zero.
No host path appears in any locator: `file:ota-image.elf` and `stat:ota-image.elf` are file names.

## 6. The filter anomaly, and what was established

On the 15:37 binary, one `Apply filter` click after typing into the box produced no change, twice in a
row. On the shipped binary the same sequence works, and it works in the automated reproducer. What is
known:

- the WebView2 accessibility tree carried a `状态 建议可用` ("suggestions available") autofill node in
  every capture of that session;
- after `Escape` dismissed it, `task` -> Apply -> clear -> Apply -> `sensor` -> Apply all behaved, and
  the same sequence is asserted by `clearing the filter and applying again asks for the whole table`;
- the shipped binary no longer shows that node (23 and 25 nodes across captures, none of them the
  autofill status), after `autoComplete="off"` was added.

What is **not** established: which of the two candidate mechanisms produced the dead click - the
autofill popup swallowing the press, or a click landing during the frame in which the panel is
unmounted while a request is in flight. The autofill change removes one possibility; it is not proof
it was the cause. Recorded as unresolved rather than written up as fixed.

## 7. Failure and last-good binding, on the shipped binary

`truncated.elf` is the first 64 bytes of `ota-image.elf`: valid `\x7fELF` magic, 32-bit little-endian,
machine `0x28`, entry `0x08000039`, and a section-header offset that runs past the file.

| Item | Observed |
| --- | --- |
| Selection line | `truncated.elf   MAP: Not provided` - a new candidate does not inherit the previous MAP |
| Error panel | `Analysis failed` / What happened: `Could not parse artifact as ELF.` / Code: `ERR-PARSE-2002` / Why we know: `artifact parsing failed: object could not parse the ELF container: Invalid ELF program header size or alignment` / What to do: `Choose the linker ELF output rather than a stripped or truncated copy.` / Diagnostics ID: `op-b80-18d9c0115ce61070` |
| Stale banner | `Previous analysis of ota-image.elf. It is not an analysis of truncated.elf.` |
| Summary | still `ota-image.elf`, still `c6d0feed…`, still `10.7 KiB` |
| Detail area | still on the last-good snapshot: the contributor list, the counts and the open inspector all kept their content |
| After switching to Sections | the query succeeded against the old snapshot and painted `.text 0.0586 KiB`, `.rodata 0.0313 KiB`, `.data 0.00391 KiB` - no error panel, no repointing at the failed candidate |
| Process | no panic, no restart; the window stayed usable throughout |

The diagnostics id is the operation id, so this failure is greppable in the database by a reader who
wants the stored record.

## 8. Close

The title-bar close button ended the session: `tasklist //FI "IMAGENAME eq firmwaresight-desktop.exe"`
returned `信息: 没有运行的任务匹配指定标准。` ("no running tasks match the specified criteria") on both
occasions the window was closed. No orphan process, no leftover WebView2 process holding the profile.

## 9. Not observed, not claimed

- **No macOS or Linux window was driven.** This smoke is Windows-only, on the machine that built the
  binary. The cross-platform claim stays with CI, and CI for the shipping commit is recorded in a
  successor document, never inside this commit.
- **Keyboard-only traversal was not exercised in the real window.** The tab strip, radios, filter form
  and pager are native controls with correct roles, asserted in jsdom; that is not the same as having
  pressed Tab through the shipped binary.
- **Peak RSS was not measured** during this smoke: `NOT MEASURED`, unchanged.
- **No timing claim.** The durations in this document are wall-clock observations of a driver session,
  not a benchmark, and no user-perceived latency is asserted anywhere.
- **The 11 evidence rows are all `observed`** for this fixture. `derived`, `declared` and `unknown`
  rendering is covered by storage and desktop tests using labelled harness rows, not by this window.
