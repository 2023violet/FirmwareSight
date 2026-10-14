# NEXT-47 engineering integration evidence

Code integration: **PASS** on `76b5c2c006f1ca0850af86da6d1e4b957227a025`.
Native user acceptance: **NOT_VERIFIED / BLOCKED_BY_ENV**.
[Actions #113](https://github.com/2023violet/FirmwareSight/actions/runs/38595778036), attempt 1, proves all ten Jobs and Check Runs on that exact code SHA. This repository upload is a later documentation/evidence commit; run #113 does not certify its new commit SHA. Read the current Git/Actions state for the upload commit.

- [Integrated report](evidence/08_NEXT47_FINAL_REPORT.md) / [JSON](evidence/09_NEXT47_FINAL_REPORT.json)
- [Independent code review](evidence/02_NEXT47_CODE_REVIEW.md)
- [Original 106 items with scoped integration decisions](evidence/03_NEXT47_AC_MATRIX_106.tsv) / [JSON](evidence/03_NEXT47_AC_MATRIX_106.json)
- [Deferred native QA risk register](evidence/04_NEXT47_DEFERRED_NATIVE_QA.md)
- [Exact code SHA CI proof](evidence/07_NEXT47_PUSH_CI_PROOF.json)
- [Independent post-push review](evidence/reviewer/POST_PUSH_CI_REVIEW.md)
- [Public evidence archive](FirmwareSight_NEXT47_PUBLIC_EVIDENCE_20261014.zip) / [archive digest](PUBLIC_ARCHIVE_SHA256.json)
- [Published file integrity and original provenance](PUBLIC_EVIDENCE_MANIFEST.json)

## Public copy and original identity

The Owner explicitly requested uploading this delivery to main after the engineering integration completed. The local original evidence archive remains unchanged and is not uploaded. This directory is a public repository derivative: local user and protected-checkout prefixes become `LOCAL_USER` and `OWNER_WORKSPACE`; execution helpers, protected staged patches and original-only package manifests are omitted. Test outputs, failures, correction history and compatibility/native limitations remain visible. The public manifest records every original payload hash, current published payload hash, redaction and omission.

Original signature/log hashes embedded in reports identify the retained local originals. A redacted file intentionally has a different hash; it must not be described as a byte-exact original or a newly signed verdict. Published binary fixtures/images/PDFs remain byte-exact. The archive is verified against the published manifest. Git attributes prevent evidence-byte conversion on checkout. Whitespace checks do not reformat retained raw evidence logs; generated repository documentation uses LF.

The historical NEXT-46 blocked verdict and original AC statuses are preserved. NEXT44-AC24, NEXT44-AC26 and NEXT45-AC20 remain MUST / NOT_VERIFIED with Owner-accepted deferred native QA. They must close before the earliest external Developer Preview distribution, Private Beta or RC. Keil/IAR compiler cohorts remain UNVERIFIED; ArmLink/IAR MAP remains NOT_SUPPORTED / FIXTURE_BLOCKED. This upload grants no installer/native acceptance, public product release, signing or license change.
