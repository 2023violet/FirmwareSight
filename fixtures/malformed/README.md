# Malformed fixtures

Each file here is a regression input for the untrusted-intake path. None of them is
large; they are named for what they claim, not for what they are.

- `empty.bin` - zero bytes: rejected as malformed, no panic.
- `wrong-magic.bin` - ELF magic followed by prose: rejected as an unsupported format.
- `truncated-elf.bin` - a real ELF header cut short, so table offsets point past EOF.
- `sparse-elf-header.bin` - ELF magic plus a zero-filled header, so every declared
  size and offset reads as zero: rejected rather than trusted.

The >512 MiB guard workload is never committed; generate it with
`python scripts/gen_p0_workload.py`.
