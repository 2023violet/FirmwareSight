---
title: "ADR-0037: Release provenance signing"
doc_id: "FS-ADR-0037"
product: "FirmwareSight"
version: "0.7.0"
status: "BASELINE"
owner: "Project Lead"
last_updated: "2026-10-18"
---

# ADR-0037: Sign release files with keyless build provenance; platform code signing waits for Owner credentials

## Status

**BASELINE** (2026-10-18). The Owner asked for code signing in chat on 2026-10-18. AGENTS.md §2 and §9 require an ADR for any signing or key change.

## Context

- v0.6.0 shipped unsigned binaries with SHA-256 digests. A digest proves that a file matches the list; it does not prove who built the file or from which commit.
- **Platform code signing** needs credentials that only the Owner can obtain:
  - Windows Authenticode needs a code-signing certificate from a CA (OV or EV), which is a purchase and an identity check;
  - macOS signing and notarization need an Apple Developer Program membership.
  An agent must not buy these, create these accounts or handle the private keys.
- **GitHub artifact attestations** (`actions/attest-build-provenance`) sign an SLSA build-provenance statement for each file. They are keyless: Sigstore issues a short-lived certificate for the workflow's OIDC identity. They need no secret, no purchase and no key management.

## Decision

1. **Now: build-provenance attestations on every package from `main`.**
   - Each package job (`package-windows`, `package-ubuntu`, `package-macos`) attests its distribution files (`.exe`/`.zip`, `.deb`/`.tar.gz`, `.dmg`/`.tar.gz`).
   - The step runs only on a push to `main`. Those are the runs a release is cut from.
   - The jobs get only `id-token: write` and `attestations: write`, on top of `contents: read`.
   - Anyone can verify a file with `gh attestation verify <file> --repo 2023violet/FirmwareSight`. The check shows the repository, the workflow and the commit that built the file.
2. **Later, when the Owner provides credentials:**
   - Windows Authenticode, from a certificate in a CI secret or a cloud HSM;
   - macOS Developer ID signing and notarization.
   Each one needs its own amendment that records where the key lives and who can use it.
3. **Unchanged:** no updater (AGENTS.md §7); the SHA-256 list stays in each release.

## Consequences

- Release notes and SECURITY.md say plainly that the files carry signed provenance but are **not platform-signed**. SmartScreen and Gatekeeper still warn.
- An attestation proves where a file came from, not that the code is safe.
- The attestations are public in the repository's Sigstore transparency log. That is acceptable, because the repository is public.
- GitHub issues these attestations to public repositories, or to private ones on Enterprise Cloud only. In a private fork the step fails.
- Only the distribution files named in Decision 1, from `main` CI runs, are attested. `SHA256SUMS.txt` and the licence files attached to a release are not.

## Alternatives rejected

- **A self-signed Authenticode certificate:** SmartScreen does not trust it, and it adds a private key to manage while proving less than an attestation.
- **Long-lived GPG keys for the checksum file:** this is key management with no gain over keyless attestations.
