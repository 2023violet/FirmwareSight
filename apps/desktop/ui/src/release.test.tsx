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
 *   it replaces anything, and shows the plan the engine rather than the page composed (prompt §48, §58);
 * - and a release attachment is shown as measured bytes with a declared kind and an unestablished
 *   provenance, never as an analyzed file, and changing the set takes the verdict with it
 *   (`04_TECH/28` §9, §4.2).
 *
 * The bridge is mocked, the way the P1 and P2 files do it: the plumbing has its own test. No assertion
 * here checks a number the shell did not send, because the page computes none of them.
 */

import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { App } from './App';
import {
  acceptReview,
  analyzeSelection,
  attachMap,
  attachReleaseFile,
  chooseBundleDestination,
  clearMap,
  compareSnapshots,
  exportCompareHtml,
  exportCompareJson,
  exportReleaseBundle,
  exportGateMarkdown,
  exportGateVisual,
  getGateRun,
  getReleaseAttachments,
  inspectProjectReadiness,
  listCompareCandidates,
  openProjectConfig,
  prepareReleaseBundle,
  queryEvidence,
  querySectionChanges,
  querySections,
  querySymbolChanges,
  querySymbols,
  removeReleaseAttachment,
  runReleaseGate,
  saveProjectPolicy,
  selectArtifact,
  setReleaseAttachmentKind,
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
  ReleaseAttachmentDto,
  ReleaseAttachmentsDto,
  ProjectReadinessDto,
} from './ipc/types';
import { precedes } from './test/order';

vi.mock('./ipc/bridge', () => ({
  // R5: the pinned baseline is read once at start; no pin unless a test says otherwise.
  getBaselinePin: vi.fn(async () => await Promise.resolve({ ok: true, value: null })),
  pinBaseline: vi.fn(),
  unpinBaseline: vi.fn(),
  exportAnalysisMarkdown: vi.fn(),
  exportCompareMarkdown: vi.fn(),
  exportGateMarkdown: vi.fn(),
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
  attachReleaseFile: vi.fn(),
  getReleaseAttachments: vi.fn(),
  inspectProjectReadiness: vi.fn(),
  removeReleaseAttachment: vi.fn(),
  setReleaseAttachmentKind: vi.fn(),
  prepareReleaseBundle: vi.fn(),
  chooseBundleDestination: vi.fn(),
  exportReleaseBundle: vi.fn(),
  exportGateVisual: vi.fn(),
  setWindowTitle: vi.fn(() => Promise.resolve({ ok: true, value: null })),
  getApplicationPreferences: vi.fn(() =>
    Promise.resolve({
      ok: true,
      value: {
        sizeUnit: 'bytes',
        startupPage: 'analyze',
        recovery: 'missing',
        keptUnreadableCopy: false,
        fileName: 'settings.json',
      },
    }),
  ),
  updateApplicationPreferences: vi.fn(() =>
    Promise.resolve({
      ok: true,
      value: {
        sizeUnit: 'bytes',
        startupPage: 'analyze',
        recovery: 'read',
        keptUnreadableCopy: false,
        fileName: 'settings.json',
      },
    }),
  ),
  resetApplicationPreferences: vi.fn(() =>
    Promise.resolve({
      ok: true,
      value: {
        sizeUnit: 'bytes',
        startupPage: 'analyze',
        recovery: 'read',
        keptUnreadableCopy: false,
        fileName: 'settings.json',
      },
    }),
  ),
  getAppIdentity: vi.fn(() => Promise.resolve({ ok: true, value: {} })),
  listHistoryBuilds: vi.fn(),
  listHistoryGateRuns: vi.fn(),
  listHistoryReleases: vi.fn(),
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
const exportGateVisualMock = vi.mocked(exportGateVisual);
const attachFileMock = vi.mocked(attachReleaseFile);
const readAttachmentsMock = vi.mocked(getReleaseAttachments);
const inspectReadinessMock = vi.mocked(inspectProjectReadiness);
const removeAttachmentMock = vi.mocked(removeReleaseAttachment);
const declareKindMock = vi.mocked(setReleaseAttachmentKind);

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
        {
          kind: 'elf',
          required: true,
          present: true,
          satisfiedBy: 'analyzed',
          sha256: 'b'.repeat(64),
          byteSize: 8192,
        },
        {
          kind: 'map',
          required: true,
          present: false,
          satisfiedBy: 'none',
          sha256: null,
          byteSize: null,
        },
        {
          kind: 'bin',
          required: true,
          present: true,
          satisfiedBy: 'attachment',
          sha256: 'c'.repeat(64),
          byteSize: 4096,
        },
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
  await screen.findByRole('heading', { level: 1, name: 'Release' });
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

/** R4: a run's findings, evidence and bundle are three views of the page. */
function openView(name: 'Findings' | 'Evidence' | 'Bundle'): void {
  fireEvent.click(screen.getByRole('tab', { name }));
}

/** R4: the Gate visual exports are one menu in the page header; open it and return one format. */
async function exportItem(name: string): Promise<HTMLButtonElement> {
  const menu = await screen.findByRole('group', { name: 'Standalone visual report' });
  fireEvent.click(within(menu).getByRole('button', { name: 'Export' }));
  return within(menu).getByRole('button', { name }) as HTMLButtonElement;
}

/** R4: PASS and N/A start collapsed; open every collapsed group so each row can be read. */
function expandAll(): void {
  for (const button of screen.queryAllByRole('button', { name: /^Show \d+ .* finding\(s\)$/ })) {
    fireEvent.click(button);
  }
}

/** The build a BuildPicker names, by snapshot id (`data-value`; empty for none). */
function pickerValue(label: string): string | null {
  return screen.getByRole('combobox', { name: label }).getAttribute('data-value');
}

/** Open a BuildPicker and choose the option whose text matches. */
function choose(label: string, option: RegExp): void {
  fireEvent.click(screen.getByRole('combobox', { name: label }));
  fireEvent.click(screen.getByRole('option', { name: option }));
}

/** One attachment row, exactly as the shell sends it: measured bytes, declared kind, unknown provenance. */
function attachment(over: Partial<ReleaseAttachmentDto> = {}): ReleaseAttachmentDto {
  return {
    attachmentId: 'att-1',
    fileName: 'app.bin',
    displayName: 'app.bin',
    byteSize: 960,
    sha256: 'a'.repeat(64),
    kind: 'bin',
    kindBasis: 'declared',
    provenance: 'unknown',
    ...over,
  };
}

/** The set the shell reports, with the run it compared it against and the answer it gave. */
function attachmentSet(
  rows: readonly ReleaseAttachmentDto[],
  gateRunId: string | null = null,
  matchesGateRun: boolean | null = null,
): ReleaseAttachmentsDto {
  return { attachments: [...rows], gateRunId, matchesGateRun };
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
  // The attachment surface reads empty by default: a screen that never had a file chosen must not render
  // one, and the tests below hand it rows the way the shell would.
  attachFileMock.mockResolvedValue(ok(null));
  readAttachmentsMock.mockResolvedValue(ok(attachmentSet([])));
  inspectReadinessMock.mockResolvedValue(ok(null));
  removeAttachmentMock.mockResolvedValue(ok([]));
  declareKindMock.mockResolvedValue(ok(attachment()));
});

describe('Release navigation', () => {
  it('does not attach a late visual export outcome to a newer Gate run', async () => {
    await openRelease();
    await runGate();
    let finish!: (value: Awaited<ReturnType<typeof exportGateVisual>>) => void;
    exportGateVisualMock.mockReturnValueOnce(new Promise((resolve) => { finish = resolve; }));
    fireEvent.click(await exportItem('Export SVG'));
    await waitFor(() => expect(exportGateVisualMock).toHaveBeenCalled());
    runGateMock.mockResolvedValueOnce(ok(gateRun({ runId: 'gate-new-subject' })));
    await act(async () => { fireEvent.click(screen.getByRole('button', { name: 'Run Gate' })); });
    await act(async () => { finish(ok({ status: 'written', fileName: 'old-gate-report.svg', format: 'svg' })); });
    expect(screen.queryByText(/old-gate-report/)).toBeNull();
    expect((await exportItem('Export SVG')).disabled).toBe(false);
  });

  it('says nothing was written when the Gate run changed while the Save dialog was open', async () => {
    await openRelease();
    await runGate();
    exportGateVisualMock.mockResolvedValueOnce({ ok: false, envelope: { code: 'ERR-EXPORT-6002', message: 'stale', operationId: 'op-s', details: null, remediation: null } });
    fireEvent.click(await exportItem('Export PNG'));
    expect(await screen.findByText(/Gate run changed while the Save dialog was open, so nothing was written/)).toBeDefined();
  });

  it('offers a standalone visual report for a current Gate run without changing Bundle export', async () => {
    await openRelease();
    await runGate();
    const png = await exportItem('Export PNG');
    const region = screen.getByRole('group', { name: 'Standalone visual report' });
    expect(within(region).getByRole('button', { name: 'Export SVG' })).toBeDefined();
    expect(within(region).getByRole('button', { name: 'Export print-ready HTML' })).toBeDefined();
    exportGateVisualMock.mockResolvedValue(ok({ status: 'written', fileName: 'firmwaresight-gate.png', format: 'png' }));
    fireEvent.click(png);
    await waitFor(() => expect(exportGateVisualMock).toHaveBeenCalledWith(`gate-${'a'.repeat(64)}`, 'snap-target', 'png'));
    expect(exportBundleMock).not.toHaveBeenCalled();
  });

  it('lists the five stages this build has and no others', async () => {
    await openRelease();
    const rail = screen.getByRole('navigation', { name: 'Pages' });
    expect(within(rail).getAllByRole('button').map((item) => item.textContent)).toEqual([
      'Overview',
      'Analyze',
      'Compare',
      'Release',
      'History',
    ]);
    const words = document.body.textContent ?? '';
    // P4 makes `Bundle` a word this build has earned, so it is no longer banned here: §48 attaches the
    // bundle *under* Release, and what §58 forbids is a navigation verb for a stage that does not exist.
    // R5 names this page "Release" and the stored-history page "History" (spec §3.2), and U1 added
    // Overview, which summarizes runs this build already stores. What is still not built gets none of its
    // words.
    for (const stage of ['Settings', 'Pricing', 'Cloud']) {
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

describe('project readiness preflight', () => {
  function readiness(overrides: Partial<ProjectReadinessDto> = {}): ProjectReadinessDto {
    return {
      scope: 'Configuration and workspace observation only; this is not a Gate result.',
      projectName: 'Controller v2',
      schemaVersion: 1,
      policySha256: 'e'.repeat(64),
      matchesLoadedPolicy: true,
      warnings: [],
      unknownKeyCount: 0,
      gitAvailable: false,
      gitHeadCommit: null,
      gitClean: null,
      gitUnknownReason: 'this directory is not a Git repository',
      releaseNotesStatus: 'missing',
      releaseNotesSha256: null,
      releaseNotesReason: null,
      declaredArtifactRequirements: ['elf', 'bin', 'hex'],
      buildEvidenceChecked: false,
      ...overrides,
    };
  }

  it('does not label an unloaded project as ready or invoke inspection', async () => {
    await openRelease();
    expect(await screen.findByRole('region', { name: 'Project readiness' })).toBeTruthy();
    expect(screen.getByText('No project is loaded. Open a project config to inspect its workspace.')).toBeTruthy();
    expect(inspectReadinessMock).not.toHaveBeenCalled();
  });

  it('shows typed workspace Unknown and declared artifact requirements after loading a project', async () => {
    openConfigMock.mockResolvedValueOnce(ok(project()));
    inspectReadinessMock.mockResolvedValueOnce(ok(readiness()));
    await openRelease();
    fireEvent.click(screen.getByRole('button', { name: 'Open project config' }));

    const region = await screen.findByRole('region', { name: 'Project readiness' });
    expect(await within(region).findByText('Controller v2')).toBeTruthy();
    expect(region.textContent).toContain('Unknown');
    expect(region.textContent).toContain('bin, hex');
    expect(region.textContent).toContain('attached evidence is not checked');
    expect(region.textContent).toContain('Build evidence is not checked');
    expect(region.textContent).not.toContain('D:');
    expect(region.querySelector('svg[aria-hidden="true"]')).toBeTruthy();
    expect(region.textContent).not.toContain('PASS');
  });

  it('keeps earlier observations visible but explicitly stale when a recheck fails', async () => {
    let finishRefresh!: (outcome: IpcOutcome<ProjectReadinessDto | null>) => void;
    openConfigMock.mockResolvedValueOnce(ok(project()));
    inspectReadinessMock
      .mockResolvedValueOnce(ok(readiness({ projectName: 'previous project' })))
      .mockImplementationOnce(() => new Promise((resolve) => { finishRefresh = resolve; }));
    await openRelease();
    fireEvent.click(screen.getByRole('button', { name: 'Open project config' }));

    const region = await screen.findByRole('region', { name: 'Project readiness' });
    expect(await within(region).findByText('previous project')).toBeTruthy();
    fireEvent.click(within(region).getByRole('button', { name: 'Recheck project' }));
    const refreshing = within(region).getByRole('button', { name: 'Rechecking…' }) as HTMLButtonElement;
    expect(refreshing.disabled).toBe(true);
    expect(within(region).getByText('Inspecting the loaded project…')).toBeTruthy();
    finishRefresh(fail(MALFORMED));
    expect(await within(region).findByText('Recheck failed. Any earlier observations are stale.')).toBeTruthy();
    expect(within(region).getByText('previous project')).toBeTruthy();
  });

  it('ignores a late inspection result after the loaded project changes', async () => {
    const pending: Array<(outcome: IpcOutcome<ProjectReadinessDto | null>) => void> = [];
    inspectReadinessMock.mockImplementation(
      () => new Promise((resolve) => pending.push(resolve)),
    );
    openConfigMock
      .mockResolvedValueOnce(ok(project({ projectName: 'first-project' })))
      .mockResolvedValueOnce(ok(project({ projectName: 'second-project' })));
    await openRelease();
    fireEvent.click(screen.getByRole('button', { name: 'Open project config' }));
    await waitFor(() => expect(pending).toHaveLength(1));
    fireEvent.click(screen.getByRole('button', { name: 'Open project config' }));
    await waitFor(() => expect(pending).toHaveLength(2));

    pending[1]?.(ok(readiness({ projectName: 'second-project' })));
    const region = screen.getByRole('region', { name: 'Project readiness' });
    await waitFor(() => expect(within(region).getByText('second-project')).toBeTruthy());
    pending[0]?.(ok(readiness({ projectName: 'first-project' })));
    await waitFor(() => expect(within(region).getByText('second-project')).toBeTruthy());
    expect(within(region).queryByText('first-project')).toBeNull();
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
    await screen.findAllByText('Controller v2');
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
    await screen.findAllByText('Controller v2');
    const words = document.body.textContent ?? '';
    expect(words).not.toMatch(/[A-Za-z]:[\\/]/);
    expect(words).not.toContain('secret-project');
  });

  it('reports a refused config as a typed error and keeps the loaded project', async () => {
    openConfigMock.mockResolvedValue(ok(project()));
    await openRelease();
    fireEvent.click(screen.getByRole('button', { name: 'Open project config' }));
    await screen.findAllByText('Controller v2');

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
    await screen.findAllByText('Controller v2');
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
    await screen.findAllByText('Controller v2');
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
    await screen.findByRole('combobox', { name: 'Current build' });
    await waitFor(() => expect(pickerValue('Current build')).toBe('snap-target'));
  });

  it('offers no baseline at all until one is asked for', async () => {
    await openRelease();
    await screen.findByRole('combobox', { name: 'Baseline (optional)' });
    expect(pickerValue('Baseline (optional)')).toBe('');
    expect(document.body.textContent).toContain('the growth rule answers Unknown');
    await runGate();
    expect(runGateMock.mock.calls[0]?.[0]).toEqual({
      snapshotId: 'snap-target',
      baselineSnapshotId: null,
    });
  });

  it('will not offer the current build as its own baseline', async () => {
    await openRelease();
    await screen.findByRole('combobox', { name: 'Current build' });
    await waitFor(() => expect(pickerValue('Current build')).toBe('snap-target'));
    fireEvent.click(screen.getByRole('combobox', { name: 'Baseline (optional)' }));
    expect(screen.queryByRole('option', { name: /app\.elf/ })).toBeNull();
    expect(screen.getByRole('option', { name: /app-old\.elf/ })).toBeDefined();
    // R4 D-R4-3: "no baseline" is a choice in the list, and the one shown while none is chosen.
    expect(screen.getByRole('option', { name: 'No baseline' }).getAttribute('aria-selected')).toBe('true');
  });

  it('sends the pair the reader chose', async () => {
    await openRelease();
    await screen.findByRole('combobox', { name: 'Baseline (optional)' });
    choose('Baseline (optional)', /app-old\.elf/);
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
    const headings = () =>
      screen.getAllByRole('heading', { level: 3 }).map((node) => (node.textContent ?? '').replace(/\d+$/, ''));
    expect(headings()).toEqual(['Block', 'Review', 'Unknown', 'Pass', 'Not applicable']);
    // R4: the number blocks are the Evidence view, in the same order as before.
    openView('Evidence');
    expect(headings()).toEqual([
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
    expect(within(row).getByText(/Reduce the footprint/)).toBeDefined();
    // R4: the evidence locators are the finding's basis, in the Inspector.
    fireEvent.click(within(row).getByRole('button', { name: 'Show the basis for memory.flash_budget' }));
    const inspector = await screen.findByRole('complementary', { name: 'Evidence' });
    expect(within(inspector).getAllByText('policy:memory.flash_budget=4096').length).toBeGreaterThan(0);
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
    await screen.findByRole('tab', { name: 'Evidence' });
    openView('Evidence');
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
    await screen.findByRole('tab', { name: 'Evidence' });
    openView('Evidence');
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
    await screen.findByRole('tab', { name: 'Evidence' });
    openView('Evidence');
    expect(await screen.findByText('No workspace facts were observed')).toBeDefined();
    expect(screen.getByText('git is not installed')).toBeDefined();
    expect(document.body.textContent).not.toContain('Workspace HEAD');
  });
});

describe('The numbers behind the findings', () => {
  it('names the set that carried each required kind, beside the one nothing carried', async () => {
    await openRelease();
    await runGate();
    await screen.findByText('memory.flash_budget');
    openView('Evidence');
    const table = block('Required artifacts').querySelector('table');
    if (table === null) {
      throw new Error('the artifact table is missing');
    }
    expect(within(table).getByText('map')).toBeDefined();
    expect(within(table).getAllByText('Required').length).toBe(3);
    expect(within(table).getByText('Missing')).toBeDefined();
    // The row that started this: an attached BIN satisfies the requirement, so the table has to say
    // where its bytes came from instead of calling it `Missing` under a verdict that counted it.
    expect(within(table).getByText('Release attachment')).toBeDefined();
    expect(within(table).getByText('Analyzed build')).toBeDefined();
    const cells = (kind: string): string[] => {
      const row = within(table).getByText(kind).closest('tr');
      if (row === null) {
        throw new Error(`the ${kind} row is missing`);
      }
      return Array.from(row.querySelectorAll('td')).map((td) => td.textContent ?? '');
    };
    expect(cells('bin')).toEqual(['Required', 'Release attachment', expect.stringMatching(/^c{8}/), '4,096 bytes']);
    expect(cells('map')[1]).toBe('Missing');
  });

  it('does not advertise analysis it has not done', async () => {
    await openRelease();
    await runGate();
    await screen.findByText('memory.flash_budget');
    openView('Evidence');
    // §9's replacement caption: the required BIN/HEX is satisfied by an attached file, and the sentence has
    // to say what that does *not* include. The old words ("checked for presence only") described the rule
    // before an attachment could satisfy it.
    expect(block('Required artifacts').textContent).toContain(
      'are satisfied by release files you attach',
    );
    expect(block('Required artifacts').textContent).toContain('does not analyze them');
    const words = document.body.textContent ?? '';
    expect(words).not.toMatch(/BIN files are analyzed/i);
  });

  it('shows a budget with its excess, and no invented number for an unknown one', async () => {
    await openRelease();
    await runGate();
    await screen.findByText('memory.flash_budget');
    openView('Evidence');
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

  it('names a memory basis by what it means, not by the Core word behind it', async () => {
    // F2 found this column printing `MapRegionAndElfLoad` in a human-facing cell while Analyze, Compare
    // and History all said what the evidence was (F2R-03). The value the shell sends is not a display
    // decision; the caption is, so the cell keeps the meaning and loses the identifier.
    const run = gateRun();
    const tables = run.tables;
    if (tables === null) {
      throw new Error('the fixture run carries no tables');
    }
    tables.budgets = tables.budgets.map((row, index) => ({
      ...row,
      basis: index === 0 ? 'MapRegionAndElfLoad' : 'region-config+elf-load',
    }));
    runGateMock.mockResolvedValue(ok(run));
    await openRelease();
    await runGate();
    await screen.findByText('memory.flash_budget');
    openView('Evidence');
    const table = block('Memory budgets').querySelector('table');
    if (table === null) {
      throw new Error('the budget table is missing');
    }
    expect(within(table).getByText('MAP regions + ELF load evidence')).toBeDefined();
    // The wire spelling is a portable value elsewhere and a caption here, so both arrive as prose.
    expect(within(table).getByText('configured regions + ELF load evidence')).toBeDefined();
    expect(table.textContent).not.toContain('MapRegionAndElfLoad');
    expect(table.textContent).not.toContain('region-config+elf-load');
  });

  it('says plainly what it cannot name, and what was never recorded', async () => {
    // A basis from a future Core release must not be dressed up as one this page understands, and a
    // budget with no basis at all keeps the sentence that says so rather than going blank (§11).
    await openRelease();
    await runGate();
    await screen.findByText('memory.flash_budget');
    openView('Evidence');
    const table = block('Memory budgets').querySelector('table');
    if (table === null) {
      throw new Error('the budget table is missing');
    }
    const flash = within(table).getByText('FLASH').closest('tr');
    const ram = within(table).getByText('RAM').closest('tr');
    if (flash === null || ram === null) {
      throw new Error('a budget row is missing');
    }
    // The fixture's `SectionHeaders` is a word the domain has never sent.
    expect(within(flash).getByText('Unrecognized evidence basis')).toBeDefined();
    expect(flash.textContent).not.toContain('SectionHeaders');
    expect(within(ram).getByText('no basis recorded')).toBeDefined();
  });

  it('moves the growth numbers P2 computed instead of recomputing them', async () => {
    await openRelease();
    await runGate();
    await screen.findByText('memory.flash_budget');
    openView('Evidence');
    const table = block('Growth against the baseline').querySelector('table');
    if (table === null) {
      throw new Error('the growth table is missing');
    }
    // R4: a delta is neutral ▲▼ text with its spoken words (ADR-0034 D1.4).
    expect(table.textContent).toContain('▲512 bytes');
    expect(within(table).getByText('increase of 512 bytes')).toBeDefined();
    expect(within(table).getByText('4,608 bytes')).toBeDefined();
    expect(within(table).getByText('256 bytes')).toBeDefined();
    expect(within(table).getByText('REVIEW')).toBeDefined();
  });

  it('relabels the same figures in KiB and asks the shell for nothing', async () => {
    await openRelease();
    await runGate();
    await screen.findByText('memory.flash_budget');
    openView('Evidence');
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
    openView('Evidence');
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
    expandAll();
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
    await screen.findByRole('tab', { name: 'Evidence' });
    openView('Evidence');
    expect(await screen.findByText('Disposition: REVIEW')).toBeDefined();
    openView('Findings');

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
    openView('Evidence');
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
    openView('Evidence');
    fireEvent.click(screen.getByRole('button', { name: 'Read the stored record' }));
    expect(await screen.findByText(/^This run was read back from history/)).toBeDefined();
    expect(screen.queryByRole('heading', { level: 2, name: /What the findings were judged on/ })).toBeNull();
    // R4: a record without tables says so in an Unknown notice rather than leaving a gap.
    expect(screen.getByRole('heading', { level: 2, name: 'Evidence tables not stored' })).toBeDefined();
    openView('Findings');
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
    await screen.findByRole('tab', { name: 'Bundle' });
    openView('Bundle');
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
    const heading = screen.getByRole('heading', { level: 2, name: 'Release Bundle' });
    // §58: the rail is still the workflow stages. A bundle is a section of Release, not a fifth
    // doorway, so the only new verb this page has is the one inside it.
    const rail = screen.getByRole('navigation', { name: 'Pages' });
    expect(within(rail).getAllByRole('button').map((item) => item.textContent)).toEqual([
      'Overview',
      'Analyze',
      'Compare',
      'Release',
      'History',
    ]);
    // §58's claim is about actions, not nouns: the rail holds pages, never the bundle verbs that live
    // inside Release. The old `/bundle/i` check was dropped when the history page carried "Bundle" in its
    // name (U1); it is "History" since R5, and what must stay absent is still a doorway that *does*
    // something.
    expect(within(rail).queryByRole('button', { name: /build|export|prepare/i })).toBeNull();
    expect(heading.closest('nav')).toBeNull();
  });

  it('keeps the bundle closed until the disposition says PASS, and says why', async () => {
    await openRelease();
    await runGate();
    // The default run's disposition is BLOCK, so nothing here is offerable — and the reason is on the
    // screen rather than inferred from a grey button (prompt §58).
    await screen.findByRole('tab', { name: 'Bundle' });
    openView('Bundle');
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
    await screen.findByText('diff.growth');
    expect(findingRow('diff.growth').querySelector('div')?.textContent).toContain('REVIEW');
    openView('Evidence');
    expect(await screen.findByText('Disposition: PASS')).toBeDefined();
    openView('Bundle');
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
    await screen.findByRole('tab', { name: 'Bundle' });
    openView('Bundle');
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

    const group = await screen.findByRole('dialog', { name: 'Replace the existing bundle?' });
    expect(within(group).getByRole('button', { name: `Replace the existing bundle named ${BUNDLE_FOLDER}` }))
      .toBeDefined();
    // The first press is unambiguous: overwrite false, and nothing on disk has changed.
    expect(exportBundleMock.mock.calls).toEqual([['bundle-4021-1', 'dst-4021-1', false]]);

    fireEvent.click(within(group).getByRole('button', { name: 'Keep it' }));
    await waitFor(() =>
      expect(document.body.textContent).toContain('Kept the bundle that was there. Nothing was written.'),
    );
    expect(screen.queryByRole('dialog', { name: 'Replace the existing bundle?' })).toBeNull();
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
    const group = await screen.findByRole('dialog', { name: 'Replace the existing bundle?' });
    fireEvent.click(
      within(group).getByRole('button', { name: `Replace the existing bundle named ${BUNDLE_FOLDER}` }),
    );

    const result = await screen.findByRole('heading', { level: 3, name: 'Bundle created' });
    expect(exportBundleMock.mock.calls[1]).toEqual(['bundle-4021-1', 'dst-4021-1', true]);
    expect(
      (result.parentElement as HTMLElement).textContent,
    ).toContain('This export replaced the bundle that was there, under your confirmation.');
  });

  it('never offers a replace decision for an occupied folder this engine did not write', async () => {
    // The engine's real order (`bundle.rs:382-393`): an existing destination answers 6106 whether or
    // not it is a recognizable bundle, and only a confirmed overwrite meets 6107. So a foreign
    // folder reaches this page as 6106 - the path the test above, which mocks 6107 on the first
    // press, never exercised, and the path E2E-F002 found asserting a bundle that is not there.
    chooseDestinationMock.mockResolvedValue(ok(bundleDestination({ exists: true, recognizableBundle: false })));
    exportBundleMock.mockResolvedValue(fail(DESTINATION_TAKEN));
    await prepared();
    await chosen();

    // What is true: occupied, not written by this engine, not going to be replaced.
    expect(document.body.textContent).toContain('it is not a bundle this engine wrote');
    expect(document.body.textContent).toContain('it will not be replaced.');
    // What must never be said about this folder, and what must never be offered.
    expect(document.body.textContent).not.toContain('already holds a FirmwareSight release bundle');
    expect(screen.queryByRole('dialog', { name: 'Replace the existing bundle?' })).toBeNull();
    expect(
      screen.queryByRole('button', { name: `Replace the existing bundle named ${BUNDLE_FOLDER}` }),
    ).toBeNull();

    // The guard is the disabled action, not a message that arrives after a doomed write attempt.
    expect(namedButton('Export bundle').disabled).toBe(true);
    fireEvent.click(namedButton('Export bundle'));
    expect(exportBundleMock).not.toHaveBeenCalled();

    // Recovery stays available: the destination is still this page's to change.
    expect(namedButton('Choose destination folder').disabled).toBe(false);
  });

  it('exports normally once a destination that is not foreign-occupied is chosen', async () => {
    chooseDestinationMock.mockResolvedValue(ok(bundleDestination({ exists: true, recognizableBundle: false })));
    exportBundleMock.mockResolvedValue(ok(bundleExport()));
    await prepared();
    await chosen();
    expect(namedButton('Export bundle').disabled).toBe(true);

    chooseDestinationMock.mockResolvedValue(ok(bundleDestination({ exists: false })));
    fireEvent.click(namedButton('Choose destination folder'));
    await waitFor(() =>
      expect(document.body.textContent).toContain('Nothing of that name is there yet.'),
    );

    expect(namedButton('Export bundle').disabled).toBe(false);
    fireEvent.click(namedButton('Export bundle'));
    await screen.findByRole('heading', { level: 3, name: 'Bundle created' });
    expect(exportBundleMock).toHaveBeenCalledTimes(1);
    expect(exportBundleMock.mock.calls[0]).toEqual(['bundle-4021-1', 'dst-4021-1', false]);
  });

  it('shows the engine refusal when a folder stops being a bundle between the choice and the replace', async () => {
    // The engine answers 6106 for any occupied destination and only reaches 6107 once a replace has
    // been authorized (`bundle.rs:382-393`), so this is how 6107 can actually arrive at this page:
    // the folder read as a bundle when it was chosen, and the write re-checked it and refused. The
    // refusal is shown, and the card that asked does not stay up repeating what was just denied.
    chooseDestinationMock.mockResolvedValue(ok(bundleDestination({ exists: true, recognizableBundle: true })));
    exportBundleMock
      .mockResolvedValueOnce(fail(DESTINATION_TAKEN))
      .mockResolvedValueOnce(fail(NOT_A_BUNDLE));
    await prepared();
    await chosen();
    fireEvent.click(namedButton('Export bundle'));

    const group = await screen.findByRole('dialog', { name: 'Replace the existing bundle?' });
    fireEvent.click(
      within(group).getByRole('button', { name: `Replace the existing bundle named ${BUNDLE_FOLDER}` }),
    );

    const panel = await screen.findByRole('alert', { name: 'Bundle step failed' });
    expect(within(panel).getByText('ERR-BUNDLE-6107')).toBeDefined();
    expect(screen.queryByRole('dialog', { name: 'Replace the existing bundle?' })).toBeNull();
    expect(exportBundleMock.mock.calls).toEqual([
      ['bundle-4021-1', 'dst-4021-1', false],
      ['bundle-4021-1', 'dst-4021-1', true],
    ]);
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
    // The button is in the DOM before the effect that focuses it has run, so a synchronous read of
    // activeElement could still see Export. Wait for the focus itself; it never arrives if the effect
    // is gone, which is what this test exists to catch.
    await waitFor(() => expect(document.activeElement).toBe(replace));
    fireEvent.click(screen.getByRole('button', { name: 'Keep it' }));
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Replace the existing bundle?' })).toBeNull());
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
    await waitFor(() => expect(runGateMock).toHaveBeenCalledTimes(2));
    await waitFor(() => expect(screen.getByRole('tab', { name: 'Findings' }).getAttribute('aria-selected')).toBe('true'));
    openView('Bundle');
    await waitFor(() => expect(screen.queryByRole('heading', { level: 3, name: 'Bundle preview' })).toBeNull());
    expect(document.body.textContent).not.toContain('Destination chosen for');
  });
});

describe('Accessibility', () => {
  it('names every field the page offers', async () => {
    openConfigMock.mockResolvedValue(ok(project()));
    await openRelease();
    fireEvent.click(screen.getByRole('button', { name: 'Open project config' }));
    await screen.findAllByText('Controller v2');
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

describe('the table layout contract', () => {
  // The same rule `details.test.tsx` pins for Analyze and `compare.test.tsx` pins for Compare, applied
  // to the four tables this page renders from one gate run: the scroll axis belongs to the wrapper, and
  // a reason or evidence-basis column keeps a measure instead of collapsing to one character.
  it('puts every gate table inside a scroll wrapper and keeps it a table', async () => {
    await openRelease();
    await runGate();
    await screen.findByRole('tab', { name: 'Evidence' });
    openView('Evidence');
    await screen.findByRole('heading', { level: 3, name: 'Required artifacts' });

    // R4: the gate tables are the shared DataTable, whose wrapper owns the scroll axis.
    const tables = screen.getAllByRole('table');
    expect(tables.length).toBe(3);
    for (const table of tables) {
      expect(table.tagName).toBe('TABLE');
      expect(table.parentElement?.className).toContain('wrap');
    }
  });
});

describe('U1P Release hierarchy', () => {
  it('gives the subject, the verdict, the settings line, then the three views', async () => {
    await openRelease();
    await runGate();

    // R4: the page names the build the run judged, the verdict leads, the settings that made the run are
    // one line, and the run itself is read in three views (spec §6.4).
    const verdict = await screen.findByRole('region', { name: 'Gate verdict' });
    const subject = screen.getByRole('region', { name: 'Subject of this page' });
    const settings = screen.getByRole('region', { name: 'Run settings' });
    const views = screen.getByRole('tablist', { name: 'Release views' });
    const findings = screen.getByRole('region', { name: 'Findings by state' });
    expect(precedes(subject, verdict)).toBe(true);
    expect(precedes(verdict, settings)).toBe(true);
    expect(precedes(settings, views)).toBe(true);
    expect(precedes(views, findings)).toBe(true);
    expect(within(views).getAllByRole('tab').map((tab) => tab.textContent)).toEqual(['Findings', 'Evidence', 'Bundle']);

    openView('Evidence');
    const readiness = screen.getByRole('region', { name: 'FirmwareSight policy readiness' });
    const evidence = screen.getByRole('region', { name: 'What the findings were judged on' });
    expect(precedes(readiness, evidence)).toBe(true);
    openView('Bundle');
    expect(screen.getByRole('region', { name: 'Release Bundle' })).toBeDefined();
  });

  it('moves focus to the verdict after a run the reader asked for', async () => {
    await openRelease();
    await runGate();
    const verdict = await screen.findByRole('region', { name: 'Gate verdict' });
    await waitFor(() => expect(document.activeElement).toBe(verdict));
  });

  it('folds the settings into one line after a run, and opens them in place', async () => {
    await openRelease();
    await runGate();
    await screen.findByRole('region', { name: 'Gate verdict' });

    // R4 (spec §6.4): the line says what the next run will judge, and the steps are one press away.
    const settings = screen.getByRole('region', { name: 'Run settings' });
    expect(settings.textContent).toContain('Build app.elf');
    expect(settings.textContent).toContain('No baseline');
    expect(settings.textContent).toContain('Stated default policy');
    expect(settings.textContent).toContain('0 attachment(s)');
    expect(within(settings).getByRole('button', { name: 'Run Gate' })).toBeDefined();
    expect(screen.queryByRole('region', { name: 'Step 2 · Project policy' })).toBeNull();

    const edit = within(settings).getByRole('button', { name: 'Edit settings' });
    expect(edit.getAttribute('aria-expanded')).toBe('false');
    fireEvent.click(edit);
    const policy = screen.getByRole('region', { name: 'Step 2 · Project policy' });
    const builds = screen.getByRole('region', { name: 'Step 1 · Build and baseline' });
    expect(within(policy).getByRole('button', { name: 'Edit policy' })).toBeDefined();
    expect(within(builds).getByRole('combobox', { name: 'Current build' })).toBeDefined();
    expect(within(settings).getByRole('button', { name: 'Hide settings' }).getAttribute('aria-expanded')).toBe('true');
  });

  it('leads with the settings it has to be given, while there is no run to report', async () => {
    // An empty store, so no build is ever chosen. With the default two builds the page names one as soon
    // as the stored list arrives, and the run hint below goes with it: a read that found the hint was
    // reading the page before that list landed (L23).
    candidatesMock.mockResolvedValue(ok(candidatePage([])));
    await openRelease();
    await screen.findByText(/^Nothing is stored yet, so there is nothing to gate\./);

    // With no run the verdict band renders nothing, so the page's first block is the one that can
    // actually be used. Asserting the empty case is what keeps the reordering from becoming a claim
    // that the verdict is always on screen.
    expect(screen.queryByRole('region', { name: 'Gate verdict' })).toBeNull();
    const policy = screen.getByRole('region', { name: 'Step 2 · Project policy' });
    const builds = screen.getByRole('region', { name: 'Step 1 · Build and baseline' });
    const attachments = screen.getByRole('region', { name: 'Step 3 · Release attachments' });
    expect(precedes(builds, policy)).toBe(true);
    expect(precedes(policy, attachments)).toBe(true);
    expect(screen.getByText('Choose a build in step 1 to run the Gate.')).toBeDefined();
  });
});

describe('Release attachments', () => {
  it('lists a chosen file with its measured bytes, its full digest and the limit that comes with them', async () => {
    readAttachmentsMock.mockResolvedValue(ok(attachmentSet([attachment()])));
    await openRelease();

    const region = await screen.findByRole('region', { name: 'Step 3 · Release attachments' });
    expect(within(region).getByText('app.bin')).toBeDefined();
    expect(within(region).getByText('960 bytes')).toBeDefined();
    // §9 asks for the full SHA-256 on this surface, so nothing here truncates it the way a run id is.
    expect(within(region).getByText('a'.repeat(64))).toBeDefined();
    expect(
      within(region).getByText('Bytes observed · kind declared · provenance unknown'),
    ).toBeDefined();
    expect(within(region).getByText('Withdraw')).toBeDefined();
    const words = region.textContent ?? '';
    expect(words).toContain('Bytes verified; build provenance not verified.');
    expect(words).toContain('BIN/HEX contents are not analyzed.');
    expect(words).not.toMatch(/analyz(e|es|ing) (the )?(BIN|HEX|file|bytes)/i);
  });

  it('names no folder on the attachment row, in either direction', async () => {
    readAttachmentsMock.mockResolvedValue(
      ok(
        attachmentSet([
          attachment({ fileName: 'motor-controller.bin', displayName: 'motor-controller.bin' }),
        ]),
      ),
    );
    await openRelease();
    const region = await screen.findByRole('region', { name: 'Step 3 · Release attachments' });
    const words = region.textContent ?? '';
    expect(words).toContain('motor-controller.bin');
    // A leaf is what a person recognizes. A folder on the row would be the shell's leak, not the owner's
    // mistake, and the DTO has no field for one: no separator, no drive root, and the file cell is the
    // leaf alone.
    expect(within(region).getByText('motor-controller.bin')).toBeDefined();
    expect(words).not.toContain('\\');
    expect(words).not.toMatch(/[A-Za-z]:[\\/]/);
    expect(words).not.toContain('fwsight');
  });

  it('shows an attachment by the name its owner chose, not by the name the bundle files it under', async () => {
    // `N01-F02`: `sanitize_leaf_name` is what a portable folder may carry, and it folds `固件镜像.bin` to
    // `bin`. The row is the owner's, so it shows the leaf they picked; the archive name keeps its role in
    // the bundle, where a path has to be safe before it can be copied.
    readAttachmentsMock.mockResolvedValue(
      ok(attachmentSet([attachment({ fileName: 'bin', displayName: '固件镜像.bin' })])),
    );
    await openRelease();
    const region = await screen.findByRole('region', { name: 'Step 3 · Release attachments' });
    const cell = region.querySelector('tbody th[scope="row"]');
    expect(cell?.textContent).toBe('固件镜像.bin');
    expect(region.textContent ?? '').not.toContain('motor-controller');

    // The two names are two roles, so a screen reader asking which file a picker belongs to is asking
    // about the one the person sees, not the one the archive chose for them.
    const picker = within(region).getByLabelText('Declared kind for 固件镜像.bin');
    expect(picker).toBeDefined();
  });

  it('says what an empty set means, in words the release owner can act on', async () => {
    await openRelease();
    const region = await screen.findByRole('region', { name: 'Step 3 · Release attachments' });
    expect(region.textContent).toContain('Nothing attached.');
    expect(region.textContent).toContain('is not analyzed');
  });

  it('asks the shell for a file when Attach is pressed, and shows the row the shell measured', async () => {
    readAttachmentsMock
      .mockResolvedValueOnce(ok(attachmentSet([])))
      .mockResolvedValueOnce(ok(attachmentSet([attachment()])));
    attachFileMock.mockResolvedValue(ok(attachment()));
    await openRelease();
    const region = await screen.findByRole('region', { name: 'Step 3 · Release attachments' });

    fireEvent.click(within(region).getByRole('button', { name: 'Attach' }));
    await waitFor(() => expect(attachFileMock).toHaveBeenCalledTimes(1));
    await waitFor(() => expect(within(region).getByText('app.bin')).toBeDefined());
  });

  it('treats a cancelled dialog as no change at all', async () => {
    await openRelease();
    const region = await screen.findByRole('region', { name: 'Step 3 · Release attachments' });
    const readsBefore = readAttachmentsMock.mock.calls.length;

    attachFileMock.mockResolvedValue(ok(null));
    fireEvent.click(within(region).getByRole('button', { name: 'Attach' }));
    await waitFor(() => expect(attachFileMock).toHaveBeenCalledTimes(1));

    // Cancel is a decision: no re-read, no invalidation, no error, and the empty row list stays as it was.
    expect(readAttachmentsMock.mock.calls.length).toBe(readsBefore);
    expect(within(region).queryByRole('alert')).toBeNull();
    expect(region.textContent).toContain('Nothing attached.');
  });

  it('drops the verdict the moment the set it judged is no longer the set on the page', async () => {
    await openRelease();
    await runGate();
    await screen.findByRole('region', { name: 'Gate verdict' });
    expect(screen.getByRole('region', { name: 'Findings by state' })).toBeDefined();

    readAttachmentsMock.mockResolvedValue(ok(attachmentSet([attachment()])));
    attachFileMock.mockResolvedValue(ok(attachment()));
    fireEvent.click(screen.getByRole('button', { name: 'Edit settings' }));
    const attached = await screen.findByRole('region', { name: 'Step 3 · Release attachments' });
    fireEvent.click(within(attached).getByRole('button', { name: 'Attach' }));

    // §4.2's order, on the surface a person stands at: a run that bound no file cannot be a verdict about
    // one file, so the record goes and the page asks again.
    await waitFor(() =>
      expect(screen.queryByRole('region', { name: 'Gate verdict' })).toBeNull(),
    );
    const again = await screen.findByRole('region', { name: 'Step 3 · Release attachments' });
    await waitFor(() => expect(within(again).getByText('app.bin')).toBeDefined());
  });

  it('says which run a changed set no longer matches, in the shell’s own comparison', async () => {
    readAttachmentsMock.mockResolvedValue(
      ok(attachmentSet([attachment()], 'gate-0123456789abcdef', false)),
    );
    await openRelease();
    const region = await screen.findByRole('region', { name: 'Step 3 · Release attachments' });
    const stale = within(region).getByRole('status');
    expect(stale.textContent).toContain('is not the set');
    expect(stale.textContent).toContain('Run the Gate again');
  });

  it('offers only the three kinds a release may declare, and sends the owner’s choice to the shell', async () => {
    readAttachmentsMock.mockResolvedValue(ok(attachmentSet([attachment()])));
    await openRelease();
    const region = await screen.findByRole('region', { name: 'Step 3 · Release attachments' });
    const picker = within(region).getByRole('combobox', {
      name: 'Declared kind for app.bin',
    }) as HTMLSelectElement;
    expect(within(picker)
      .getAllByRole('option')
      .map((option) => option.textContent)).toEqual(['bin', 'hex', 'unknown']);
    expect(picker.value).toBe('bin');

    declareKindMock.mockResolvedValue(ok(attachment({ kind: 'hex' })));
    fireEvent.change(picker, { target: { value: 'hex' } });
    await waitFor(() =>
      expect(declareKindMock).toHaveBeenCalledWith('att-1', 'hex'),
    );
  });

  it('withdraws one file and shows the set that remains', async () => {
    readAttachmentsMock
      .mockResolvedValueOnce(ok(attachmentSet([attachment()])))
      .mockResolvedValueOnce(ok(attachmentSet([])));
    await openRelease();
    const region = await screen.findByRole('region', { name: 'Step 3 · Release attachments' });
    fireEvent.click(within(region).getByRole('button', { name: 'Withdraw' }));
    await waitFor(() => expect(removeAttachmentMock).toHaveBeenCalledWith('att-1'));
    await waitFor(() => expect(region.textContent).toContain('Nothing attached.'));
  });

  it('reports a refused attachment step with the shell’s sentence and keeps the rows it had', async () => {
    readAttachmentsMock.mockResolvedValue(ok(attachmentSet([attachment()])));
    declareKindMock.mockResolvedValue(
      fail(ENVELOPE('ERR-INPUT-0001', 'That kind cannot be declared for an attachment.')),
    );
    await openRelease();
    const region = await screen.findByRole('region', { name: 'Step 3 · Release attachments' });
    fireEvent.change(within(region).getByRole('combobox', { name: 'Declared kind for app.bin' }), {
      target: { value: 'unknown' },
    });
    await waitFor(() =>
      expect(within(region).getByText('That kind cannot be declared for an attachment.')).toBeDefined(),
    );
    // The refusal changed nothing: the row is still the one the shell reported.
    expect(within(region).getByText('app.bin')).toBeDefined();
  });
});

describe('R4 Release workbench', () => {
  /** Run the Gate on the pair the fixture run names, so the run on screen and the settings agree. */
  async function runOnFixturePair(): Promise<void> {
    await openRelease();
    await screen.findByRole('combobox', { name: 'Baseline (optional)' });
    choose('Baseline (optional)', /app-old\.elf/);
    await runGate();
    await screen.findByRole('region', { name: 'Gate verdict' });
  }

  const subjectBar = () => screen.getByRole('region', { name: 'Subject of this page' });

  it('names the judged build in the SubjectBar, and says when the settings have moved past it', async () => {
    await runOnFixturePair();
    expect(subjectBar().textContent).toContain('app.elf');
    expect(subjectBar().textContent).toContain('Latest');
    // A stored candidate reports no MAP, so the bar makes no MAP claim either way (D-R4-4).
    expect(subjectBar().textContent).not.toMatch(/MAP/);

    fireEvent.click(screen.getByRole('button', { name: 'Edit settings' }));
    choose('Baseline (optional)', /^No baseline$/);
    expect(subjectBar().textContent).toContain('Previous run kept · current settings not judged');
    expect(screen.getByText(/now name a different build or baseline than this run judged/)).toBeDefined();
    // The verdict is still the run's own: moving the settings judges nothing.
    expect(screen.getByRole('region', { name: 'Gate verdict' })).toBeDefined();
    expect(runGateMock).toHaveBeenCalledTimes(1);
  });

  it('says a failed attempt kept the previous run, in the SubjectBar', async () => {
    await runOnFixturePair();
    runGateMock.mockResolvedValueOnce(fail(STORAGE_MISS));
    await runGate();
    await screen.findByRole('alert', { name: 'Gate run failed' });
    expect(subjectBar().textContent).toContain('Gate run failed · previous run kept');
  });

  it('comes back to a run as the latest one, because the settings start from the run (D-R4-7)', async () => {
    await runOnFixturePair();
    fireEvent.click(screen.getByRole('button', { name: 'Analyze page' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Release page' }));
    await screen.findByRole('region', { name: 'Gate verdict' });
    expect(subjectBar().textContent).toContain('Latest');
    expect(screen.getByRole('region', { name: 'Run settings' }).textContent).toContain('Baseline app-old.elf');
  });

  it('starts PASS and N/A collapsed with their counts in view, and opens them on request', async () => {
    await runOnFixturePair();
    expect(screen.queryByText('git.clean')).toBeNull();
    const show = screen.getByRole('button', { name: 'Show 1 Pass finding(s)' });
    expect(show.getAttribute('aria-expanded')).toBe('false');
    expect(screen.getByRole('heading', { level: 3, name: /Pass/ }).textContent).toContain('1');
    fireEvent.click(show);
    expect(screen.getByText('git.clean')).toBeDefined();
    expect(screen.getByRole('button', { name: 'Hide the Pass findings' }).getAttribute('aria-expanded')).toBe('true');
    // What asks for a decision is never folded away.
    expect(screen.getByText('memory.flash_budget')).toBeDefined();
    expect(screen.getByText('diff.growth')).toBeDefined();
  });

  it('opens a finding basis in the Inspector and returns focus to the row', async () => {
    await runOnFixturePair();
    const button = screen.getByRole('button', { name: 'Show the basis for memory.flash_budget' });
    button.focus();
    fireEvent.click(button);
    const inspector = await screen.findByRole('complementary', { name: 'Evidence' });
    expect(inspector.textContent).toContain('Derived');
    expect(inspector.textContent).toContain('Effective severity');
    expect(inspector.textContent).toContain('evidence:ev-memory-flash');
    fireEvent.click(within(inspector).getByRole('button', { name: 'Close evidence panel' }));
    await waitFor(() => expect(document.activeElement).toBe(button));
  });

  it('classes an Unknown finding as Unknown in the Inspector, never as derived', async () => {
    await runOnFixturePair();
    fireEvent.click(screen.getByRole('button', { name: 'Show the basis for release.notes' }));
    const inspector = await screen.findByRole('complementary', { name: 'Evidence' });
    expect(within(inspector).getAllByText('Unknown').length).toBeGreaterThan(0);
    expect(inspector.textContent).not.toContain('Derived');
  });

  it('opens a budget row in the Inspector, with Unknown for the figure that was never measured', async () => {
    await runOnFixturePair();
    openView('Evidence');
    fireEvent.click(screen.getByRole('button', { name: 'Show the budget basis for RAM' }));
    const inspector = await screen.findByRole('complementary', { name: 'Evidence' });
    expect(inspector.textContent).toContain('memory.ram_budget');
    expect(inspector.textContent).toContain('Unknown');
    expect(inspector.textContent).toContain('2,048 bytes');
    expect(inspector.textContent).not.toContain('0 bytes');
  });

  it('writes an unmeasured growth delta as Unknown, never as "not present"', async () => {
    const run = gateRun();
    if (run.tables === null) {
      throw new Error('the fixture run carries no tables');
    }
    run.tables.growth = run.tables.growth.map((row) => ({ ...row, deltaBytes: null }));
    runGateMock.mockResolvedValue(ok(run));
    await runOnFixturePair();
    openView('Evidence');
    const table = block('Growth against the baseline').querySelector('table');
    if (table === null) {
      throw new Error('the growth table is missing');
    }
    expect(table.textContent).toContain('Unknown');
    expect(table.textContent).not.toContain('not present');
  });

  it('edits the policy in a Sheet that Esc closes, returning focus to Edit policy', async () => {
    await openRelease();
    const edit = screen.getByRole('button', { name: 'Edit policy' });
    edit.focus();
    fireEvent.click(edit);
    const sheet = await screen.findByRole('dialog', { name: 'Edit project policy' });
    expect(sheet.getAttribute('aria-modal')).toBe('true');
    expect(within(sheet).getByLabelText('FLASH budget (bytes)')).toBeDefined();
    expect(within(sheet).getByRole('button', { name: 'Save as firmwaresight.toml' })).toBeDefined();
    fireEvent.keyDown(document, { key: 'Escape' });
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Edit project policy' })).toBeNull());
    expect(document.activeElement).toBe(edit);
    expect(savePolicyMock).not.toHaveBeenCalled();
  });

  it('asks for the acceptance in a Sheet, with focus on the name field', async () => {
    await runOnFixturePair();
    fireEvent.click(screen.getByRole('button', { name: 'Accept review' }));
    const sheet = await screen.findByRole('dialog', { name: 'Accept the review of diff.growth' });
    await waitFor(() => expect(document.activeElement).toBe(within(sheet).getByLabelText('Who accepts this review')));
  });

  it('counts the findings that carry a REVIEW disposition, including the unknown ones (R4 smoke)', async () => {
    // The smoke found "0 findings wait on an explicit acceptance" under a REVIEW verdict: the two rows that
    // carried it were UNKNOWN, and `[gate.on_unknown]` made them reviews. The sentence counts the rows whose
    // effective severity is the verdict's word, which is Core's own field on each row.
    runGateMock.mockResolvedValue(
      ok(
        gateRun({
          overallEffectiveSeverity: 'REVIEW',
          dispositionEffectiveSeverity: 'REVIEW',
          counts: { pass: 0, review: 0, block: 0, unknown: 2, notApplicable: 0 },
          findings: [
            finding({ id: 'u1', ruleId: 'git.clean', state: 'UNKNOWN', effectiveSeverity: 'REVIEW' }),
            finding({ id: 'u2', ruleId: 'release.notes', state: 'UNKNOWN', effectiveSeverity: 'REVIEW' }),
          ],
        }),
      ),
    );
    await openRelease();
    await runGate();
    const verdict = await screen.findByRole('region', { name: 'Gate verdict' });
    expect(verdict.textContent).toContain('2 finding(s) wait on an explicit acceptance, 2 of them because their evidence is missing');
    expect(verdict.textContent).not.toContain('0 finding(s)');
  });

  it('keeps the stale note when the 6002 answer lands in the same tick as the new run (R4 review S2-3)', async () => {
    await runOnFixturePair();
    let finishExport!: (value: Awaited<ReturnType<typeof exportGateVisual>>) => void;
    exportGateVisualMock.mockReturnValueOnce(new Promise((resolve) => { finishExport = resolve; }));
    fireEvent.click(await exportItem('Export PNG'));
    await waitFor(() => expect(exportGateVisualMock).toHaveBeenCalled());
    let finishRun!: (value: Awaited<ReturnType<typeof runReleaseGate>>) => void;
    runGateMock.mockReturnValueOnce(new Promise((resolve) => { finishRun = resolve; }));
    fireEvent.click(screen.getByRole('button', { name: 'Run Gate' }));
    await act(async () => {
      finishExport({ ok: false, envelope: { code: 'ERR-EXPORT-6002', message: 'stale', operationId: 'op-s', details: null, remediation: null } });
      finishRun(ok(gateRun({ runId: 'gate-same-tick' })));
    });
    await screen.findByRole('region', { name: 'Gate verdict' });
    expect(screen.getByText(/Gate run changed while the Save dialog was open, so nothing was written/)).toBeDefined();
    expect(screen.queryByRole('alert', { name: 'Visual report export error' })).toBeNull();
  });

  it('returns focus to the acceptance record once the Accept button it came from is retired (R4 review S2-1)', async () => {
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
    const accept = await screen.findByRole('button', { name: 'Accept review' });
    accept.focus();
    fireEvent.click(accept);
    fireEvent.change(await screen.findByLabelText('Who accepts this review'), { target: { value: 'rosa' } });
    fireEvent.change(screen.getByLabelText('Why it is acceptable'), { target: { value: 'the growth is the new bootloader' } });
    fireEvent.click(screen.getByRole('button', { name: 'Record acceptance' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    const record = within(findingRow('diff.growth')).getByText('rosa').closest('[role="status"]');
    await waitFor(() => expect(document.activeElement).toBe(record));
  });

  it('keeps an edited policy draft when its Sheet is closed without saving (R4 review S2-2)', async () => {
    await openRelease();
    fireEvent.click(screen.getByRole('button', { name: 'Edit policy' }));
    fireEvent.change(await screen.findByLabelText('FLASH budget (bytes)'), { target: { value: '8192' } });
    fireEvent.keyDown(document, { key: 'Escape' });
    await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull());
    fireEvent.click(screen.getByRole('button', { name: 'Edit policy' }));
    expect(((await screen.findByLabelText('FLASH budget (bytes)')) as HTMLInputElement).value).toBe('8192');
    expect(savePolicyMock).not.toHaveBeenCalled();
  });

  it('keeps the run Latest when only reading the stored record fails (R4 review S3)', async () => {
    await runOnFixturePair();
    getRunMock.mockResolvedValue(fail(STORAGE_MISS));
    openView('Evidence');
    fireEvent.click(screen.getByRole('button', { name: 'Read the stored record' }));
    expect(await screen.findByRole('alert', { name: 'Reading the stored record failed' })).toBeDefined();
    expect(subjectBar().textContent).toContain('Latest');
    expect(screen.queryByRole('alert', { name: 'Gate run failed' })).toBeNull();
  });

  it('marks the run as kept once a loaded policy differs from the one it was judged under (R4 review S3)', async () => {
    await runOnFixturePair();
    expect(subjectBar().textContent).toContain('Latest');
    openConfigMock.mockResolvedValue(ok(project({ policySha256: 'f'.repeat(64) })));
    fireEvent.click(screen.getByRole('button', { name: 'Edit settings' }));
    fireEvent.click(screen.getByRole('button', { name: 'Open project config' }));
    await waitFor(() => expect(subjectBar().textContent).toContain('Previous run kept · current settings not judged'));
  });

  it('exports the run on screen as a Markdown reading copy (R6)', async () => {
    vi.mocked(exportGateMarkdown).mockResolvedValue(
      ok({ status: 'written', fileName: 'firmwaresight-gate-review.md', format: 'markdown' }),
    );
    await runOnFixturePair();
    fireEvent.click(await exportItem('Export Markdown'));
    await waitFor(() => expect(vi.mocked(exportGateMarkdown)).toHaveBeenCalledWith(gateRun().runId, 'snap-target'));
    expect(exportGateVisualMock).not.toHaveBeenCalled();
    expect(await screen.findByText('Wrote firmwaresight-gate-review.md.')).toBeDefined();
  });

  it('shows the stale note when Rust refuses a late export after the run changed (CI #123 rule)', async () => {
    await runOnFixturePair();
    let finish!: (value: Awaited<ReturnType<typeof exportGateVisual>>) => void;
    exportGateVisualMock.mockReturnValueOnce(new Promise((resolve) => { finish = resolve; }));
    fireEvent.click(await exportItem('Export PNG'));
    runGateMock.mockResolvedValueOnce(ok(gateRun({ runId: 'gate-newer' })));
    await act(async () => { fireEvent.click(screen.getByRole('button', { name: 'Run Gate' })); });
    await act(async () => {
      finish({ ok: false, envelope: { code: 'ERR-EXPORT-6002', message: 'stale', operationId: 'op-s', details: null, remediation: null } });
    });
    expect(screen.getByText(/Gate run changed while the Save dialog was open, so nothing was written/)).toBeDefined();
    expect(screen.queryByRole('alert', { name: 'Visual report export error' })).toBeNull();
  });
});
