# Security policy

## Scope

FirmwareSight reads files you point it at (ELF images, linker MAP files, project configuration) and writes reports and release bundles where you ask. It does not open network connections, has no telemetry and no auto-updater, and stores its history in a local SQLite database.

Issues we treat as security issues include:
- a crafted ELF, MAP, configuration or bundle that crashes the parser, hangs it, or makes it read or write outside the paths the user chose;
- a report or export that executes script, loads a remote resource, or leaks a local path, user name or host name it should not;
- terminal control sequences from user files reaching the terminal unescaped;
- a release bundle whose integrity check can be bypassed.

## Supported versions

Only the latest release on `main` receives fixes.

## Reporting a vulnerability

Please report privately through GitHub's **Report a vulnerability** button (Security › Advisories) on this repository. Do not open a public issue for a vulnerability.

Include:
- the version;
- the input that triggers the issue (synthetic, never proprietary firmware);
- what happens and what you expected.

You will receive an acknowledgement, and the advisory is published together with the fix.

## Binaries

Since v0.7.0, each installer and CLI archive in a release (built by CI on `main`) carries a signed **build-provenance attestation** (Sigstore, keyless; [ADR-0037](09_ADR/ADR-0037-release-provenance-signing.md)). Check that a download was built by this repository's workflow, and from which commit:

```bash
gh attestation verify FirmwareSight-0.7.0-windows-x86_64-nsis.exe --repo 2023violet/FirmwareSight
```

The binaries are **not platform code-signed** (no Windows Authenticode, no Apple notarization), so Windows SmartScreen and macOS Gatekeeper will still warn. You can also check a download against the SHA-256 published with the release, or build from source (see [CONTRIBUTING.md](CONTRIBUTING.md)).
