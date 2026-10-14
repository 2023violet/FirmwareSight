# Clang ELF metadata fixtures

Four genuine Clang 22.1.8 relocatable objects, produced from the synthetic `fixture.c` by the exact relative commands in `provenance.json`. Targets cover ARM ELF32 LE/BE and AArch64 ELF64 LE/BE. Independent llvm-readobj header/section/symbol logs are kept in the external delivery evidence. No private firmware, vendor IDE or commercial toolchain was used.

Regenerate into **new** directories with `python scripts/gen_interop_fixtures.py <fixture-directory> <external-evidence-directory>`. Compare binary SHA-256 values with the committed provenance. Object bytes are deterministic for this recorded compiler/environment. These fixtures prove metadata parsing; they are not linked memory-layout or Keil/IAR compatibility evidence. They must not be relabelled as vendor-generated `.axf`/`.out` files.
