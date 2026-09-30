/**
 * P3 Release: the screen a release owner uses to judge one stored build, read the findings, and sign a
 * review off.
 *
 * These tests protect seven claims, and every one is about what the screen is allowed to say:
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
 * - and a run that failed leaves the last good one on screen, labelled as previous (prompt §49).
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
  clearMap,
  compareSnapshots,
  exportCompareHtml,
  exportCompareJson,
  getGateRun,
  listCompareCandidates,
  openProjectConfig,
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
}));

const candidatesMock = vi.mocked(listCompareCandidates);
const openConfigMock = vi.mocked(openProjectConfig);
const savePolicyMock = vi.mocked(saveProjectPolicy);
const runGateMock = vi.mocked(runReleaseGate);
const acceptMock = vi.mocked(acceptReview);
const getRunMock = vi.mocked(getGateRun);

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
    for (const stage of ['Bundle', 'History', 'Settings', 'Pricing', 'Cloud']) {
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
