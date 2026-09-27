# Malformed fixtures

Each file here is a regression input for the untrusted-intake path.

- `empty.bin` - zero bytes: rejected as malformed, no panic.
- `wrong-magic.bin` - valid-looking prefix, wrong overall format.
- `truncated-elf.bin` - a real ELF header cut short, so table offsets point past EOF.
- `oversize-sparse.bin` - a generator seed, not the oversize workload itself.

The >512 MiB guard workload is never committed. Generate it with
`python scripts/gen_perf_workloads.py`.
