---
title: "P0 Fixture Register"
doc_id: "FS-P0-005"
product: "FirmwareSight"
version: "0.5.1"
status: "EXECUTION_RECORD"
owner: "Engineering"
last_updated: "2026-09-28"
---

# P0 Fixture Register

Requirement: the tests must run on artifacts that are real linker output, whose provenance is
recorded, and whose identity is verified before any assertion trusts them.

Status: **LOCAL PASS**

## Two compiled fixtures, built from the same source

| | Fixture A `p0-basic` | Fixture B `p0-dual-region` |
| --- | --- | --- |
| `firmware.elf` | 7,288 bytes, `1c0ae94e611497c1635c117834132f23b36799ad651852ba878be421ba20557f` | 10,960 bytes, `c6d0feed0a1e4153d80592519f0bfe1a11be67d756f2ecd811799b4f7def4e62` |
| `firmware.map` | not provided (by design) | 3,149 bytes committed, `c4182c5bcc69b155e6a9ccc641990a6d6d8649ebb6b57e3ea63a3fd6f0ce2a61` |
| Linker script | default single-region | `p0-dual-region.ld`, `f4bf022bb856294251776f60a3a5ab8f0c02e478a3e12baf580db8672ba17cc1` |
| What it proves | detection, sections, symbols, identity, debug exclusion, capability degradation | ROM/RAM region table, LMA-in-ROM with VMA-in-RAM, dual accounting at `MapRegionAndElfLoad` |
| Architecture | ARM cortex-m4, Thumb, 32-bit little-endian | same |

Both compile the identical `source/main.c` (1,059 bytes). That is the point: the only difference
between the two analyses is the load evidence available, so any difference in the reported
claims is attributable to evidence and not to content.

## Toolchain provenance (recorded in each `fixture.toml`)

- `arm-none-eabi-gcc 14.3.1 20250623` (Arm GNU Toolchain 14.3.Rel1, build arm-14.174)
- `GNU ld (Arm GNU Toolchain 14.3.Rel1) 2.44.0.20250616`
- Command, exactly as recorded:

```
arm-none-eabi-gcc -g -Os -mcpu=cortex-m4 -mthumb -c source/main.c -o build/main.o &&
arm-none-eabi-gcc -g -Os -mcpu=cortex-m4 -mthumb -nostdlib -nostartfiles -e main build/main.o
    -T p0-dual-region.ld -Wl,-Map=firmware.map -o firmware.elf
```

Regeneration: `python scripts/gen_p0_fixtures.py` (refuses to overwrite without `--force`).
Neither the ARM toolchain nor any compiler is required to run the test suite - the binaries are
committed precisely so that CI tests the parser rather than the toolchain.

## Reproducibility check performed on 2026-09-28

`python scripts/gen_p0_fixtures.py --force` was run and then compared against git.

- `firmware.elf` for both fixtures: **byte-identical**
- `firmware.map`: **byte-identical to the working copy** - a weaker statement than it sounds, and
  the reason the next section's defect survived. That working copy held CRLF, the regeneration
  reproduced CRLF, so the comparison agreed while the committed blob was LF.
- `fixture.toml` / `manifest.json`: changed, for two legitimate reasons - the recorded date rolled
  forward, and the recorded command line was corrected (it had been written as
  `-p0-dual-region.ld` where the toolchain was actually invoked with `-T p0-dual-region.ld`).
  The provenance text had been wrong; the build was not.

So the same toolchain, source and linker script reproduce the same artifact bytes, and the only
defect surfaced was in the record rather than in the artifact.

## What a checkout is allowed to do to a fixture (added 2026-09-28)

Remote CI Run #1 failed `committed_fixtures_match_their_recorded_hashes` on Windows for
`p0-dual-region.ld`. The same class of failure was already latent for `firmware.map` on Linux, hidden
behind the Ubuntu job that never reached the tests. One bug, two directions, and neither end is a
fixture defect:

| File | Committed blob | What a checkout produced | The recorded hash described |
| --- | --- | --- | --- |
| `p0-dual-region.ld` | LF `f4bf022b…` | CRLF `7b872afd…` on a Windows `core.autocrlf=true` checkout | the blob, so the checkout was wrong |
| `firmware.map` | LF `c4182c5b…` | LF on Linux, CRLF `b2be5f19…` on Windows | the CRLF working copy, so the record was wrong |

`.gitattributes` carried rules for `.elf` and `.bin` but none for `.ld`, `.map`, `.c` or `.h`, while
its own header comment asserted the MAP files were covered. They were not. Git only abstains from
line-ending conversion for files it has classified, so an unclassified text file is at the mercy of
whoever cloned it.

The repair is the text policy plus one record correction; the assertions did not move:

- `*.map -text` and `*.ld text eol=lf`, plus `text eol=lf` for the other source and configuration
  types the repository tracks, so a checkout yields the committed bytes on every platform;
- `fixtures/manifest.json` records the `.map` as `c4182c5b…`, the SHA-256 of the committed blob,
  confirmed by reading `HEAD:fixtures/elf/p0-dual-region/firmware.map` rather than by re-hashing a
  working copy;
- no newline normalization was added to the parse or test path, and the hash-first assertion still
  compares raw bytes;
- the committed `.map` bytes did not move at all: this commit changes the record and the checkout
  policy, and `git diff` for `fixtures/elf/p0-dual-region/firmware.map` is empty. Only the local
  working copy, smudged by an earlier checkout, was rewritten back to the blob it came from.

While fixing the record, one more register defect surfaced: `fixtures/malformed/`'s fourth file is
`sparse-elf-header.bin` (20 bytes), not the `oversize-sparse.bin` this document used to name.

Verified with fresh clones instead of argument:

```text
before the fix: clone with core.autocrlf=true  -> p0-dual-region.ld = 7b872afd…  (1 of 10 disagree)
before the fix: clone with core.autocrlf=false -> firmware.map      = c4182c5b…  (1 of 10 disagree)
after the fix:  both clones                                            10 of 10 agree
```

## Malformed inputs

`fixtures/malformed/` holds four deliberately broken files, each with a stated purpose in
`README.md`: `empty.bin` (0 bytes), `wrong-magic.bin` (18 bytes), `truncated-elf.bin` (64 bytes
of a real ELF header), `sparse-elf-header.bin` (20 bytes of ELF magic over a zero-filled header).
They are inputs to the no-panic regression tests, not examples of supported formats.

## Synthetic performance workloads - not firmware

`fixtures/generated/` is gitignored and produced by `scripts/gen_p0_workload.py`. Each file is
the first bytes of fixture B's ELF followed by zero padding, so the header parses and the volume
is unreasonable - the shape a size guard must survive.

| File | Bytes | SHA-256 |
| --- | --- | --- |
| `guard-100mib.elf` | 104,857,600 | `f587dab97242d14e79b8981f4f7c6464a2b386cb166ddfb57d2fc502a5d044b3` |
| `guard-256mib.elf` | 268,435,456 | `8a6a910ecece27061249b0817a2e1662e43cbc1c8da1bf7ccf278acc3e58bc4c` |
| `guard-512mib.elf` | 536,870,912 | `56611d6aaa735b7ab82ccc70dd73989b14c19358af0286156874543a64d0d44c` |
| `guard-over-512mib.elf` | 536,871,936 | `23f55e760d7e391deaa18c3a79a964331ab20de28ca7528c7c571d14dc7f9914` |

**Synthetic Performance Workload.** Nothing about them is a claim that such firmware exists, and
no measurement taken from them is presented as a real firmware size.

## Hash-first rule

`p0_acceptance.rs` recomputes each SHA-256 and compares against `fixtures/manifest.json` before
any other assertion runs, so a corrupted or substituted fixture fails with a message that names
the hash rather than producing a misleading parse failure later.

```
$ RUSTUP_TOOLCHAIN=stable cargo test -p firmwaresight-artifact
test result: ok. 19 passed; 0 failed     # tests/p0_acceptance.rs
test result: ok.  4 passed; 0 failed     # unit tests
```
