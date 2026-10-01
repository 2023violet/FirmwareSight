/**
 * P3 Release and P4's section under it: the screen a release owner uses to judge one stored build, read the
 * findings, sign a review off, and hand the result to a folder that does not need FirmwareSight to read.
 *
 * These tests protect eight claims, and every one is about what the screen is allowed to say:
 *
 * - the rail lists the three stages this build has, and no doorway to one it does not (prompt §43);
 * - a config dialog that was cancelled is not a failure, and a config that was refused keeps the project
 *   the page already had (prompt §40, §49);
 * - no host path and no raw config text reaches the screen, in either direction (prompt §40, §46, §53);
 * - a finding shows its rule, its state, its effective severity, its evidence and its next step, and the
 *   groups read BLOCK before REVIEW before UNKNOWN (prompt §47);
 * - UNKNOWN stays neutral, and an `UNKNOWN` whose disposition is `BLOCK` says both (ADR-0023, §47);
 * - only a REVIEW can be accepted, an acceptance needs a name and a reason, the row stays REVIEW, and
 *   nothing offers to edit or delete the record (prompt §48);
 * - and a run that failed leaves the last good one on screen, labelled as previous (prompt §49);
 * - and the bundle section stays a section of this page: it opens only on a PASS disposition, asks before
 *   it replaces anything, and shows the plan the engine rather than the page composed (prompt §48, §58).
 *
 * The bridge is mocked, the way the P1 and P2 files do it: the plumbing has its own test. No assertion
 * here checks a number the shell did not send, because the page computes none of them.
 */

import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { App } from './App';
import {
  acceptReview,
  analyzeSelection,
  attachMap,
  chooseBundleDestination,
  clearMap,
  compareSnapshots,
  exportCompareHtml,
  exportCompareJson,
  exportReleaseBundle,
  getGateRun,
  listCompareCandidates,
  openProjectConfig,
  prepareReleaseBundle,
  queryEvidence,
  querySectionChanges,
  querySections,
  querySymbolChanges,
  querySymbols,
  runReleaseGate,
  saveProjectPolicy,
  selectArtifact,
} from './ipc/bridge';
import type { IpcOutcome } from './ipc/bridge';
import type {
  AcceptReviewOutcomeDto,
  BundleDestinationDto,
  BundleExportDto,
  BundleFileRowDto,
  BundlePreviewDto,
  CandidatePageDto,
  CompareCandidateDto,
  ErrorEnvelopeDto,
  GateFindingRowDto,
  GateRunDto,
  ProjectContextDto,
  ProjectPolicyDto,
} from './ipc/types';

vi.mock('./ipc/bridge', () => ({
  selectArtifact: vi.fn(),
  attachMap: vi.fn(),
  clearMap: vi.fn(),
  analyzeSelection: vi.fn(),
  querySections: vi.fn(),
  querySymbols: vi.fn(),
  queryEvidence: vi.fn(),
  listCompareCandidates: vi.fn(),
  compareSnapshots: vi.fn(),
  querySectionChanges: vi.fn(),
  querySymbolChanges: vi.fn(),
  exportCompareJson: vi.fn(),
  exportCompareHtml: vi.fn(),
  openProjectConfig: vi.fn(),
  saveProjectPolicy: vi.fn(),
  runReleaseGate: vi.fn(),
  acceptReview: vi.fn(),
  getGateRun: vi.fn(),
  prepareReleaseBundle: vi.fn(),
  chooseBundleDestination: vi.fn(),
  exportReleaseBundle: vi.fn(),
}));

const candidatesMock = vi.mocked(listCompareCandidates);
const openConfigMock = vi.mocked(openProjectConfig);
const savePolicyMock = vi.mocked(saveProjectPolicy);
const runGateMock = vi.mocked(runReleaseGate);
const acceptMock = vi.mocked(acceptReview);
const getRunMock = vi.mocked(getGateRun);
const prepareBundleMock = vi.mocked(prepareReleaseBundle);
const chooseDestinationMock = vi.mocked(chooseBundleDestination);
const exportBundleMock = vi.mocked(exportReleaseBundle);

function ok<T>(value: T): IpcOutcome<T> {
  return { ok: true, value };
}

function fail<T>(envelope: ErrorEnvelopeDto): IpcOutcome<T> {
  return { ok: false, envelope };
}

const ENVELOPE = (code: string, message: string): ErrorEnvelopeDto => ({
  code,
  message,
  operationId: 'op-gate-1',
  details: null,
  remediation: 'Correct the project config and try again.',
});

const MALFORMED = ENVELOPE('ERR-CONFIG-7002', '`firmwaresight.toml` is not valid TOML: line 4.');
const SAVE_REFUSED = ENVELOPE(
  'ERR-CONFIG-7007',
  'refusing to save firmwaresight.toml: it holds keys this build does not understand (sbom.extra)',
);
const STORAGE_MISS = ENVELOPE('ERR-STORAGE-4005', 'No stored build matches that baseline.');
const NOT_A_REVIEW = ENVELOPE(
  'ERR-STORAGE-4008',
  'Only a REVIEW finding can be accepted; this finding is BLOCK.',
);
const STALE_CONTEXT = ENVELOPE(
  'ERR-BUNDLE-6102',
  'The release context changed after this plan was prepared, so the plan authorizes nothing.',
);
const DESTINATION_TAKEN = ENVELOPE(
  'ERR-BUNDLE-6106',
  'A folder of that name is already at the destination; replacing it needs an explicit confirmation.',
);
const NOT_A_BUNDLE = ENVELOPE(
  'ERR-BUNDLE-6107',
  'The folder at that destination is not a bundle this engine wrote, so it is not replaced.',
);

function emptyPage() {
  return { rows: [], total: 0, offset: 0, limit: 100, nextOffset: null };
}

/** A stored build, spelled out field by field like the generated contract requires. */
function candidate(overrides: Partial<CompareCandidateDto> = {}): CompareCandidateDto {
  const base: CompareCandidateDto = {
    buildId: 'build-target',
    snapshotId: 'snap-target',
    fileName: 'app.elf',
    sha256: 'b'.repeat(64),
    byteSize: 8192,
    architecture: 'Arm',
    importedAt: '2026-09-30T07:14:52Z',
    nonvolatile: { state: 'exact', bytes: 4096 },
    runtimeRam: { state: 'partial', bytes: 1024 },
  };
  return { ...base, ...overrides };
}

function candidatePage(rows: readonly CompareCandidateDto[]): CandidatePageDto {
  const page: CandidatePageDto = {
    rows: [...rows],
    total: rows.length,
    offset: 0,
    limit: 25,
    nextOffset: null,
  };
  return page;
}

function policy(overrides: Partial<ProjectPolicyDto> = {}): ProjectPolicyDto {
  const base: ProjectPolicyDto = {
    projectName: 'Controller v2',
    requiredArtifactKinds: ['elf', 'map'],
    flashBudget: 4096,
    ramBudget: 2048,
    requireCleanGit: true,
    requireReleaseNotes: true,
    releaseNotesPath: 'docs/RELEASE_NOTES.md',
    versionSource: 'git_tag',
    versionPattern: '^v(?<version>[0-9.]+)$',
    expectedVersion: '2.1.0',
    expectedCommit: null,
    flashGrowthReviewBytes: 256,
    ramGrowthReviewBytes: 128,
    unknownEvidenceReviewCount: 3,
    onUnknown: {
      gitClean: 'review',
      commitMatchesRelease: 'review',
      versionMatch: 'review',
      flashBudget: 'block',
      ramBudget: 'block',
      baselineGrowth: 'review',
      releaseNotes: 'review',
    },
  };
  return { ...base, ...overrides };
}

function project(overrides: Partial<ProjectContextDto> = {}): ProjectContextDto {
  const base: ProjectContextDto = {
    projectName: 'Controller v2',
    configFileName: 'firmwaresight.toml',
    configSchemaVersion: 1,
    policy: policy(),
    policySha256: 'e'.repeat(64),
    warnings: [],
    unknownKeys: [],
  };
  return { ...base, ...overrides };
}

function finding(overrides: Partial<GateFindingRowDto> = {}): GateFindingRowDto {
  const base: GateFindingRowDto = {
    id: 'f-git-clean',
    ruleId: 'git.clean',
    state: 'PASS',
    effectiveSeverity: 'PASS',
    summary: 'The workspace has no uncommitted changes.',
    evidenceRefs: ['git:status', 'git:head'],
    remediation: null,
    acceptable: false,
    acceptance: null,
  };
  return { ...base, ...overrides };
}

const GROWTH_FINDING = {
  id: 'f-growth',
  ruleId: 'diff.growth',
  state: 'REVIEW',
  effectiveSeverity: 'REVIEW',
  summary: 'Nonvolatile image grew by 512 B over the 256 B review threshold.',
  evidenceRefs: [
    'policy:diff.flash_growth_review_bytes=256',
    'diff:snap-base:flash:snap-target',
  ],
  remediation: 'Review the growth, or tighten the accounting.',
  acceptable: true,
} satisfies Partial<GateFindingRowDto>;

function gateRun(overrides: Partial<GateRunDto> = {}): GateRunDto {
  const base: GateRunDto = {
    runId: `gate-${'a'.repeat(64)}`,
    snapshotId: 'snap-target',
    baselineSnapshotId: 'snap-base',
    projectName: 'Controller v2',
    policySource: 'firmwaresight.toml',
    policySha256: 'e'.repeat(64),
    createdAt: '2026-09-30T07:20:11Z',
    overallEffectiveSeverity: 'BLOCK',
    dispositionEffectiveSeverity: 'BLOCK',
    counts: { pass: 1, review: 1, block: 1, unknown: 1, notApplicable: 1 },
    findings: [
      finding(),
      finding({
        id: 'f-flash',
        ruleId: 'memory.flash_budget',
        state: 'BLOCK',
        effectiveSeverity: 'BLOCK',
        summary: 'FLASH footprint 4608 B exceeds the 4096 B budget.',
        evidenceRefs: ['policy:memory.flash_budget=4096', 'evidence:ev-memory-flash'],
        remediation: 'Reduce the footprint, or raise the budget deliberately.',
      }),
      finding(GROWTH_FINDING),
      finding({
        id: 'f-notes',
        ruleId: 'release.notes',
        state: 'UNKNOWN',
        effectiveSeverity: 'BLOCK',
        summary:
          'Release Notes `docs/RELEASE_NOTES.md` could not be read: the file is locked by another program.',
        evidenceRefs: ['policy:release.require_release_notes', 'file:docs/RELEASE_NOTES.md'],
        remediation: 'Fix the file permissions or lock and rerun.',
      }),
      finding({
        id: 'f-commit',
        ruleId: 'release.commit_matches_expected',
        state: 'N/A',
        effectiveSeverity: 'PASS',
        summary: '`release.expected_commit` is not configured.',
        evidenceRefs: ['policy:release.expected_commit'],
      }),
    ],
    warnings: [],
    git: {
      available: true,
      headCommit: '1f2e3d4c5b6a7988',
      exactTag: 'v2.1.0',
      dirty: true,
      summary: 'Workspace dirty',
      reason: null,
    },
    policy: policy(),
    tables: {
      artifacts: [
        { kind: 'elf', required: true, present: true, sha256: 'b'.repeat(64), byteSize: 8192 },
        { kind: 'map', required: true, present: false, sha256: null, byteSize: null },
      ],
      budgets: [
        {
          side: 'flash',
          label: 'FLASH',
          ruleId: 'memory.flash_budget',
          state: 'BLOCK',
          effectiveSeverity: 'BLOCK',
          budgetBytes: 4096,
          actualBytes: 4608,
          headroomBytes: null,
          overBytes: 512,
          exact: true,
          admissible: true,
          basis: 'SectionHeaders',
          reason: null,
        },
        {
          side: 'ram',
          label: 'RAM',
          ruleId: 'memory.ram_budget',
          state: 'UNKNOWN',
          effectiveSeverity: 'BLOCK',
          budgetBytes: 2048,
          actualBytes: null,
          headroomBytes: null,
          overBytes: null,
          exact: false,
          admissible: false,
          basis: null,
          reason: 'the snapshot carries no complete RAM attribution',
        },
      ],
      growth: [
        {
          side: 'flash',
          label: 'FLASH',
          ruleId: 'diff.growth',
          state: 'REVIEW',
          effectiveSeverity: 'REVIEW',
          oldBytes: 4096,
          newBytes: 4608,
          deltaBytes: 512,
          thresholdBytes: 256,
          comparability: 'exact',
          reason: null,
        },
      ],
      notes: {
        required: true,
        state: 'UNKNOWN',
        effectiveSeverity: 'BLOCK',
        relativePath: 'docs/RELEASE_NOTES.md',
        present: true,
        sha256: 'c'.repeat(64),
        reason: 'the file is locked by another program',
      },
    },
    recordNote: null,
  };
  return { ...base, ...overrides };
}

/** A run whose only outstanding answer is one review, so accepting it can be seen to move the aggregate. */
function reviewOnlyRun(): GateRunDto {
  return gateRun({
    overallEffectiveSeverity: 'REVIEW',
    dispositionEffectiveSeverity: 'REVIEW',
    counts: { pass: 4, review: 1, block: 0, unknown: 0, notApplicable: 5 },
    findings: [
      finding(),
      finding({
        id: 'f-version',
        ruleId: 'release.version_matches_policy',
        state: 'PASS',
        effectiveSeverity: 'PASS',
        summary: 'Workspace tag v2.1.0 matches the configured pattern.',
      }),
      finding(GROWTH_FINDING),
      finding({
        id: 'f-notes',
        ruleId: 'release.notes',
        state: 'PASS',
        effectiveSeverity: 'PASS',
        summary: 'Release Notes `docs/RELEASE_NOTES.md` is present.',
        evidenceRefs: ['file:docs/RELEASE_NOTES.md'],
      }),
      finding({
        id: 'f-commit',
        ruleId: 'release.commit_matches_expected',
        state: 'N/A',
        effectiveSeverity: 'PASS',
        summary: '`release.expected_commit` is not configured.',
      }),
    ],
  });
}

/**
 * A run whose reviews are all accepted, so the disposition is PASS and the bundle section can act.
 *
 * The accepted row still reads REVIEW: a PASS disposition here is Core's aggregate counting an acceptance,
 * not the finding changing state, which is exactly the distinction §48 makes the screen preserve.
 */
function passingRun(overrides: Partial<GateRunDto> = {}): GateRunDto {
  return gateRun({
    overallEffectiveSeverity: 'REVIEW',
    dispositionEffectiveSeverity: 'PASS',
    counts: { pass: 4, review: 1, block: 0, unknown: 0, notApplicable: 1 },
    findings: [
      finding(),
      finding({
        id: 'f-flash',
        ruleId: 'memory.flash_budget',
        state: 'PASS',
        effectiveSeverity: 'PASS',
        summary: 'FLASH footprint 3840 B is inside the 4096 B budget.',
      }),
      finding({
        ...GROWTH_FINDING,
        acceptable: false,
        acceptance: {
          findingId: 'f-growth',
          actor: 'rosa',
          acceptedAt: '2026-09-30T08:02:00Z',
          reason: 'the growth is the new bootloader',
          originalState: 'REVIEW',
        },
      }),
      finding({
        id: 'f-notes',
        ruleId: 'release.notes',
        state: 'PASS',
        effectiveSeverity: 'PASS',
        summary: 'Release Notes `docs/RELEASE_NOTES.md` is present.',
        evidenceRefs: ['file:docs/RELEASE_NOTES.md'],
      }),
      finding({
        id: 'f-commit',
        ruleId: 'release.commit_matches_expected',
        state: 'N/A',
        effectiveSeverity: 'PASS',
        summary: '`release.expected_commit` is not configured.',
      }),
    ],
    ...overrides,
  });
}

const BUNDLE_FOLDER = 'Controller-v2-2.1.0-7f3a91c2';

function bundleFile(overrides: Partial<BundleFileRowDto> = {}): BundleFileRowDto {
  const base: BundleFileRowDto = {
    path: 'analysis.json',
    role: 'analysis',
    byteSize: 8192,
    sha256: 'a'.repeat(64),
  };
  return { ...base, ...overrides };
}

/** The plan's own list, in the order the manifest records it, so a reader sees one order. */
function bundleFiles(): BundleFileRowDto[] {
  return [
    bundleFile({ path: 'accepted-reviews.json', role: 'accepted-reviews', byteSize: 512, sha256: '1'.repeat(64) }),
    bundleFile({ path: 'analysis.json', role: 'analysis', byteSize: 8192, sha256: '2'.repeat(64) }),
    bundleFile({ path: 'artifacts/firmware.elf', role: 'artifact', byteSize: 65536, sha256: '3'.repeat(64) }),
    bundleFile({ path: 'artifacts/firmware.map', role: 'artifact', byteSize: 4096, sha256: '4'.repeat(64) }),
    bundleFile({ path: 'diff.json', role: 'comparison', byteSize: 2048, sha256: '5'.repeat(64) }),
    bundleFile({ path: 'gate-results.json', role: 'gate-results', byteSize: 3072, sha256: '6'.repeat(64) }),
    bundleFile({ path: 'release-manifest.json', role: 'manifest', byteSize: 1536, sha256: '7'.repeat(64) }),
    bundleFile({ path: 'release-notes.md', role: 'release-notes', byteSize: 256, sha256: '8'.repeat(64) }),
    bundleFile({ path: 'release-report.html', role: 'report', byteSize: 32768, sha256: '9'.repeat(64) }),
    bundleFile({ path: 'SHA256SUMS', role: 'checksum-index', byteSize: 1024, sha256: '0'.repeat(64) }),
  ];
}

function bundlePreview(overrides: Partial<BundlePreviewDto> = {}): BundlePreviewDto {
  const base: BundlePreviewDto = {
    planId: 'bundle-4021-1',
    releaseId: `release-${'7'.repeat(64)}`,
    releaseVersion: '2.1.0',
    snapshotId: 'snap-target',
    baselineSnapshotId: 'snap-base',
    gateRunId: gateRun().runId,
    disposition: 'PASS',
    acceptedReviewCount: 1,
    files: bundleFiles(),
    warnings: [],
    bundleFolderName: BUNDLE_FOLDER,
  };
  return { ...base, ...overrides };
}

function bundleDestination(overrides: Partial<BundleDestinationDto> = {}): BundleDestinationDto {
  const base: BundleDestinationDto = {
    destinationToken: 'dst-4021-1',
    bundleFolderName: BUNDLE_FOLDER,
    exists: false,
    recognizableBundle: false,
  };
  return { ...base, ...overrides };
}

function bundleExport(overrides: Partial<BundleExportDto> = {}): BundleExportDto {
  const base: BundleExportDto = {
    releaseId: `release-${'7'.repeat(64)}`,
    releaseVersion: '2.1.0',
    folderDisplayName: BUNDLE_FOLDER,
    manifestSha256: 'd'.repeat(64),
    fileCount: 10,
    totalBytes: 117504,
    artifactCount: 2,
    replaced: false,
    recordWritten: true,
  };
  return { ...base, ...overrides };
}

async function openRelease(): Promise<void> {
  render(<App />);
  fireEvent.click(await screen.findByRole('button', { name: 'Release page' }));
  await screen.findByRole('heading', { level: 1, name: 'Release Gate' });
}

/**
 * Press Run Gate once the screen has actually chosen a build.
 *
 * The control is disabled until a build is named, and the name arrives with the stored list, so a test
 * that clicked immediately would be pressing a button the page had not yet enabled.
 */
async function runGate(): Promise<void> {
  const button = (await screen.findByRole('button', { name: 'Run Gate' })) as HTMLButtonElement;
  await waitFor(() => expect(button.disabled).toBe(false));
  fireEvent.click(button);
}

/** The block a heading opens, so an assertion names the part of the page it is about. */
function block(heading: string): HTMLElement {
  const node = screen.getByRole('heading', { level: 3, name: heading });
  const container = node.parentElement;
  if (container === null) {
    throw new Error(`${heading} is not inside a block`);
  }
  return container;
}

function findingRow(rule: string): HTMLElement {
  const node = screen.getByText(rule);
  const row = node.closest('li');
  if (row === null) {
    throw new Error(`${rule} has no finding row`);
  }
  return row;
}

beforeEach(() => {
  vi.clearAllMocks();
  candidatesMock.mockResolvedValue(
    ok(
      candidatePage([
        candidate(),
        candidate({
          buildId: 'build-base',
          snapshotId: 'snap-base',
          fileName: 'app-old.elf',
          byteSize: 7680,
        }),
      ]),
    ),
  );
  vi.mocked(analyzeSelection).mockResolvedValue(fail(ENVELOPE('ERR-INPUT-0001', 'No artifact was found at that path.')));
  vi.mocked(selectArtifact).mockResolvedValue(ok(null));
  vi.mocked(attachMap).mockResolvedValue(ok(null));
  vi.mocked(clearMap).mockResolvedValue(fail(ENVELOPE('ERR-INPUT-0001', 'missing')));
  vi.mocked(querySections).mockResolvedValue(ok(emptyPage()));
  vi.mocked(querySymbols).mockResolvedValue(ok(emptyPage()));
  vi.mocked(queryEvidence).mockResolvedValue(ok(emptyPage()));
  vi.mocked(querySectionChanges).mockResolvedValue(ok(emptyPage()));
  vi.mocked(querySymbolChanges).mockResolvedValue(ok(emptyPage()));
  vi.mocked(compareSnapshots).mockResolvedValue(fail(ENVELOPE('ERR-DIFF-5001', 'same')));
  vi.mocked(exportCompareJson).mockResolvedValue(
    ok({ status: 'cancelled', fileName: null, format: 'json' }),
  );
  vi.mocked(exportCompareHtml).mockResolvedValue(
    ok({ status: 'cancelled', fileName: null, format: 'html' }),
  );
  openConfigMock.mockResolvedValue(ok(null));
  savePolicyMock.mockResolvedValue(ok(null));
  runGateMock.mockResolvedValue(ok(gateRun()));
  acceptMock.mockResolvedValue(fail(NOT_A_REVIEW));
  getRunMock.mockResolvedValue(ok(null));
  prepareBundleMock.mockResolvedValue(ok(bundlePreview()));
  chooseDestinationMock.mockResolvedValue(ok(bundleDestination()));
  exportBundleMock.mockResolvedValue(ok(bundleExport()));
});

describe('Release navigation', () => {
  it('lists the three stages this build has and no others', async () => {
    await openRelease();
    const rail = screen.getByRole('navigation', { name: 'Pages' });
    expect(within(rail).getAllByRole('button').map((item) => item.textContent)).toEqual([
      'Analyze',
      'Compare',
      'Release',
    ]);
    const words = document.body.textContent ?? '';
    // P4 makes `Bundle` a word this build has earned, so it is no longer banned here: §48 attaches the
    // bundle *under* Release, and what §58 forbids is a fifth navigation verb, which the rail assertion
    // above is what pins. The stages still not built get none of their words.
    for (const stage of ['History', 'Settings', 'Pricing', 'Cloud']) {
      expect(words).not.toMatch(new RegExp(`\\b${stage}\\b`));
    }
  });

  it('claims policy readiness and nothing beyond it', async () => {
    await openRelease();
    await runGate();
    await screen.findByText('memory.flash_budget');
    const words = document.body.textContent ?? '';
    for (const claim of ['safe to ship', 'legally compliant', 'certified', 'production ready']) {
      expect(words.toLowerCase()).not.toContain(claim);
    }
    expect(words).toContain('FirmwareSight policy readiness');
  });
});

describe('Project config', () => {
  it('treats a cancelled dialog as no decision rather than a failure', async () => {
    await openRelease();
    fireEvent.click(screen.getByRole('button', { name: 'Open project config' }));
    await waitFor(() => expect(openConfigMock).toHaveBeenCalled());
    expect(screen.queryByRole('alert', { name: 'Config load or save failed' })).toBeNull();
    expect(document.body.textContent).toContain('No project policy is loaded');
  });

  it('shows the project name and the file name, and no path', async () => {
    openConfigMock.mockResolvedValue(
      ok(project({ warnings: ['[gate] unknown_evidence_review_count is recorded and not acted on'] })),
    );
    await openRelease();
    fireEvent.click(screen.getByRole('button', { name: 'Open project config' }));
    await screen.findByText('Controller v2');
    expect(document.body.textContent).toContain('firmwaresight.toml');
    expect(document.body.textContent).toContain('schema_version 1');
    expect(document.body.textContent).toContain('[gate] unknown_evidence_review_count');
  });

  it('never puts a host path on the screen', async () => {
    openConfigMock.mockResolvedValue(
      ok(project({ configFileName: 'firmwaresight.toml', policySha256: 'f'.repeat(64) })),
    );
    await openRelease();
    fireEvent.click(screen.getByRole('button', { name: 'Open project config' }));
    await screen.findByText('Controller v2');
    const words = document.body.textContent ?? '';
    expect(words).not.toMatch(/[A-Za-z]:[\\/]/);
    expect(words).not.toContain('secret-project');
  });

  it('reports a refused config as a typed error and keeps the loaded project', async () => {
    openConfigMock.mockResolvedValue(ok(project()));
    await openRelease();
    fireEvent.click(screen.getByRole('button', { name: 'Open project config' }));
    await screen.findByText('Controller v2');

    openConfigMock.mockResolvedValue(fail(MALFORMED));
    fireEvent.click(screen.getByRole('button', { name: 'Open project config' }));
    const panel = await screen.findByRole('alert', { name: 'Config load or save failed' });
    expect(within(panel).getByText('ERR-CONFIG-7002')).toBeDefined();
    expect(screen.getAllByText('Controller v2').length).toBeGreaterThan(0);
  });

  it('lists unknown keys instead of dropping them quietly', async () => {
    openConfigMock.mockResolvedValue(ok(project({ unknownKeys: ['sbom.extra'] })));
    await openRelease();
    fireEvent.click(screen.getByRole('button', { name: 'Open project config' }));
    const warning = await screen.findByText(/keys this build does not understand/);
    expect(warning.textContent).toContain('sbom.extra');
    expect(warning.textContent).toContain('refused until they are resolved');
  });

  it('sends the edited policy and nothing else', async () => {
    openConfigMock.mockResolvedValue(ok(project()));
    await openRelease();
    fireEvent.click(screen.getByRole('button', { name: 'Open project config' }));
    await screen.findByText('Controller v2');
    fireEvent.click(screen.getByRole('button', { name: 'Edit policy' }));

    fireEvent.change(await screen.findByLabelText('FLASH budget (bytes)'), {
      target: { value: '8192' },
    });
    fireEvent.change(screen.getByLabelText('Require a clean workspace (git.clean)'), {
      target: { value: 'no' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Save policy' }));

    await waitFor(() => expect(savePolicyMock).toHaveBeenCalled());
    const sent = savePolicyMock.mock.calls[0]?.[0];
    expect(sent?.flashBudget).toBe(8192);
    expect(sent?.requireCleanGit).toBe(false);
    const wire = JSON.stringify(sent ?? {});
    expect(wire).not.toMatch(/[A-Za-z]:[\\/]/);
    expect(wire).not.toContain('schema_version');
  });

  it('sends an untouched field as unset, so the shell keeps the default', async () => {
    await openRelease();
    fireEvent.click(screen.getByRole('button', { name: 'Edit policy' }));
    fireEvent.change(await screen.findByLabelText('Project name'), {
      target: { value: 'New project' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Save as firmwaresight.toml' }));
    await waitFor(() => expect(savePolicyMock).toHaveBeenCalled());
    const sent = savePolicyMock.mock.calls[0]?.[0];
    // Nothing in this payload is a default the front end invented: every field is still unset, which is
    // how the shell is told to apply the one it owns (prompt §41).
    expect(sent?.requireCleanGit).toBeNull();
    expect(sent?.releaseNotesPath).toBeNull();
    expect(sent?.requiredArtifactKinds).toBeNull();
    expect(sent?.onUnknown.flashBudget).toBeNull();
  });

  it('reports a refused save as a typed error without losing the loaded policy', async () => {
    openConfigMock.mockResolvedValue(ok(project()));
    savePolicyMock.mockResolvedValue(fail(SAVE_REFUSED));
    await openRelease();
    fireEvent.click(screen.getByRole('button', { name: 'Open project config' }));
    await screen.findByText('Controller v2');
    fireEvent.click(screen.getByRole('button', { name: 'Edit policy' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Save policy' }));

    const panel = await screen.findByRole('alert', { name: 'Config load or save failed' });
    expect(within(panel).getByText('ERR-CONFIG-7007')).toBeDefined();
    expect(document.body.textContent).toContain('Controller v2');
  });
});

describe('Build and baseline', () => {
  it('prefers the build Analyze last proved, else the newest stored one', async () => {
    await openRelease();
    // The name arrives with the stored list, so the page is waited on rather than assumed (prompt §44).
    const current = (await screen.findByLabelText('Current build')) as HTMLSelectElement;
    await waitFor(() => expect(current.value).toBe('snap-target'));
  });

  it('offers no baseline at all until one is asked for', async () => {
    await openRelease();
    const baseline = (await screen.findByLabelText('Baseline (optional)')) as HTMLSelectElement;
    expect(baseline.value).toBe('');
    expect(document.body.textContent).toContain('the growth rule answers Unknown');
    await runGate();
    expect(runGateMock.mock.calls[0]?.[0]).toEqual({
      snapshotId: 'snap-target',
      baselineSnapshotId: null,
    });
  });

  it('will not offer the current build as its own baseline', async () => {
    await openRelease();
    const current = (await screen.findByLabelText('Current build')) as HTMLSelectElement;
    await waitFor(() => expect(current.value).toBe('snap-target'));
    const baseline = screen.getByLabelText('Baseline (optional)');
    expect(within(baseline).queryByRole('option', { name: /app\.elf/ })).toBeNull();
    expect(within(baseline).getByRole('option', { name: /app-old\.elf/ })).toBeDefined();
  });

  it('sends the pair the reader chose', async () => {
    await openRelease();
    fireEvent.change(await screen.findByLabelText('Baseline (optional)'), {
      target: { value: 'snap-base' },
    });
    await runGate();
    expect(runGateMock.mock.calls[0]?.[0]).toEqual({
      snapshotId: 'snap-target',
      baselineSnapshotId: 'snap-base',
    });
  });
});

describe('Findings', () => {
  it('groups BLOCK before REVIEW before UNKNOWN before PASS before N/A', async () => {
    await openRelease();
    await runGate();
    await screen.findByText('memory.flash_budget');
    // The group headings are the order the reader is asked to read in, and they come before the number
    // blocks, which are detail rather than verdicts (prompt §47, §43).
    const headings = screen
      .getAllByRole('heading', { level: 3 })
      .map((node) => (node.textContent ?? '').replace(/\d+$/, ''));
    expect(headings.slice(0, 5)).toEqual(['Block', 'Review', 'Unknown', 'Pass', 'Not applicable']);
    expect(headings.slice(5)).toEqual([
      'Required artifacts',
      'Memory budgets',
      'Growth against the baseline',
      'Release notes',
    ]);
  });

  it('shows rule, state, effective severity, summary, evidence and next step', async () => {
    await openRelease();
    await runGate();
    await screen.findByText('memory.flash_budget');
    const row = findingRow('memory.flash_budget');
    expect(within(row).getByText('BLOCK')).toBeDefined();
    expect(within(row).getByText('effective severity: BLOCK')).toBeDefined();
    expect(within(row).getByText('FLASH footprint 4608 B exceeds the 4096 B budget.')).toBeDefined();
    expect(within(row).getByText('policy:memory.flash_budget=4096')).toBeDefined();
    expect(within(row).getByText(/Reduce the footprint/)).toBeDefined();
  });

  it('states an UNKNOWN as a gap with a next step, and never as a pass', async () => {
    await openRelease();
    await runGate();
    await screen.findByText('release.notes');
    const row = findingRow('release.notes');
    expect(within(row).getByText('UNKNOWN')).toBeDefined();
    expect(within(row).getByText('effective severity: BLOCK')).toBeDefined();
    expect(within(row).queryByText('PASS')).toBeNull();
    expect(within(row).getByText(/Fix the file permissions/)).toBeDefined();
  });

  it('shows the aggregate both as computed and as dispositioned', async () => {
    await openRelease();
    await runGate();
    expect(await screen.findByText('Disposition: BLOCK')).toBeDefined();
    expect(screen.getByText('As computed: BLOCK')).toBeDefined();
  });

  it('keeps the last good run on screen when a new attempt fails', async () => {
    await openRelease();
    await runGate();
    await screen.findByText('memory.flash_budget');

    runGateMock.mockResolvedValueOnce(fail(STORAGE_MISS));
    await runGate();
    const panel = await screen.findByRole('alert', { name: 'Gate run failed' });
    expect(within(panel).getByText('ERR-STORAGE-4005')).toBeDefined();
    expect(screen.getByText(/last run that succeeded/)).toBeDefined();
    expect(screen.getByText('memory.flash_budget')).toBeDefined();
  });

  it('uses the workspace wording Rust sent and never over-claims it', async () => {
    await openRelease();
    await runGate();
    expect(await screen.findByText('Workspace dirty')).toBeDefined();
    expect(screen.getByText('exact tag v2.1.0 on workspace HEAD')).toBeDefined();
    expect(screen.getByText('dirty (workspace, not artifact)')).toBeDefined();
    const words = document.body.textContent ?? '';
    expect(words).not.toMatch(/Artifact commit/i);
    expect(words).not.toMatch(/built from (the )?HEAD/i);
  });

  it('says plainly when no workspace was observed', async () => {
    runGateMock.mockResolvedValue(
      ok(
        gateRun({
          git: {
            available: false,
            headCommit: null,
            exactTag: null,
            dirty: null,
            summary: 'No workspace facts were observed',
            reason: 'git is not installed',
          },
        }),
      ),
    );
    await openRelease();
    await runGate();
    expect(await screen.findByText('No workspace facts were observed')).toBeDefined();
    expect(screen.getByText('git is not installed')).toBeDefined();
    expect(document.body.textContent).not.toContain('Workspace HEAD');
  });
});

describe('The numbers behind the findings', () => {
  it('shows required artifacts as required, present or missing', async () => {
    await openRelease();
    await runGate();
    await screen.findByText('memory.flash_budget');
    const table = block('Required artifacts').querySelector('table');
    if (table === null) {
      throw new Error('the artifact table is missing');
    }
    expect(within(table).getByText('map')).toBeDefined();
    expect(within(table).getAllByText('Required').length).toBe(2);
    expect(within(table).getByText('Missing')).toBeDefined();
  });

  it('does not advertise analysis it has not done', async () => {
    await openRelease();
    await runGate();
    await screen.findByText('memory.flash_budget');
    expect(block('Required artifacts').textContent).toContain('checked for presence only');
    const words = document.body.textContent ?? '';
    expect(words).not.toMatch(/BIN files are analyzed/i);
  });

  it('shows a budget with its excess, and no invented number for an unknown one', async () => {
    await openRelease();
    await runGate();
    await screen.findByText('memory.flash_budget');
    const table = block('Memory budgets').querySelector('table');
    if (table === null) {
      throw new Error('the budget table is missing');
    }
    expect(within(table).getByText('512 bytes over')).toBeDefined();
    const flash = within(table).getByText('FLASH').closest('tr');
    const ram = within(table).getByText('RAM').closest('tr');
    if (flash === null || ram === null) {
      throw new Error('a budget row is missing');
    }
    expect(within(flash).getByText('4,096 bytes')).toBeDefined();
    const cells = within(ram)
      .getAllByRole('cell')
      .map((cell) => (cell.textContent ?? '').trim());
    // Actual, budget, headroom-or-over, state, basis. The two number cells the reader would take as
    // measurements both say `Unknown`, and no cell holds an invented zero (prompt §52).
    expect(cells[0]).toBe('Unknown');
    expect(cells[1]).toBe('2,048 bytes');
    expect(cells[2]).toBe('Unknown');
    expect(cells).not.toContain('0 bytes');
    expect(cells[3]).toContain('UNKNOWN');
    expect(cells[4]).toContain('the snapshot carries no complete RAM attribution');
  });

  it('moves the growth numbers P2 computed instead of recomputing them', async () => {
    await openRelease();
    await runGate();
    await screen.findByText('memory.flash_budget');
    const table = block('Growth against the baseline').querySelector('table');
    if (table === null) {
      throw new Error('the growth table is missing');
    }
    expect(within(table).getByText('+512 bytes')).toBeDefined();
    expect(within(table).getByText('4,608 bytes')).toBeDefined();
    expect(within(table).getByText('256 bytes')).toBeDefined();
    expect(within(table).getByText('REVIEW')).toBeDefined();
  });

  it('relabels the same figures in KiB and asks the shell for nothing', async () => {
    await openRelease();
    await runGate();
    await screen.findByText('memory.flash_budget');
    const table = block('Memory budgets').querySelector('table');
    if (table === null) {
      throw new Error('the budget table is missing');
    }
    expect(within(table).getByText('4,096 bytes')).toBeDefined();
    fireEvent.click(screen.getByRole('radio', { name: 'KiB' }));
    expect(within(table).getByText('4.00 KiB')).toBeDefined();
    expect(runGateMock).toHaveBeenCalledTimes(1);
    expect(candidatesMock).toHaveBeenCalledTimes(1);
  });

  it('shows the notes file by relative path only', async () => {
    await openRelease();
    await runGate();
    await screen.findByText('memory.flash_budget');
    const container = block('Release notes');
    expect(within(container).getByText('docs/RELEASE_NOTES.md')).toBeDefined();
    expect(within(container).getByText('Required · Present')).toBeDefined();
    expect(within(container).getByText('c'.repeat(64))).toBeDefined();
    expect(container.textContent).toContain('the folder itself is never shown');
  });
});

describe('Review acceptance', () => {
  it('offers Accept on a REVIEW row and nowhere else', async () => {
    await openRelease();
    await runGate();
    await screen.findByText('diff.growth');
    expect(screen.getAllByRole('button', { name: 'Accept review' })).toHaveLength(1);
    for (const rule of [
      'memory.flash_budget',
      'release.notes',
      'git.clean',
      'release.commit_matches_expected',
    ]) {
      expect(within(findingRow(rule)).queryByRole('button', { name: 'Accept review' })).toBeNull();
    }
  });

  it('refuses to submit without a name and a reason', async () => {
    await openRelease();
    await runGate();
    fireEvent.click(await screen.findByRole('button', { name: 'Accept review' }));
    const submit = (await screen.findByRole('button', {
      name: 'Record acceptance',
    })) as HTMLButtonElement;
    expect(submit.disabled).toBe(true);
    expect(acceptMock).not.toHaveBeenCalled();

    fireEvent.change(screen.getByLabelText('Who accepts this review'), {
      target: { value: 'rosa' },
    });
    expect(submit.disabled).toBe(true);

    fireEvent.change(screen.getByLabelText('Why it is acceptable'), {
      target: { value: 'the growth is the new bootloader' },
    });
    expect(submit.disabled).toBe(false);
  });

  it('keeps the row at REVIEW, shows the record beside it, and moves only the aggregate', async () => {
    runGateMock.mockResolvedValue(ok(reviewOnlyRun()));
    acceptMock.mockResolvedValue(
      ok({
        runId: reviewOnlyRun().runId,
        findingId: 'f-growth',
        state: 'REVIEW',
        dispositionEffectiveSeverity: 'PASS',
        acceptance: {
          findingId: 'f-growth',
          actor: 'rosa',
          acceptedAt: '2026-09-30T08:02:00Z',
          reason: 'the growth is the new bootloader',
          originalState: 'REVIEW',
        },
      } satisfies AcceptReviewOutcomeDto),
    );
    await openRelease();
    await runGate();
    expect(await screen.findByText('Disposition: REVIEW')).toBeDefined();

    fireEvent.click(screen.getByRole('button', { name: 'Accept review' }));
    fireEvent.change(screen.getByLabelText('Who accepts this review'), { target: { value: 'rosa' } });
    fireEvent.change(screen.getByLabelText('Why it is acceptable'), {
      target: { value: 'the growth is the new bootloader' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Record acceptance' }));

    // The form closes when the shell accepts the record, so waiting for it to go is waiting for the
    // write to land rather than for a render tick.
    await waitFor(() => expect(screen.queryByLabelText('Who accepts this review')).toBeNull());
    const row = findingRow('diff.growth');
    // The row's own head still carries REVIEW, and the record beside it says which state was accepted:
    // an accepted review is never rendered as a pass (prompt §48).
    const head = row.querySelector('div');
    expect(head?.textContent).toContain('REVIEW');
    expect(head?.textContent).toContain('effective severity: REVIEW');
    const text = row.textContent ?? '';
    expect(text).toContain('rosa');
    expect(text).toContain('2026-09-30T08:02:00Z');
    expect(text).toContain('the growth is the new bootloader');
    expect(text).toContain('cannot be edited or deleted');
    expect(within(row).queryByRole('button', { name: /edit|delete|remove/i })).toBeNull();
    expect(within(row).queryByRole('button', { name: 'Accept review' })).toBeNull();
    await waitFor(() => expect(screen.getByText('Disposition: PASS')).toBeDefined());
    expect(screen.getByText('As computed: REVIEW')).toBeDefined();
  });

  it('shows a refused acceptance as a typed error and keeps the run', async () => {
    await openRelease();
    await runGate();
    fireEvent.click(await screen.findByRole('button', { name: 'Accept review' }));
    fireEvent.change(screen.getByLabelText('Who accepts this review'), { target: { value: 'rosa' } });
    fireEvent.change(screen.getByLabelText('Why it is acceptable'), { target: { value: 'because' } });
    fireEvent.click(screen.getByRole('button', { name: 'Record acceptance' }));

    const panel = await screen.findByRole('alert', { name: 'Acceptance of diff.growth failed' });
    expect(within(panel).getByText('ERR-STORAGE-4008')).toBeDefined();
    expect(screen.getByText('diff.growth')).toBeDefined();
    expect(screen.getByText('memory.flash_budget')).toBeDefined();
  });

  it('reads the stored record and says what a record does not carry', async () => {
    getRunMock.mockResolvedValue(
      ok(
        gateRun({
          tables: null,
          policy: null,
          projectName: null,
          policySource: 'policy eeeeeeee (read back from history)',
          recordNote:
            'This run was read back from history. Its findings, evidence and accepted reviews are the stored record.',
        }),
      ),
    );
    await openRelease();
    await runGate();
    await screen.findByText('memory.flash_budget');
    fireEvent.click(screen.getByRole('button', { name: 'Read the stored record' }));
    expect(await screen.findByText(/^This run was read back from history/)).toBeDefined();
    expect(screen.queryByRole('heading', { level: 2, name: /What the findings were judged on/ })).toBeNull();
    expect(screen.getByText('memory.flash_budget')).toBeDefined();
    expect(getRunMock.mock.calls[0]?.[0]).toBe(gateRun().runId);
  });
});

describe('Release bundle', () => {
  /** A button as the element it is, because this suite has no jest-dom matchers. */
  function namedButton(name: string | RegExp): HTMLButtonElement {
    return screen.getByRole('button', { name }) as HTMLButtonElement;
  }

  /** The value beside a term in one of the preview blocks. */
  function fact(container: HTMLElement, term: string): string {
    const label = within(container).getByText(term);
    const value = label.parentElement?.querySelector('dd');
    if (value === null || value === undefined) {
      throw new Error(`${term} has no value`);
    }
    return value.textContent ?? '';
  }

  /** Run the Gate to a PASS disposition and prepare the plan, so a test starts at the preview. */
  async function prepared(): Promise<void> {
    runGateMock.mockResolvedValue(ok(passingRun()));
    await openRelease();
    await runGate();
    fireEvent.click(await screen.findByRole('button', { name: 'Prepare bundle' }));
    await screen.findByRole('heading', { level: 3, name: 'Bundle preview' });
  }

  /** Choose a destination in the dialog this page cannot see, and wait for the screen to acknowledge it. */
  async function chosen(): Promise<void> {
    fireEvent.click(namedButton('Choose destination folder'));
    await waitFor(() => expect(document.body.textContent).toContain('Destination chosen for'));
  }

  it('attaches the bundle under Release and adds no navigation verb', async () => {
    await prepared();
    const heading = screen.getByRole('heading', { level: 2, name: '6 · Release Bundle' });
    // §58: the rail is still the three stages. A bundle is a section of Release, not a fourth doorway,
    // so the only new verb this build has is the one inside this page.
    const rail = screen.getByRole('navigation', { name: 'Pages' });
    expect(within(rail).getAllByRole('button').map((item) => item.textContent)).toEqual([
      'Analyze',
      'Compare',
      'Release',
    ]);
    expect(within(rail).queryByRole('button', { name: /bundle/i })).toBeNull();
    expect(heading.closest('nav')).toBeNull();
  });

  it('keeps the bundle closed until the disposition says PASS, and says why', async () => {
    await openRelease();
    await runGate();
    // The default run's disposition is BLOCK, so nothing here is offerable — and the reason is on the
    // screen rather than inferred from a grey button (prompt §58).
    await screen.findByRole('button', { name: 'Prepare bundle' });
    expect(namedButton('Prepare bundle').disabled).toBe(true);
    expect(
      screen.getByText(
        /^No bundle is prepared: the disposition is BLOCK\. A release bundle is written only for a PASS disposition/,
      ),
    ).toBeDefined();
    expect(
      screen.getByText('Prepare is disabled because of the disposition above, not because of anything this page decided.'),
    ).toBeDefined();
    expect(namedButton('Choose destination folder').disabled).toBe(true);
    expect(namedButton('Export bundle').disabled).toBe(true);
    expect(screen.queryByRole('heading', { level: 3, name: 'Bundle preview' })).toBeNull();
    expect(prepareBundleMock).not.toHaveBeenCalled();
  });

  it('opens once the aggregate reaches PASS on the strength of an accepted review', async () => {
    runGateMock.mockResolvedValue(ok(passingRun()));
    await openRelease();
    await runGate();
    // The row still reads REVIEW with its acceptance beside it while the disposition reads PASS: the
    // section opens on Core's aggregate, not on a rewritten finding (prompt §48).
    expect(await screen.findByText('Disposition: PASS')).toBeDefined();
    expect(findingRow('diff.growth').querySelector('div')?.textContent).toContain('REVIEW');
    await waitFor(() => expect(namedButton('Prepare bundle').disabled).toBe(false));
  });

  it('shows the plan a release owner authorizes, and no path with it', async () => {
    await prepared();
    const preview = block('Bundle preview');
    expect(fact(preview, 'Release')).toBe(`release-${'7'.repeat(64)}`);
    expect(fact(preview, 'Version')).toBe('2.1.0');
    expect(fact(preview, 'Current build')).toBe('snap-target');
    expect(fact(preview, 'Baseline')).toBe('snap-base');
    expect(fact(preview, 'Gate run')).toBe(gateRun().runId);
    expect(fact(preview, 'Accepted reviews')).toBe('1');
    expect(fact(preview, 'Proposed folder')).toBe(BUNDLE_FOLDER);
    expect(preview.textContent).toContain('8,192 bytes');
    const words = document.body.textContent ?? '';
    expect(words).not.toMatch(/[A-Za-z]:[\\/]/);
    expect(words).not.toContain('dst-');
  });

  it('sends Prepare three ids and nothing else', async () => {
    await prepared();
    expect(prepareBundleMock).toHaveBeenCalledWith({
      snapshotId: 'snap-target',
      baselineSnapshotId: 'snap-base',
      gateRunId: gateRun().runId,
    });
  });

  it('says which build the plan is standing on when there is no baseline', async () => {
    prepareBundleMock.mockResolvedValue(ok(bundlePreview({ baselineSnapshotId: null })));
    await prepared();
    expect(fact(block('Bundle preview'), 'Baseline')).toBe('None');
  });

  it('reports a refused Prepare as the typed error the shell wrote', async () => {
    runGateMock.mockResolvedValue(ok(passingRun()));
    prepareBundleMock.mockResolvedValue(
      fail(ENVELOPE('ERR-BUNDLE-6114', 'This session has not analyzed that build, so its bytes are not held.')),
    );
    await openRelease();
    await runGate();
    fireEvent.click(await screen.findByRole('button', { name: 'Prepare bundle' }));
    const panel = await screen.findByRole('alert', { name: 'Bundle step failed' });
    expect(within(panel).getByText('ERR-BUNDLE-6114')).toBeDefined();
    expect(within(panel).getByText('This session has not analyzed that build, so its bytes are not held.'))
      .toBeDefined();
    expect(screen.getByRole('heading', { level: 2, name: 'Release bundle error' })).toBeDefined();
    expect(screen.queryByRole('heading', { level: 3, name: 'Bundle preview' })).toBeNull();
  });

  it('lists the plan warnings instead of dropping them quietly', async () => {
    prepareBundleMock.mockResolvedValue(
      ok(bundlePreview({ warnings: ['the MAP leaf name was disambiguated for the bundle folder'] })),
    );
    await prepared();
    const list = await screen.findByRole('list', { name: 'Plan warnings' });
    expect(within(list).getByText(/disambiguated/)).toBeDefined();
  });

  it('treats a cancelled destination as a decision, not a failure', async () => {
    chooseDestinationMock.mockResolvedValue(ok(null));
    await prepared();
    fireEvent.click(namedButton('Choose destination folder'));
    await waitFor(() =>
      expect(document.body.textContent).toContain('No folder was chosen. The bundle has not been written.'),
    );
    expect(screen.queryByRole('alert', { name: 'Bundle step failed' })).toBeNull();
    expect(screen.getByRole('heading', { level: 3, name: 'Bundle preview' })).toBeDefined();
    expect(namedButton('Export bundle').disabled).toBe(true);
    expect(exportBundleMock).not.toHaveBeenCalled();
  });

  it('labels the chosen destination by name and says what is already there', async () => {
    await prepared();
    await chosen();
    expect(document.body.textContent).toContain('Nothing of that name is there yet.');
    expect(document.body.textContent).not.toMatch(/[A-Za-z]:[\\/]/);

    chooseDestinationMock.mockResolvedValue(ok(bundleDestination({ exists: true, recognizableBundle: true })));
    fireEvent.click(namedButton('Choose destination folder'));
    await waitFor(() =>
      expect(document.body.textContent).toContain('it reads as a FirmwareSight bundle.'),
    );

    chooseDestinationMock.mockResolvedValue(ok(bundleDestination({ exists: true, recognizableBundle: false })));
    fireEvent.click(namedButton('Choose destination folder'));
    await waitFor(() =>
      expect(document.body.textContent).toContain('it will not be replaced.'),
    );
  });

  it('asks before it replaces a bundle, and writes nothing until the answer is to replace', async () => {
    chooseDestinationMock.mockResolvedValue(ok(bundleDestination({ exists: true, recognizableBundle: true })));
    exportBundleMock.mockResolvedValue(fail(DESTINATION_TAKEN));
    await prepared();
    await chosen();
    fireEvent.click(namedButton('Export bundle'));

    const group = await screen.findByRole('group', { name: 'Replace the existing bundle?' });
    expect(within(group).getByRole('button', { name: `Replace the existing bundle named ${BUNDLE_FOLDER}` }))
      .toBeDefined();
    // The first press is unambiguous: overwrite false, and nothing on disk has changed.
    expect(exportBundleMock.mock.calls).toEqual([['bundle-4021-1', 'dst-4021-1', false]]);

    fireEvent.click(within(group).getByRole('button', { name: 'Keep it' }));
    await waitFor(() =>
      expect(document.body.textContent).toContain('Kept the bundle that was there. Nothing was written.'),
    );
    expect(screen.queryByRole('group', { name: 'Replace the existing bundle?' })).toBeNull();
    expect(exportBundleMock).toHaveBeenCalledTimes(1);
    expect(screen.queryByRole('heading', { level: 3, name: 'Bundle created' })).toBeNull();
  });

  it('replaces only after the second, explicit press, and says that it did', async () => {
    chooseDestinationMock.mockResolvedValue(ok(bundleDestination({ exists: true, recognizableBundle: true })));
    exportBundleMock
      .mockResolvedValueOnce(fail(DESTINATION_TAKEN))
      .mockResolvedValueOnce(ok(bundleExport({ replaced: true })));
    await prepared();
    await chosen();
    fireEvent.click(namedButton('Export bundle'));
    const group = await screen.findByRole('group', { name: 'Replace the existing bundle?' });
    fireEvent.click(
      within(group).getByRole('button', { name: `Replace the existing bundle named ${BUNDLE_FOLDER}` }),
    );

    const result = await screen.findByRole('heading', { level: 3, name: 'Bundle created' });
    expect(exportBundleMock.mock.calls[1]).toEqual(['bundle-4021-1', 'dst-4021-1', true]);
    expect(
      (result.parentElement as HTMLElement).textContent,
    ).toContain('This export replaced the bundle that was there, under your confirmation.');
  });

  it('refuses a folder that is not a bundle and offers no replace button for it', async () => {
    chooseDestinationMock.mockResolvedValue(ok(bundleDestination({ exists: true, recognizableBundle: false })));
    exportBundleMock.mockResolvedValue(fail(NOT_A_BUNDLE));
    await prepared();
    await chosen();
    fireEvent.click(namedButton('Export bundle'));

    const panel = await screen.findByRole('alert', { name: 'Bundle step failed' });
    expect(within(panel).getByText('ERR-BUNDLE-6107')).toBeDefined();
    // §32: an arbitrary directory is never a replacement target, so there is nothing to confirm.
    expect(screen.queryByRole('group', { name: 'Replace the existing bundle?' })).toBeNull();
    expect(exportBundleMock).toHaveBeenCalledTimes(1);
  });

  it('drops a stale plan instead of letting it authorize a write', async () => {
    exportBundleMock.mockResolvedValue(fail(STALE_CONTEXT));
    await prepared();
    await chosen();
    fireEvent.click(namedButton('Export bundle'));

    const panel = await screen.findByRole('alert', { name: 'Bundle step failed' });
    expect(within(panel).getByText('ERR-BUNDLE-6102')).toBeDefined();
    await waitFor(() =>
      expect(screen.queryByRole('heading', { level: 3, name: 'Bundle preview' })).toBeNull(),
    );
    expect(document.body.textContent).not.toContain('Destination chosen for');
    expect(namedButton('Choose destination folder').disabled).toBe(true);
    expect(exportBundleMock).toHaveBeenCalledTimes(1);
    // The screen says what to do next, and it is to prepare again rather than to press the same button.
    expect(namedButton('Prepare bundle').disabled).toBe(false);
  });

  it('clears the authorization when the plan itself has expired', async () => {
    exportBundleMock.mockResolvedValue(
      fail(ENVELOPE('ERR-BUNDLE-6105', 'No plan with that id is held by this session.')),
    );
    await prepared();
    await chosen();
    fireEvent.click(namedButton('Export bundle'));
    const panel = await screen.findByRole('alert', { name: 'Bundle step failed' });
    expect(within(panel).getByText('ERR-BUNDLE-6105')).toBeDefined();
    await waitFor(() =>
      expect(screen.queryByRole('heading', { level: 3, name: 'Bundle preview' })).toBeNull(),
    );
  });

  it('summarizes what the export wrote and what it does not claim', async () => {
    await prepared();
    await chosen();
    fireEvent.click(namedButton('Export bundle'));

    const result = await screen.findByRole('heading', { level: 3, name: 'Bundle created' });
    const summary = result.parentElement as HTMLElement;
    expect(fact(summary, 'Folder')).toBe(BUNDLE_FOLDER);
    expect(fact(summary, 'Manifest SHA-256')).toBe('d'.repeat(64));
    expect(fact(summary, 'Files')).toBe('10 (2 artifacts)');
    expect(summary.textContent).toContain('verified against its own SHA256SUMS and manifest');
    expect(summary.textContent).toContain('The release is recorded in this database.');
    expect(summary.textContent).not.toMatch(/[A-Za-z]:[\\/]/);
    // §62: the bundle is integrity-checkable, and the screen never reaches past that.
    const words = (document.body.textContent ?? '').toLowerCase();
    for (const claim of ['trusted', 'authentic', 'signed', 'tamper-proof']) {
      expect(words).not.toMatch(new RegExp(`\\b${claim}\\b`));
    }
  });

  it('says plainly when the bytes landed but the record did not', async () => {
    exportBundleMock.mockResolvedValue(ok(bundleExport({ recordWritten: false })));
    await prepared();
    await chosen();
    fireEvent.click(namedButton('Export bundle'));
    const result = await screen.findByRole('heading', { level: 3, name: 'Bundle created' });
    const text = (result.parentElement as HTMLElement).textContent ?? '';
    expect(text).toContain('The release record was not written; the bundle itself is complete.');
    expect(text).toContain('verified against its own SHA256SUMS and manifest');
  });

  it('keeps the file list a table a reader can navigate', async () => {
    await prepared();
    const table = within(block('Bundle preview')).getByRole('table');
    expect(within(table).getByRole('caption')?.textContent).toContain('Every file the bundle will hold');
    expect(within(table).getAllByRole('columnheader').map((item) => item.textContent)).toEqual([
      'File',
      'Role',
      'Size',
      'SHA-256',
    ]);
    expect(within(table).getAllByRole('row')).toHaveLength(11);
    expect(within(table).getAllByRole('rowheader').map((item) => item.textContent)).toEqual(
      bundleFiles().map((file) => file.path),
    );
  });

  it('puts focus on the decision it just asked for', async () => {
    chooseDestinationMock.mockResolvedValue(ok(bundleDestination({ exists: true, recognizableBundle: true })));
    exportBundleMock.mockResolvedValue(fail(DESTINATION_TAKEN));
    await prepared();
    await chosen();
    const exportButton = namedButton('Export bundle');
    exportButton.focus();
    fireEvent.click(exportButton);

    const replace = await screen.findByRole('button', {
      name: `Replace the existing bundle named ${BUNDLE_FOLDER}`,
    });
    expect(document.activeElement).toBe(replace);
    fireEvent.click(screen.getByRole('button', { name: 'Keep it' }));
    await waitFor(() => expect(screen.queryByRole('group', { name: 'Replace the existing bundle?' })).toBeNull());
    expect(namedButton('Export bundle')).toBeDefined();
  });

  it('drops the plan when the release owner moves to another run', async () => {
    await prepared();
    await chosen();
    // A new run is a different release, so the preview that was authorized against the old one goes with it
    // (§28). Changing the build selection re-runs the Gate and the shell hands back a different run id.
    runGateMock.mockResolvedValue(
      ok(passingRun({ runId: `gate-${'b'.repeat(64)}`, snapshotId: 'snap-other' })),
    );
    fireEvent.click(namedButton('Run Gate'));
    await waitFor(() => expect(screen.queryByRole('heading', { level: 3, name: 'Bundle preview' })).toBeNull());
    expect(document.body.textContent).not.toContain('Destination chosen for');
  });
});

describe('Accessibility', () => {
  it('names every field the page offers', async () => {
    openConfigMock.mockResolvedValue(ok(project()));
    await openRelease();
    fireEvent.click(screen.getByRole('button', { name: 'Open project config' }));
    await screen.findByText('Controller v2');
    fireEvent.click(screen.getByRole('button', { name: 'Edit policy' }));

    const controls = Array.from(
      document.querySelectorAll<HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement>(
        'input, select, textarea',
      ),
    );
    expect(controls.length).toBeGreaterThan(10);
    for (const control of controls) {
      const labelFor =
        control.id === '' ? null : document.querySelector(`label[for="${control.id}"]`);
      const named =
        control.getAttribute('aria-label') !== null ||
        control.closest('label') !== null ||
        labelFor !== null;
      expect({ id: control.id, named }).toEqual({ id: control.id, named: true });
    }
  });

  it('pairs each state with its own word, not a colour alone', async () => {
    await openRelease();
    await runGate();
    await screen.findByText('memory.flash_budget');
    const label = within(findingRow('memory.flash_budget')).getByText('BLOCK');
    // The badge that carries the word also carries the icon, so the state is legible without colour.
    const badge = label.parentElement;
    expect(badge?.querySelector('svg')).not.toBeNull();
    expect(badge?.getAttribute('aria-hidden')).toBeNull();
  });

  it('opens and closes the acceptance form from the keyboard', async () => {
    await openRelease();
    await runGate();
    const accept = await screen.findByRole('button', { name: 'Accept review' });
    accept.focus();
    fireEvent.click(accept);
    expect(await screen.findByLabelText('Who accepts this review')).toBeDefined();
    fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
    await waitFor(() => expect(screen.queryByLabelText('Who accepts this review')).toBeNull());
  });

  it('keeps the size unit one answer across the page', async () => {
    await openRelease();
    await runGate();
    const switchGroup = screen.getByRole('group', { name: 'Size units' });
    expect(within(switchGroup).getByRole('radio', { name: 'Bytes' })).toBeDefined();
    expect(within(switchGroup).getByRole('radio', { name: 'KiB' })).toBeDefined();
  });
});
