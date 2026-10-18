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

Release binaries are currently **not code-signed**. Windows SmartScreen and macOS Gatekeeper will warn about them. Verify a download against the SHA-256 published with the release, or build from source (see [CONTRIBUTING.md](CONTRIBUTING.md)).
