/**
 * P2 Compare: the screen a reader uses to pick two stored builds and read what moved between them.
 *
 * These tests exist to protect six claims, and they are all about what the screen is allowed to say:
 *
 * - the rail offers the two stages this build has, and no doorway to a stage it does not;
 * - nothing is compared until the reader asks, and a pair of one is refused rather than invented;
 * - every metric shows Old, New and a signed Delta, and an absence or an unmeasurable difference
 *   keeps its word instead of becoming `0` or `+0` (prompt §23, §38);
 * - Added, Removed and Changed stay diff vocabulary and never borrow a Gate state (prompt §40);
 * - the capped ranking is an aid whose entries filter the full table, which the shell pages,
 *   filters and orders (prompt §25, §41);
 * - and a failed comparison leaves the last good one on screen, labelled as the pair it belongs to
 *   (prompt §42).
 *
 * The bridge is mocked, the way the P1 files do it: the plumbing has its own file. The change-kind
 * filter is honoured by the stub, because a stub that ignores it would let the additions list pass
 * with rows that are not additions at all.
 */

import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { App } from './App';
import {
  analyzeSelection,
  attachMap,
  clearMap,
  compareSnapshots,
  exportCompareHtml,
  exportCompareJson,
  listCompareCandidates,
  queryEvidence,
  querySectionChanges,
  querySections,
  querySymbolChanges,
  querySymbols,
  selectArtifact,
} from './ipc/bridge';
import type { IpcOutcome } from './ipc/bridge';
import type {
  AnalysisSummaryDto,
  ByteDeltaDto,
  CandidatePageDto,
  CompareCandidateDto,
  CompareSummaryDto,
  ErrorEnvelopeDto,
  SectionChangePageDto,
  SectionChangeQueryDto,
  SectionChangeRowDto,
  SelectionDto,
  SymbolChangePageDto,
  SymbolChangeQueryDto,
  SymbolChangeRowDto,
} from './ipc/types';
import { precedes } from './test/order';

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
  setWindowTitle: vi.fn(() => Promise.resolve({ ok: true, value: null })),
  getAppIdentity: vi.fn(() => Promise.resolve({ ok: true, value: {} })),
  listHistoryBuilds: vi.fn(),
  listHistoryGateRuns: vi.fn(),
  listHistoryReleases: vi.fn(),
}));

const selectMock = vi.mocked(selectArtifact);
const analyzeMock = vi.mocked(analyzeSelection);
const candidatesMock = vi.mocked(listCompareCandidates);
const compareMock = vi.mocked(compareSnapshots);
const sectionChangesMock = vi.mocked(querySectionChanges);
const symbolChangesMock = vi.mocked(querySymbolChanges);
const exportJsonMock = vi.mocked(exportCompareJson);
const exportHtmlMock = vi.mocked(exportCompareHtml);

function ok<T>(value: T): IpcOutcome<T> {
  return { ok: true, value };
}

function fail<T>(envelope: ErrorEnvelopeDto): IpcOutcome<T> {
  return { ok: false, envelope };
}

/** Where the artifact actually sits. It must never reach the screen, or cross back as an argument. */
const MACHINE_PATH = 'D:\\work\\secret-build\\output';

const PAIR_REFUSED: ErrorEnvelopeDto = {
  code: 'ERR-DIFF-5001',
  message: 'Both sides name the same build, so there is nothing to compare.',
  operationId: 'op-diff-1',
  details: null,
  remediation: 'Choose a different base or target.',
};

const WRITE_FAILED: ErrorEnvelopeDto = {
  code: 'ERR-EXPORT-6001',
  message: 'The export could not be written.',
  operationId: 'op-export-1',
  details: 'firmwaresight-diff.json: access denied',
  remediation: 'Check that the folder exists and is writable, then export again.',
};

const ADDITION_REASON =
  'present only in the target build: an addition is not a change from zero';
const REMOVAL_REASON = 'present only in the base build: a removal is not a change to zero';

/** A stored build, spelled out field by field like the generated contract requires. */
function candidate(overrides: Partial<CompareCandidateDto> = {}): CompareCandidateDto {
  const base: CompareCandidateDto = {
    buildId: 'build-old',
    snapshotId: 'snap-old',
    fileName: 'app-old.elf',
    sha256: 'a'.repeat(64),
    byteSize: 5432,
    architecture: 'Arm',
    importedAt: '2026-09-28T09:00:00Z',
    nonvolatile: { state: 'exact', bytes: 4200 },
    runtimeRam: { state: 'exact', bytes: 1024 },
  };
  return { ...base, ...overrides };
}

const NEWER: CompareCandidateDto = candidate({
  buildId: 'build-new',
  snapshotId: 'snap-new',
  fileName: 'app-new.elf',
  sha256: 'b'.repeat(64),
  byteSize: 5560,
  importedAt: '2026-09-29T08:30:00Z',
  nonvolatile: { state: 'exact', bytes: 4456 },
  runtimeRam: { state: 'exact', bytes: 956 },
});

const THIRD: CompareCandidateDto = candidate({
  buildId: 'build-third',
  snapshotId: 'snap-third',
  fileName: 'app-third.elf',
  sha256: 'c'.repeat(64),
  byteSize: 5600,
});

function candidatePage(
  rows: readonly CompareCandidateDto[],
  overrides: Partial<CandidatePageDto> = {},
): CandidatePageDto {
  const base: CandidatePageDto = {
    rows: [...rows],
    total: rows.length,
    offset: 0,
    limit: 25,
    nextOffset: null,
  };
  return { ...base, ...overrides };
}

function change(
  base: number | null,
  target: number | null,
  delta: number | null,
  comparability = 'exact',
  reason: string | null = null,
): ByteDeltaDto {
  return { base, target, delta, comparability, reason };
}

function sectionRow(overrides: Partial<SectionChangeRowDto> = {}): SectionChangeRowDto {
  const base: SectionChangeRowDto = {
    changeKind: 'Changed',
    key: '.text',
    nameKnown: true,
    ambiguous: false,
    fileSize: change(60, 92, 32),
    memorySize: change(60, 92, 32),
    base: {
      index: 1,
      role: 'Code',
      region: 'FLASH',
      virtualAddress: '0x08000000',
      loadAddress: '0x08000000',
      fileOffset: '0x00001000',
      fileSize: 60,
      memorySize: 60,
    },
    target: {
      index: 1,
      role: 'Code',
      region: 'FLASH',
      virtualAddress: '0x08000000',
      loadAddress: '0x08000000',
      fileOffset: '0x00001000',
      fileSize: 92,
      memorySize: 92,
    },
    differingFields: ['file_size', 'memory_size'],
    indeterminateFields: [],
  };
  return { ...base, ...overrides };
}

/** The three shapes a change table has to carry: a pair, an addition, a removal. */
function sectionRows(): SectionChangeRowDto[] {
  return [
    sectionRow(),
    sectionRow({
      changeKind: 'Added',
      key: '.noinit',
      fileSize: change(null, 256, null, 'unknown', ADDITION_REASON),
      memorySize: change(null, 256, null, 'unknown', ADDITION_REASON),
      base: null,
      target: {
        index: 9,
        role: 'Data',
        region: 'RAM',
        virtualAddress: '0x20000100',
        loadAddress: '0x08000200',
        fileOffset: '0x00002000',
        fileSize: 256,
        memorySize: 256,
      },
      differingFields: [],
      indeterminateFields: [],
    }),
    sectionRow({
      changeKind: 'Removed',
      key: '.oldboot',
      fileSize: change(128, null, null, 'unknown', REMOVAL_REASON),
      memorySize: change(128, null, null, 'unknown', REMOVAL_REASON),
      base: {
        index: 4,
        role: 'Code',
        region: 'FLASH',
        virtualAddress: '0x08000400',
        loadAddress: '0x08000400',
        fileOffset: '0x00001400',
        fileSize: 128,
        memorySize: 128,
      },
      target: null,
      differingFields: [],
      indeterminateFields: [],
    }),
  ];
}

function sectionPage(rows: readonly SectionChangeRowDto[]): SectionChangePageDto {
  return {
    rows: [...rows],
    total: rows.length,
    offset: 0,
    limit: 100,
    nextOffset: null,
  };
}

function symbolRow(overrides: Partial<SymbolChangeRowDto> = {}): SymbolChangeRowDto {
  const base: SymbolChangeRowDto = {
    changeKind: 'Changed',
    name: 'decode_packet',
    kind: 'Func',
    binding: 'Global',
    ambiguous: false,
    size: change(512, 640, 128),
    base: { ordinal: 24, address: '0x08001200', size: 512, sectionRef: 'Index(1)' },
    target: { ordinal: 24, address: '0x08001240', size: 640, sectionRef: 'Index(1)' },
    differingFields: ['address', 'size'],
    indeterminateFields: [],
  };
  return { ...base, ...overrides };
}

function symbolRows(): SymbolChangeRowDto[] {
  return [
    symbolRow(),
    symbolRow({
      changeKind: 'Added',
      name: 'ota_resume',
      size: change(null, 64, null, 'unknown', ADDITION_REASON),
      base: null,
      target: { ordinal: 40, address: '0x08009000', size: 64, sectionRef: 'Index(1)' },
      differingFields: [],
      indeterminateFields: [],
    }),
    symbolRow({
      // The shape US-002 cares most about: a size that was never recorded on one side.
      changeKind: 'Changed',
      name: 'g_threshold',
      kind: 'Object',
      size: change(4, null, null, 'unknown', 'at least one side records no size for this symbol'),
      target: { ordinal: 12, address: '0x20000000', size: null, sectionRef: 'Index(3)' },
      differingFields: ['address'],
      indeterminateFields: ['size'],
    }),
  ];
}

function symbolPage(rows: readonly SymbolChangeRowDto[]): SymbolChangePageDto {
  return {
    rows: [...rows],
    total: rows.length,
    offset: 0,
    limit: 100,
    nextOffset: rows.length === 0 ? null : 3,
  };
}

function comparison(overrides: Partial<CompareSummaryDto> = {}): CompareSummaryDto {
  const base: CompareSummaryDto = {
    diffId: 'cmp-7f3a-1',
    base: { snapshotId: 'snap-old', fileName: 'app-old.elf', sha256: 'a'.repeat(64) },
    target: { snapshotId: 'snap-new', fileName: 'app-new.elf', sha256: 'b'.repeat(64) },
    memory: {
      nonvolatile: change(4200, 4456, 256),
      runtimeRam: change(1024, 956, -68),
      comparability: 'exact',
      base: {
        footprintRowPresent: true,
        nonvolatile: { state: 'exact', bytes: 4200 },
        runtimeRam: { state: 'exact', bytes: 1024 },
        mapBacked: true,
        layoutSource: 'map',
        weakestEvidenceBasis: 'map-memory-configuration+elf-load',
      },
      target: {
        footprintRowPresent: true,
        nonvolatile: { state: 'exact', bytes: 4456 },
        runtimeRam: { state: 'exact', bytes: 956 },
        mapBacked: true,
        layoutSource: 'map',
        weakestEvidenceBasis: 'map-memory-configuration+elf-load',
      },
      evidenceWarning: null,
    },
    counts: {
      sections: { added: 1, removed: 2, changed: 3, ambiguous: 0 },
      symbols: { added: 4, removed: 5, changed: 6, ambiguous: 2 },
      unchangedSections: 12,
      unchangedSymbols: 340,
    },
    // The honest state of this build: no stored fact attributes a row to an object file.
    objectChanges: {
      available: false,
      reason: 'no stored fact attributes a section or symbol to an object file',
    },
    topSections: [{ key: '.text', changeKind: 'Changed', delta: 32, bytes: 92 }],
    topSymbols: [{ key: 'decode_packet', changeKind: 'Changed', delta: 128, bytes: 640 }],
    warnings: [],
  };
  return { ...base, ...overrides };
}

/** A pair whose target was laid out from names and flags, which is the degradation the screen shows. */
function degradedComparison(): CompareSummaryDto {
  const summary = comparison();
  return {
    ...summary,
    memory: {
      ...summary.memory,
      nonvolatile: change(4200, 4456, null, 'partial', 'the target total is a floor'),
      runtimeRam: change(1024, 956, -68, 'partial'),
      comparability: 'partial',
      target: {
        ...summary.memory.target,
        mapBacked: false,
        layoutSource: 'elf-address-and-flags',
        weakestEvidenceBasis: 'elf-address-and-flags',
      },
      evidenceWarning: 'target layout came from section names and flags, not from the linker',
    },
    warnings: [
      {
        code: 'SECTION-AMBIGUOUS',
        message: '2 section rows share a name and were left unpaired',
      },
    ],
  };
}

/** One pair whose two sides report the given evidence bases, for the wording guards below. */
function withBases(baseBasis: string | null, targetBasis: string | null): CompareSummaryDto {
  const summary = comparison();
  return {
    ...summary,
    memory: {
      ...summary.memory,
      base: { ...summary.memory.base, weakestEvidenceBasis: baseBasis },
      target: { ...summary.memory.target, weakestEvidenceBasis: targetBasis },
    },
  };
}

function selectionSummary(): AnalysisSummaryDto {
  return {
    source: 'artifact',
    artifact: {
      fileName: 'app-new.elf',
      kind: 'elf',
      sha256: 'b'.repeat(64),
      byteSize: 5560,
      parserId: 'object-elf/0.40',
      architecture: 'Arm',
      bitness: '32',
      endianness: 'little',
      entryPoint: '0x080003f8',
      entryPointUnknownReason: null,
      buildId: null,
      buildIdUnknownReason: 'no NT_GNU_BUILD_ID note was found',
    },
    identity: {
      schema: 'firmwaresight.analyze/p0-internal-1',
      schemaStability: 'p0-internal',
      // The snapshot Analyze proved, which the Compare selectors should prefer.
      snapshotId: 'snap-new',
      normalizationVersion: 'p0-normalize-1',
      createdByFwsightVersion: '0.1.0',
    },
    memory: {
      accountingRule: 'adr-0021-dual-budget',
      layoutSource: 'map',
      weakestEvidenceBasis: 'map-memory-configuration+elf-load',
      admissibleForHardBlock: true,
      nonvolatileImageFootprint: {
        state: 'exact',
        classification: 'observed',
        bytes: 4456,
        unattributed: [],
        reason: null,
      },
      runtimeRamFootprint: {
        state: 'exact',
        classification: 'observed',
        bytes: 956,
        unattributed: [],
        reason: null,
      },
      dualAccountedSections: [],
      excludedMetadataBytes: 0,
    },
    sectionCount: 14,
    symbolCount: 30,
    capabilities: {
      elf: 'supported',
      sections: 'available',
      symbols: 'available',
      debugInfo: 'available',
      map: 'provided',
      objectAttribution: 'unavailable',
      git: 'unknown',
    },
    evidenceSummary: { total: 9, observed: 9, derived: 0, declared: 0, unknown: 0 },
  } as AnalysisSummaryDto;
}

/** The stub answers a change-kind filter the way the shell does. */
function sectionChangesAnswer(
  request: SectionChangeQueryDto,
): IpcOutcome<SectionChangePageDto> {
  const all = sectionRows();
  const rows =
    request.changeKind === null
      ? all
      : all.filter((row) => row.changeKind.toLowerCase() === request.changeKind);
  return ok(sectionPage(rows));
}

function symbolChangesAnswer(request: SymbolChangeQueryDto): IpcOutcome<SymbolChangePageDto> {
  const all = symbolRows();
  const rows =
    request.changeKind === null
      ? all
      : all.filter((row) => row.changeKind.toLowerCase() === request.changeKind);
  return ok(symbolPage(rows));
}

beforeEach(() => {
  selectMock.mockReset();
  analyzeMock.mockReset();
  candidatesMock.mockReset();
  compareMock.mockReset();
  sectionChangesMock.mockReset();
  symbolChangesMock.mockReset();
  exportJsonMock.mockReset();
  exportHtmlMock.mockReset();

  const emptyPage = { rows: [], total: 0, offset: 0, limit: 100, nextOffset: null };
  vi.mocked(querySections).mockResolvedValue(ok(emptyPage));
  vi.mocked(querySymbols).mockResolvedValue(ok(emptyPage));
  vi.mocked(queryEvidence).mockResolvedValue(ok(emptyPage));

  const selection: SelectionDto = {
    selectionId: 'sel-1',
    fileName: 'app-new.elf',
    mapFileName: null,
    mapAttached: false,
  };
  selectMock.mockResolvedValue(ok(selection));
  vi.mocked(attachMap).mockResolvedValue(ok(null));
  vi.mocked(clearMap).mockResolvedValue(ok(selection));
  analyzeMock.mockResolvedValue(ok(selectionSummary()));

  candidatesMock.mockResolvedValue(ok(candidatePage([NEWER, candidate()])));
  compareMock.mockResolvedValue(ok(comparison()));
  sectionChangesMock.mockImplementation(async (request) => sectionChangesAnswer(request));
  symbolChangesMock.mockImplementation(async (request) => symbolChangesAnswer(request));
  exportJsonMock.mockResolvedValue(
    ok({ status: 'written', fileName: 'firmwaresight-diff.json', format: 'json' }),
  );
  exportHtmlMock.mockResolvedValue(
    ok({ status: 'written', fileName: 'firmwaresight-diff.html', format: 'html' }),
  );
});

/** Open Compare with stored builds and wait until the pair is named and the action is live. */
async function openCompare(summary: CompareSummaryDto = comparison()) {
  compareMock.mockResolvedValue(ok(summary));
  render(<App />);
  fireEvent.click(await screen.findByRole('button', { name: 'Compare page' }));
  await screen.findByRole('combobox', { name: 'New / Target' });
  await waitFor(() =>
    expect((screen.getByRole('button', { name: 'Compare' }) as HTMLButtonElement).disabled).toBe(
      false,
    ),
  );
}

async function runCompare() {
  fireEvent.click(screen.getByRole('button', { name: 'Compare' }));
  await screen.findByRole('region', { name: 'Build comparison' });
}

function pair(): [HTMLSelectElement, HTMLSelectElement] {
  return [
    screen.getByRole('combobox', { name: 'Old / Base' }) as HTMLSelectElement,
    screen.getByRole('combobox', { name: 'New / Target' }) as HTMLSelectElement,
  ];
}

function rowOf(term: string): string {
  const node = screen.getByText(term).closest('div');
  return node?.textContent ?? '';
}

/** A read the test finishes by hand, so an arrival order can be forced rather than wished for. */
function deferred<T>() {
  let resolve!: (outcome: IpcOutcome<T>) => void;
  const promise = new Promise<IpcOutcome<T>>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}

describe('the rail and the pages it lists', () => {
  it('lists the stages this build has, and marks the page the reader is on', async () => {
    render(<App />);

    const rail = await screen.findByRole('navigation', { name: 'Pages' });
    // P3 added a third real page, P5 a fourth, and U1 a fifth - Overview - while two entries took the
    // names the pages already gave themselves on screen. This list is still the build's own inventory: an
    // entry appears when the stage exists and nowhere else does its word appear (prompt §43).
    expect(within(rail).getAllByRole('button').map((item) => item.textContent)).toEqual([
      'Overview',
      'Analyze',
      'Compare',
      'Release Gate',
      'Bundle & History',
    ]);
    expect(screen.getByRole('button', { name: 'Analyze page' }).getAttribute('aria-current')).toBe(
      'page',
    );

    fireEvent.click(screen.getByRole('button', { name: 'Compare page' }));
    await screen.findByRole('heading', { level: 1, name: 'Compare' });
    expect(screen.getByRole('button', { name: 'Compare page' }).getAttribute('aria-current')).toBe(
      'page',
    );
    expect(screen.queryByRole('heading', { level: 1, name: 'Analyze' })).toBeNull();
  });

  it('names the two sides by their roles, never as A and B', async () => {
    await openCompare();

    expect(screen.getByText('Old / Base')).toBeDefined();
    expect(screen.getByText('New / Target')).toBeDefined();
    expect(screen.queryByText('A / B')).toBeNull();
  });

  it('shows the recorded facts of each side before any comparison runs', async () => {
    await openCompare();

    const facts = within(screen.getByText('Old / Base').closest('div') as HTMLElement);
    expect(facts.getByText('5,432 bytes')).toBeDefined();
    expect(facts.getByText('Nonvolatile exact 4,200 bytes')).toBeDefined();
    expect(facts.getByText('Runtime RAM exact 1,024 bytes')).toBeDefined();
    expect(facts.getByText('imported 2026-09-28T09:00:00Z')).toBeDefined();
  });
});

describe('defaults and refusals', () => {
  it('compares nothing until the reader asks', async () => {
    await openCompare();

    expect(compareMock).not.toHaveBeenCalled();
    expect(sectionChangesMock).not.toHaveBeenCalled();
    expect(screen.queryByRole('region', { name: 'Build comparison' })).toBeNull();
  });

  it('prefers the build Analyze last proved as the target and the next distinct one as the base', async () => {
    render(<App />);
    fireEvent.click(await screen.findByRole('button', { name: 'Choose firmware artifact' }));
    await screen.findByText('app-new.elf');
    fireEvent.click(screen.getByRole('button', { name: 'Analyze' }));
    await screen.findByRole('region', { name: 'Analysis summary' });

    fireEvent.click(screen.getByRole('button', { name: 'Compare page' }));
    await screen.findByRole('combobox', { name: 'New / Target' });
    await waitFor(() => expect(pair()[1].value).toBe('snap-new'));

    const [base, target] = pair();
    expect(base.value).toBe('snap-old');
    expect(target.selectedOptions[0]?.textContent).toContain('last analyzed');
  });

  it('falls back to the two most recent stored builds when nothing was analyzed', async () => {
    await openCompare();

    const [base, target] = pair();
    expect(target.value).toBe('snap-new');
    expect(base.value).toBe('snap-old');
  });

  it('refuses a pair of one build: the action locks and the screen says why', async () => {
    await openCompare();

    fireEvent.change(pair()[1], { target: { value: 'snap-old' } });

    expect((screen.getByRole('button', { name: 'Compare' }) as HTMLButtonElement).disabled).toBe(
      true,
    );
    const note = await screen.findByRole('note');
    expect(note.textContent).toContain('Both sides name the same build');
    expect(note.textContent).toContain('app-old.elf · aaaaaaaaaaaa');
    expect(note.textContent).toContain('not the same as nothing having changed');
    expect(compareMock).not.toHaveBeenCalled();
  });

  it('swaps the two sides', async () => {
    await openCompare();
    fireEvent.click(screen.getByRole('button', { name: 'Swap' }));

    const [base, target] = pair();
    expect(base.value).toBe('snap-new');
    expect(target.value).toBe('snap-old');
  });

  it('asks for another build when there is nothing to compare yet', async () => {
    candidatesMock.mockResolvedValue(ok(candidatePage([candidate()], { total: 1 })));
    render(<App />);
    fireEvent.click(await screen.findByRole('button', { name: 'Compare page' }));

    const empty = await screen.findByRole('region', { name: 'Nothing to compare yet' });
    expect(empty.textContent).toContain('Analyze another firmware build before comparing.');
    expect(screen.queryByRole('combobox', { name: 'Old / Base' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Compare' })).toBeNull();
    expect(compareMock).not.toHaveBeenCalled();

    fireEvent.click(within(empty).getByRole('button', { name: 'Go to Analyze' }));
    await screen.findByRole('heading', { level: 1, name: 'Analyze' });
  });

  it('reports a build list that would not load instead of an empty history', async () => {
    candidatesMock.mockResolvedValue(fail(PAIR_REFUSED));
    render(<App />);
    fireEvent.click(await screen.findByRole('button', { name: 'Compare page' }));

    const alert = await screen.findByRole('alert', { name: 'Build list error' });
    expect(alert.textContent).toContain('ERR-DIFF-5001');
    expect(screen.queryByRole('region', { name: 'Nothing to compare yet' })).toBeNull();
  });
});

describe('the summary and its signs', () => {
  it('shows old, new and a signed delta for both memory totals', async () => {
    await openCompare();
    await runCompare();

    const memory = await screen.findByRole('region', { name: 'Memory comparison' });
    expect(memory.textContent).toContain('Delta = target − base');
    expect(rowOf('Nonvolatile / load image')).toContain('Old 4,200 bytes');
    expect(rowOf('Nonvolatile / load image')).toContain('New 4,456 bytes');
    expect(rowOf('Nonvolatile / load image')).toContain('Delta +256 bytes');

    expect(rowOf('Runtime RAM')).toContain('Old 1,024 bytes');
    expect(rowOf('Runtime RAM')).toContain('New 956 bytes');
    expect(rowOf('Runtime RAM')).toContain('Delta -68 bytes');
  });

  it('says Unknown with the reason when a delta cannot be computed', async () => {
    await openCompare(degradedComparison());
    await runCompare();

    const nonvolatile = rowOf('Nonvolatile / load image');
    expect(nonvolatile).toContain('Delta Unknown');
    expect(nonvolatile).toContain('the target total is a floor');
    expect((await screen.findByRole('region', { name: 'Memory comparison' })).textContent).toContain(
      'partial',
    );
  });

  it('never writes +0, or a zero where a side is absent or unknown', async () => {
    await openCompare(degradedComparison());
    await runCompare();

    const text = document.body.textContent ?? '';
    expect(text).not.toContain('+0');
    expect(text).not.toMatch(/Delta 0 /);
    expect(text).not.toMatch(/Old 0 bytes/);
  });

  it('carries the capability and evidence warnings the diff recorded', async () => {
    await openCompare(degradedComparison());
    await runCompare();

    const notice = await screen.findByRole('region', { name: 'Capability and evidence' });
    expect(notice.textContent).toContain(
      'target layout came from section names and flags, not from the linker',
    );
    expect(notice.textContent).toContain('SECTION-AMBIGUOUS');
    expect(notice.textContent).toContain('2 section rows share a name and were left unpaired');
    expect(notice.textContent).toContain(
      'no stored fact attributes a section or symbol to an object file',
    );
    expect(within(notice).getByText('unavailable')).toBeDefined();

    const memory = await screen.findByRole('region', { name: 'Memory comparison' });
    expect(memory.textContent).toContain('No linker MAP');
    expect(memory.textContent).toContain('weakest basis ELF address/flags evidence');
  });

  // L20: `weakest_basis` crosses IPC as the Core enum's own identity, formatted with `{:?}` in
  // `domain::diff.rs`. That is a wire name, and the screen used to print it. The mapping below is
  // presentation only — no variant is renamed and no basis is recomputed in the UI — so every value
  // the shell can send today gets a caption, and anything it cannot gets said to be unrecognized
  // rather than being quietly rendered as a known basis or crashing the page.
  it('captions the two MAP and region backed evidence bases instead of printing the enum', async () => {
    await openCompare(withBases('MapRegionAndElfLoad', 'RegionConfigAndElfLoad'));
    await runCompare();

    const memory = await screen.findByRole('region', { name: 'Memory comparison' });
    expect(memory.textContent).toContain('weakest basis MAP regions + ELF load evidence');
    expect(memory.textContent).toContain('weakest basis configured regions + ELF load evidence');
    expect(memory.textContent).not.toContain('MapRegionAndElfLoad');
    expect(memory.textContent).not.toContain('RegionConfigAndElfLoad');
  });

  it('captions the ELF and name-heuristic evidence bases', async () => {
    await openCompare(withBases('ElfAddressAndFlags', 'SectionNameHeuristic'));
    await runCompare();

    const memory = await screen.findByRole('region', { name: 'Memory comparison' });
    expect(memory.textContent).toContain('weakest basis ELF address/flags evidence');
    expect(memory.textContent).toContain('weakest basis section-name heuristic');
    expect(memory.textContent).not.toContain('SectionNameHeuristic');
  });

  it('captions an insufficient basis and names an unknown future variant as unrecognized', async () => {
    await openCompare(withBases('Insufficient', 'SomeBasisNoBuildHasEverSent'));
    await runCompare();

    const memory = await screen.findByRole('region', { name: 'Memory comparison' });
    expect(memory.textContent).toContain('weakest basis insufficient evidence');
    expect(memory.textContent).toContain('weakest basis Unrecognized evidence basis');
    expect(memory.textContent).not.toContain('SomeBasisNoBuildHasEverSent');
  });

  it('keeps a missing basis Unknown rather than captioning it as something', async () => {
    await openCompare(withBases('MapRegionAndElfLoad', null));
    await runCompare();

    const memory = await screen.findByRole('region', { name: 'Memory comparison' });
    expect(memory.textContent).toContain('weakest basis unknown');
  });

  it('labels object attribution as the two different questions the pages answer', async () => {
    // L19: Analyze answers whether one build can attribute rows to an object or module; Compare
    // answers whether any object file changed between two builds. One word on both pages made a
    // reader choose. The labels must stay distinct, and the states under them must not move.
    await openCompare();
    await runCompare();

    const notice = await screen.findByRole('region', { name: 'Capability and evidence' });
    expect(within(notice).getByText('Object-level change attribution')).toBeDefined();
    expect(notice.textContent).not.toContain('Object attribution');
    expect(within(notice).getByText('unavailable')).toBeDefined();
  });

  it('re-labels the same figures in KiB and asks the shell for nothing', async () => {
    await openCompare();
    await runCompare();
    // Each change table owns its own IPC, so the page is still fetching when `Build comparison` first
    // appears. Sampling here caught zero calls on a Windows runner and two on a Linux one, and the
    // delta below then charged the page's own fetches to the KiB radio. Waiting for a row that only
    // exists once the query has resolved puts the sample after the fetch; the guard proves it did.
    const sections = await screen.findByRole('region', { name: 'Section Changes' });
    await within(sections).findByText('.text');
    const symbols = await screen.findByRole('region', { name: 'Symbol Changes' });
    await within(symbols).findByText('g_threshold');

    const calls = compareMock.mock.calls.length;
    const changeCalls = sectionChangesMock.mock.calls.length;
    expect(changeCalls).toBeGreaterThan(0);

    fireEvent.click(screen.getByRole('radio', { name: 'KiB' }));

    expect(rowOf('Nonvolatile / load image')).toContain('Old 4.10 KiB');
    expect(rowOf('Nonvolatile / load image')).toContain('New 4.35 KiB');
    expect(rowOf('Nonvolatile / load image')).toContain('Delta +0.250 KiB');
    expect(compareMock.mock.calls.length).toBe(calls);
    expect(sectionChangesMock.mock.calls.length).toBe(changeCalls);
  });
});

describe('the change tables', () => {
  it('counts what moved in diff words and never in Gate states', async () => {
    await openCompare();
    await runCompare();

    const moved = await screen.findByRole('region', { name: 'What moved' });
    expect(within(moved).getByText('Added 1')).toBeDefined();
    expect(within(moved).getByText('Removed 2')).toBeDefined();
    expect(within(moved).getByText('Changed 3')).toBeDefined();
    expect(moved.textContent).toContain('340 unchanged');
    expect(moved.textContent).toContain('2 unpaired');
    for (const gate of ['PASS', 'REVIEW', 'BLOCK']) {
      expect(moved.textContent).not.toContain(gate);
    }
  });

  it('shows an added row as absent on the base side, never as zero', async () => {
    await openCompare();
    await runCompare();

    const table = await screen.findByRole('region', { name: 'Section Changes' });
    const row = (await within(table).findByText('.noinit')).closest('tr') as HTMLTableRowElement;
    expect(within(row).getByText('Added')).toBeDefined();
    expect(within(row).getAllByText('Not present').length).toBe(2);
    expect(row.textContent).not.toContain('Old 0 bytes');
  });

  it('shows a removed row as absent on the target side', async () => {
    await openCompare();
    await runCompare();

    const table = await screen.findByRole('region', { name: 'Section Changes' });
    const row = (await within(table).findByText('.oldboot')).closest('tr') as HTMLTableRowElement;
    expect(within(row).getByText('Removed')).toBeDefined();
    expect(within(row).getAllByText('Not present').length).toBeGreaterThan(0);
    expect(row.textContent).toContain('128 bytes');
  });

  it('keeps a size that was never recorded as Unknown with its reason', async () => {
    await openCompare();
    await runCompare();

    const table = await screen.findByRole('region', { name: 'Symbol Changes' });
    const row = (await within(table).findByText('g_threshold')).closest('tr') as HTMLTableRowElement;
    expect(row.textContent).toContain('Unknown');
    expect(row.textContent).toContain('at least one side records no size for this symbol');
    expect(row.textContent).toContain('not decidable: size');
  });

  it('sends the change-kind filter to the shell instead of hiding rows it already has', async () => {
    await openCompare();
    await runCompare();

    const table = await screen.findByRole('region', { name: 'Section Changes' });
    // Each table owns its own kind filter: the symbol table's radios must not move the sections.
    for (const [label, wanted] of [
      ['Added', 'added'],
      ['Removed', 'removed'],
      ['Changed', 'changed'],
      ['All', null],
    ] as const) {
      fireEvent.click(await within(table).findByRole('radio', { name: label }));
      await waitFor(() =>
        expect(
          sectionChangesMock.mock.calls
            .map(([request]) => request)
            .at(-1)
            ?.changeKind === wanted,
        ).toBe(true),
      );
    }
    // What the reader sees is what the shell answered, so a filtered view is a smaller answer and
    // not a hidden one: under Added the removed row is not on the page, and All brings it back.
    fireEvent.click(within(table).getByRole('radio', { name: 'Added' }));
    await waitFor(() => expect(within(table).queryByText('.oldboot')).toBeNull());
    expect(within(table).getByText('.noinit')).toBeDefined();

    fireEvent.click(within(table).getByRole('radio', { name: 'All' }));
    await waitFor(() => expect(within(table).getByText('.oldboot')).toBeDefined());
    expect(within(table).getByText('.text')).toBeDefined();
  });

  it('sends a name filter to the shell, from the form, with the offset reset', async () => {
    await openCompare();
    await runCompare();

    const table = await screen.findByRole('region', { name: 'Section Changes' });
    const input = await within(table).findByRole('textbox', { name: 'Filter sections by name' });
    fireEvent.change(input, { target: { value: 'boot' } });
    fireEvent.click(await within(table).findByRole('button', { name: 'Apply filter' }));

    await waitFor(() => {
      const requests = sectionChangesMock.mock.calls.map(([request]) => request);
      expect(requests.some((request) => request.filter === 'boot' && request.offset === 0)).toBe(
        true,
      );
    });
  });

  it('applies every filter the two change tables are offered, and each one reaches the shell', async () => {
    // L25 revalidation. One dead `Apply filter` click was reported against an intermediate P1 build
    // and never reproduced on a shipped binary, so this drives the control at the count the round
    // asks for rather than once: 15 name filters on sections, 15 on symbols, and 6 more submitted
    // from the keyboard. Each one types a different value, because an unchanged draft emits no
    // request at all — that is correct behaviour, and counting it as an Apply would inflate the
    // very number this test exists to establish.
    const sectionQueries = [
      '.text', '.data', '.bss', 'boot', 'decode', 'uart', 'cfg', '.ota', 'noinit', 'task',
      'ring', 'flash', 'irq', 'stack', '.rodata',
    ];
    const symbolQueries = [
      'mqtt_task', 'tls_handshake', 'sensor_fifo', 'g_threshold', 'g_scratch', 'main',
      'decode_packet', 'uart_putc', 'irq_handler', 'ring_push', 'cfg_table', 'ota_verify',
      'flash_write', 'task_init', 'boot_header',
    ];
    // Values the click loops above never typed, so a request carrying one can only have come from
    // the keyboard path being asserted here.
    const submitQueries = ['eeprom', 'watchdog', 'spi', 'i2c', 'crc', 'timer'];

    await openCompare();
    await runCompare();

    const sections = await screen.findByRole('region', { name: 'Section Changes' });
    const symbols = await screen.findByRole('region', { name: 'Symbol Changes' });
    const sectionInput = within(sections).getByRole('textbox', { name: 'Filter sections by name' });
    const symbolInput = within(symbols).getByRole('textbox', { name: 'Filter symbols by name' });
    const sectionApply = within(sections).getByRole('button', { name: 'Apply filter' });
    const sectionForm = sectionInput.closest('form');
    expect(sectionForm).not.toBeNull();
    const symbolApply = within(symbols).getByRole('button', { name: 'Apply filter' });

    const requested = <T extends { filter: string | null; offset: number }>(
      mock: { mock: { calls: readonly [T][] } },
      value: string,
    ) => mock.mock.calls.some(([request]) => request.filter === value && request.offset === 0);

    let applied = 0;
    for (const value of sectionQueries) {
      fireEvent.change(sectionInput, { target: { value } });
      expect((sectionApply as HTMLButtonElement).disabled).toBe(false);
      fireEvent.click(sectionApply);
      await waitFor(() => expect(requested(sectionChangesMock, value)).toBe(true));
      applied += 1;
    }
    for (const value of symbolQueries) {
      fireEvent.change(symbolInput, { target: { value } });
      expect((symbolApply as HTMLButtonElement).disabled).toBe(false);
      fireEvent.click(symbolApply);
      await waitFor(() => expect(requested(symbolChangesMock, value)).toBe(true));
      applied += 1;
    }

    // The keyboard path, as far as this harness can honestly take it. A synthetic Enter keydown was
    // tried first and produced no request at all — jsdom does not implement implicit form
    // submission — so a real keypress in an installed window stays `NOT_VERIFIED_THROUGH_A_KEYPRESS`
    // and is recorded as such rather than being claimed from here. What this can prove is the two
    // conditions a browser needs for Enter to work, plus the code path Enter then takes: the control
    // is a submit button inside a form, and that form's submit handler asks the shell for the filter.
    expect((sectionApply as HTMLButtonElement).type).toBe('submit');
    let bySubmit = 0;
    for (const value of submitQueries) {
      fireEvent.change(sectionInput, { target: { value } });
      fireEvent.submit(sectionForm as HTMLElement);
      await waitFor(() => expect(requested(sectionChangesMock, value)).toBe(true));
      bySubmit += 1;
    }

    expect([applied, bySubmit]).toEqual([30, 6]);
  });

  it('asks for the other direction when the same column is chosen twice', async () => {
    await openCompare();
    await runCompare();

    const table = await screen.findByRole('region', { name: 'Section Changes' });
    const nameHeader = () =>
      within(table)
        .getAllByRole('columnheader')
        .find((cell) => cell.textContent?.startsWith('Name'));

    // The table opens ordered by name ascending, so the first click on that column asks for the
    // other end of the same ordering rather than a different column.
    fireEvent.click(await within(table).findByRole('button', { name: 'Sort by Name' }));
    await waitFor(() => {
      const requests = sectionChangesMock.mock.calls.map(([request]) => request);
      expect(
        requests.some((request) => request.sort === 'key' && request.direction === 'desc'),
      ).toBe(true);
    });
    await within(table).findByRole('button', { name: 'Sort by Name' });
    expect(nameHeader()?.getAttribute('aria-sort')).toBe('descending');

    fireEvent.click(within(table).getByRole('button', { name: 'Sort by Name' }));
    await waitFor(() => {
      const requests = sectionChangesMock.mock.calls.map(([request]) => request);
      expect(
        requests.some((request) => request.sort === 'key' && request.direction === 'asc'),
      ).toBe(true);
    });
    await within(table).findByRole('button', { name: 'Sort by Name' });
    expect(nameHeader()?.getAttribute('aria-sort')).toBe('ascending');
  });

  it('pages with the offset the shell reported and says which rows are on screen', async () => {
    await openCompare();
    await runCompare();

    // The region mounts before its first page lands: the pager only renders once the shell has
    // answered, so reading it synchronously lost that race on a Windows runner. Wait for the range
    // itself - it exists only from a resolved page, and Next page is rendered beside it.
    const table = await screen.findByRole('region', { name: 'Symbol Changes' });
    expect(await within(table).findByText(/Showing 1 to 3 of 3 symbols/)).toBeDefined();

    fireEvent.click(within(table).getByRole('button', { name: 'Next page' }));
    await waitFor(() => {
      const requests = symbolChangesMock.mock.calls.map(([request]) => request);
      expect(requests.some((request) => request.offset === 3)).toBe(true);
    });
  });

  it('orders the table on the quantity the additions list ranks by', async () => {
    await openCompare();
    await runCompare();

    fireEvent.click(await screen.findByRole('button', { name: 'Sort by RAM size' }));
    await waitFor(() => {
      const requests = sectionChangesMock.mock.calls.map(([request]) => request);
      expect(
        requests.some((request) => request.sort === 'memorySize' && request.direction === 'asc'),
      ).toBe(true);
    });
  });
});

describe('the ranking as a way to the full list', () => {
  it('lists growth and additions apart, and keeps the full tables reachable', async () => {
    await openCompare();
    await runCompare();

    const ranking = await screen.findByRole('region', { name: 'Top growth and largest additions' });
    expect(within(ranking).getByText('Top growth')).toBeDefined();
    expect(within(ranking).getByText('Largest additions')).toBeDefined();
    // An addition is never shown as growth from zero, and each list names its own rows.
    // Growth travels with the summary that put the region on screen, so it is already here. The
    // added rows are a second read that commits on its own (Compare.tsx:224), so naming one of them
    // has to wait for that read: the sixth L23 instance, lost by the local full gate under load.
    expect(screen.getByRole('button', { name: 'Show sections changes for .text' })).toBeDefined();
    expect(
      await screen.findByRole('button', { name: 'Show sections changes for .noinit' }),
    ).toBeDefined();
    // The full tables are asked for with no kind filter, so nothing hides behind the ranking.
    const requests = sectionChangesMock.mock.calls.map(([request]) => request);
    expect(requests.some((request) => request.changeKind === null && request.limit === null)).toBe(
      true,
    );
    expect(await screen.findByRole('region', { name: 'Section Changes' })).toBeDefined();
    expect(screen.getByRole('region', { name: 'Symbol Changes' })).toBeDefined();
  });

  it('tells the reader the ranked lists are not a breakdown of the load-image delta', async () => {
    await openCompare();
    await runCompare();

    // A reader who adds up the rows on screen gets a number that will not land on the Memory
    // headline, and the reason is a difference of basis as well as a cut-off. The screen has to say
    // so next to the lists rather than leave the gap to be guessed at.
    const ranking = await screen.findByRole('region', { name: 'Top growth and largest additions' });
    const basis = within(ranking).getByText(/load image[\s\S]*delta/i);
    // Which headline the sentence points at, what each side is measured over, and that a shortfall
    // is neither a bug nor a row the table is hiding.
    expect(basis.textContent).toMatch(/neither/i);
    expect(basis.textContent).toMatch(/whole image/i);
    expect(basis.textContent).toMatch(/runtime size/i);
    expect(basis.textContent).toMatch(/not a missing row/i);
  });

  it('says the added rows are still being read before it names one of them', async () => {
    await openCompare();
    // Hold the added-rows read open. The ranking region is painted from the summary and does not
    // wait for it, so the screen has to say what is missing rather than show a row it never got.
    const pending = deferred<SectionChangePageDto>();
    sectionChangesMock.mockImplementation((request) =>
      request.changeKind === 'added' ? pending.promise : Promise.resolve(sectionChangesAnswer(request)),
    );

    await runCompare();

    const ranking = await screen.findByRole('region', { name: 'Top growth and largest additions' });
    expect(within(ranking).getByText('Reading the added rows…')).toBeDefined();
    expect(screen.queryByRole('button', { name: 'Show sections changes for .noinit' })).toBeNull();
    // The growth list is the summary's own answer, so it is already here.
    expect(screen.getByRole('button', { name: 'Show sections changes for .text' })).toBeDefined();

    pending.resolve(
      ok(sectionPage(sectionRows().filter((row) => row.changeKind === 'Added'))),
    );

    expect(
      await screen.findByRole('button', { name: 'Show sections changes for .noinit' }),
    ).toBeDefined();
    expect(within(ranking).queryByText('Reading the added rows…')).toBeNull();
  });

  it('drills a growth entry into the section table that holds it', async () => {
    await openCompare();
    await runCompare();

    fireEvent.click(await screen.findByRole('button', { name: 'Show sections changes for .text' }));

    await waitFor(() => {
      const requests = sectionChangesMock.mock.calls.map(([request]) => request);
      expect(requests.some((request) => request.filter === '.text' && request.offset === 0)).toBe(
        true,
      );
    });
    expect(
      (screen.getByRole('textbox', { name: 'Filter sections by name' }) as HTMLInputElement).value,
    ).toBe('.text');
    expect(document.activeElement?.getAttribute('aria-label')).toBe('Section Changes');
  });

  it('drills an addition into the symbol table that holds it', async () => {
    await openCompare();
    await runCompare();

    fireEvent.click(
      await screen.findByRole('button', { name: 'Show symbols changes for ota_resume' }),
    );

    await waitFor(() => {
      const requests = symbolChangesMock.mock.calls.map(([request]) => request);
      expect(requests.some((request) => request.filter === 'ota_resume')).toBe(true);
    });
    expect(document.activeElement?.getAttribute('aria-label')).toBe('Symbol Changes');
  });
});

describe('export', () => {
  it('names the file it wrote, and nothing about where it lives', async () => {
    await openCompare();
    await runCompare();

    fireEvent.click(await screen.findByRole('button', { name: 'Export JSON' }));
    expect(await screen.findByText('Wrote firmwaresight-diff.json (JSON).')).toBeDefined();
    fireEvent.click(screen.getByRole('button', { name: 'Export HTML' }));
    expect(await screen.findByText('Wrote firmwaresight-diff.html (HTML).')).toBeDefined();

    const sent = JSON.stringify(exportJsonMock.mock.calls);
    expect(sent).not.toContain(MACHINE_PATH);
    expect(sent.toLowerCase()).not.toContain('path');
    expect((document.body.textContent ?? '')).not.toContain(MACHINE_PATH);
  });

  it('treats a cancelled save dialog as a normal outcome, not an error', async () => {
    await openCompare();
    await runCompare();
    exportJsonMock.mockResolvedValue(ok({ status: 'cancelled', fileName: null, format: 'json' }));

    fireEvent.click(await screen.findByRole('button', { name: 'Export JSON' }));

    expect(await screen.findByText('Export cancelled. No file was written.')).toBeDefined();
    expect(screen.queryByRole('alert', { name: 'Export error' })).toBeNull();
  });

  it('treats a refused overwrite as the reader deciding', async () => {
    await openCompare();
    await runCompare();
    exportHtmlMock.mockResolvedValue(
      ok({ status: 'kept-existing', fileName: null, format: 'html' }),
    );

    fireEvent.click(await screen.findByRole('button', { name: 'Export HTML' }));

    expect(
      await screen.findByText('Kept the file that was already there. Nothing was written.'),
    ).toBeDefined();
    expect(screen.queryByRole('alert', { name: 'Export error' })).toBeNull();
  });

  it('keeps the comparison on screen when an export fails', async () => {
    await openCompare();
    await runCompare();
    exportJsonMock.mockResolvedValue(fail(WRITE_FAILED));

    fireEvent.click(await screen.findByRole('button', { name: 'Export JSON' }));

    const alert = await screen.findByRole('alert', { name: 'Export error' });
    expect(alert.textContent).toContain('ERR-EXPORT-6001');
    expect(alert.textContent).toContain('op-export-1');
    // A failed export does not invalidate the comparison, so the report is still the one it made.
    expect(screen.getByRole('region', { name: 'Build comparison' })).toBeDefined();
    expect(within(screen.getByRole('region', { name: 'Memory comparison' })).getByText('+256 bytes')).toBeDefined();
  });

  it('offers no export before there is a comparison', async () => {
    await openCompare();

    expect(screen.queryByRole('region', { name: 'Export' })).toBeNull();
    expect(exportJsonMock).not.toHaveBeenCalled();
  });
});

describe('a failed comparison', () => {
  it('keeps the previous valid diff and says which pair it belongs to', async () => {
    await openCompare();
    await runCompare();
    compareMock.mockResolvedValue(fail(PAIR_REFUSED));

    fireEvent.click(screen.getByRole('button', { name: 'Compare' }));

    const alert = await screen.findByRole('alert', { name: 'Comparison error' });
    expect(alert.textContent).toContain('ERR-DIFF-5001');

    const lastGood = await screen.findByRole('region', { name: 'Last good comparison' });
    expect(lastGood.textContent).toContain(
      'Previous comparison of app-old.elf · aaaaaaaaaaaa and app-new.elf · bbbbbbbbbbbb.',
    );
    expect(lastGood.textContent).toContain(
      'The attempt above produced no result, so nothing here was replaced.',
    );
    expect(within(lastGood).getByText('+256 bytes')).toBeDefined();
    expect(screen.queryByRole('region', { name: 'Build comparison' })).toBeNull();
  });

  it('says which pair a standing report describes once the selection has moved', async () => {
    candidatesMock.mockResolvedValue(ok(candidatePage([NEWER, candidate(), THIRD])));
    await openCompare();
    await runCompare();

    // The reader repoints the base at a third build and does not press Compare again. Nothing has
    // failed here, so the only signal that the screen is showing an earlier answer is this note.
    fireEvent.change(pair()[0], { target: { value: 'snap-third' } });

    const lastGood = await screen.findByRole('region', { name: 'Last good comparison' });
    expect(lastGood.textContent).toContain(
      'This is the comparison of app-old.elf · aaaaaaaaaaaa and app-new.elf · bbbbbbbbbbbb that you asked for.',
    );
    expect(lastGood.textContent).toContain('The pair selected above is a different one');
    expect(screen.queryByRole('alert')).toBeNull();
    // The tables still answer for the comparison that succeeded, not for the pair now selected.
    const requests = sectionChangesMock.mock.calls.map(([request]) => request);
    expect(requests.every((request) => request.diffId === 'cmp-7f3a-1')).toBe(true);
  });
});

describe('accessibility of the comparison', () => {
  it('labels every control a reader has to operate', async () => {
    await openCompare();
    await runCompare();

    expect(screen.getByLabelText('Old / Base')).toBeDefined();
    expect(screen.getByLabelText('New / Target')).toBeDefined();
    expect(screen.getByRole('group', { name: 'Size units' })).toBeDefined();
    expect(screen.getByRole('group', { name: 'Filter sections by change kind' })).toBeDefined();
    expect(screen.getByRole('group', { name: 'Filter symbols by change kind' })).toBeDefined();
    expect(screen.getByRole('textbox', { name: 'Filter sections by name' })).toBeDefined();
    expect(screen.getByRole('textbox', { name: 'Filter symbols by name' })).toBeDefined();
    // The name box is a query about firmware, not a login field.
    expect(
      (screen.getByRole('textbox', { name: 'Filter sections by name' }) as HTMLInputElement)
        .autocomplete,
    ).toBe('off');
  });

  it('gives the tables header semantics, not colour-only state', async () => {
    await openCompare();
    await runCompare();

    // The region mounts before its first page lands: Compare.tsx clears the page on every request
    // and the table renders only from a resolved one, so reading its headers synchronously lost
    // that race on a Windows runner (Run 37128593254) the same way 055b54e closed the pager.
    const table = await screen.findByRole('region', { name: 'Section Changes' });
    const headers = await within(table).findAllByRole('columnheader');
    expect(headers.length).toBeGreaterThan(8);
    const sortable = headers.filter((cell) => cell.getAttribute('aria-sort') !== null);
    expect(sortable.length).toBeGreaterThan(0);
    // The kind is a word in the cell, so a reader who cannot see the diff colours still knows.
    for (const kind of ['Added', 'Removed', 'Changed']) {
      expect(within(table).getAllByText(kind).length).toBeGreaterThan(0);
    }
  });

  it('keeps a contributor reachable by keyboard, where Tab lands in reading order', async () => {
    await openCompare();
    await runCompare();

    const pick = await screen.findByRole('button', { name: 'Show sections changes for .text' });
    // A contributor is a real button, so the keyboard path reaches it in reading order and the
    // global focus ring marks where the reader has come to.
    expect(pick.tagName).toBe('BUTTON');
    pick.focus();
    expect(document.activeElement).toBe(pick);
    fireEvent.click(pick);
    await waitFor(() =>
      expect(
        sectionChangesMock.mock.calls.some(([request]) => request.filter === '.text'),
      ).toBe(true),
    );
  });
});

describe('the table layout contract', () => {
  // F2R-01 closed this defect class on the Analyze sections table; U1 §B.9 found the same shape still
  // live here, where both change tables carried `display: block` and a name column allowed to wrap.
  // jsdom cannot measure a clip, so the tree is pinned and the geometry proof stays with the
  // installed run, exactly as `details.test.tsx` puts it.
  it('puts every change table inside a scroll wrapper and keeps it a table', async () => {
    await openCompare();
    await runCompare();

    const tables = await screen.findAllByRole('table');
    // The contract is universal rather than counted: a third table added later is inside a wrapper or
    // this test fails, which is the case a fixed count would silently wave through.
    expect(tables.length).toBeGreaterThan(0);
    for (const table of tables) {
      expect(table.tagName).toBe('TABLE');
      expect(table.parentElement?.className).toContain('viewport');
    }
  });
});

describe('U1P Compare hierarchy', () => {
  it('names the pair first, then what moved, then what it cost, then the prose', async () => {
    await openCompare();
    await runCompare();

    const selectors = screen.getByRole('region', { name: 'Build selection' });
    const moved = await screen.findByRole('region', { name: 'What moved' });
    const memory = screen.getByRole('region', { name: 'Memory comparison' });
    const sections = screen.getByRole('region', { name: 'Section Changes' });
    const symbols = screen.getByRole('region', { name: 'Symbol Changes' });
    const evidence = screen.getByRole('region', { name: 'Capability and evidence' });

    // §10: the result leads the page. The reader meets the change counts before the flash and RAM
    // deltas, those before the tables, and the evidence narrative last, because it explains a
    // comparison instead of being one.
    expect(precedes(selectors, moved)).toBe(true);
    expect(precedes(moved, memory)).toBe(true);
    expect(precedes(memory, sections)).toBe(true);
    expect(precedes(sections, symbols)).toBe(true);
    expect(precedes(symbols, evidence)).toBe(true);
  });

  it('states the pair as two labelled picks with an arrow that says nothing twice', async () => {
    await openCompare();

    const selectors = screen.getByRole('region', { name: 'Build selection' });
    // Each side is answerable by name, which is what keeps the pair readable in the accessibility tree.
    expect(within(selectors).getByRole('combobox', { name: 'Old / Base' })).toBeDefined();
    expect(within(selectors).getByRole('combobox', { name: 'New / Target' })).toBeDefined();

    // The arrow is decoration. Left unhidden it would be read out between two selects that already name
    // their own direction, and §22 counts a spoken glyph that repeats a label as noise.
    const arrow = selectors.querySelector('[aria-hidden="true"]');
    expect(arrow?.textContent).toBe('\u2192');
  });
});

/**
 * U1P-R3: a comparison is a fact about this session, so it must outlive the page that computed it.
 *
 * The Architect read the installed `REG_Compare_1440x900.png` and found two candidates named, the action
 * present, and nothing below: `Compare.tsx` kept `summary` in component state while `App.tsx` unmounts the
 * page on every navigation, and the only empty state on the page answers "fewer than two builds".
 *
 * Measured against `f01eec1` before any product file changed - see `target/u1p_r3_red.txt` for the run and
 * the digest-restored re-check of T3b: T1, T3, T3b, T4, T5, T6, T8 and T9 failed, because the behaviour they
 * require did not exist. T2 and T7 passed and were meant to: they are locks on the two states that must not
 * move while the gap is closed - the fewer-than-two flow and the session that has no comparison to show.
 *
 * No test here measures a pixel. jsdom lays nothing out, so the blank-cell question and the above-the-fold
 * question are answered by the installed screenshots, and the matching Overview contract says so in its own
 * name.
 */
describe('U1P-R3 the comparison survives the page it was made on', () => {
  it('T1 tells a reader with two builds and no comparison what to do next', async () => {
    await openCompare();

    const ready = await screen.findByRole('region', { name: 'Ready to compare' });
    expect(within(ready).getByRole('heading', { level: 2, name: 'Ready to compare' })).toBeDefined();
    expect(
      within(ready).getByText('Choose Old / Base and New / Target, then select Compare.'),
    ).toBeDefined();

    // A promise of a result is not a result: the ready state may not smuggle in counts, deltas or a verdict.
    expect(screen.queryByRole('region', { name: 'Build comparison' })).toBeNull();
    expect(screen.queryByText('What moved')).toBeNull();
    expect(compareMock).not.toHaveBeenCalled();
    // and the action it describes stays available, because the empty state must not replace the control.
    expect((screen.getByRole('button', { name: 'Compare' }) as HTMLButtonElement).disabled).toBe(false);
  });

  it('T2 keeps asking for another build when there is only one, and says something different', async () => {
    candidatesMock.mockResolvedValue(
      ok(candidatePage([candidate()], { total: 1, nextOffset: null })),
    );
    render(<App />);
    fireEvent.click(await screen.findByRole('button', { name: 'Compare page' }));

    const single = await screen.findByRole('region', { name: 'Nothing to compare yet' });
    expect(within(single).getByText(/Analyze another firmware build before comparing/)).toBeDefined();
    // The two empty states are different facts and must not collapse into one sentence.
    expect(screen.queryByRole('region', { name: 'Ready to compare' })).toBeNull();
  });

  it('T3 keeps the real comparison on screen after the reader leaves Compare and comes back', async () => {
    await openCompare();
    await runCompare();

    fireEvent.click(screen.getByRole('button', { name: 'Overview page' }));
    await screen.findByRole('heading', { level: 1, name: 'Overview' });
    fireEvent.click(screen.getByRole('button', { name: 'Compare page' }));

    const report = await screen.findByRole('region', { name: 'Build comparison' });
    expect(within(report).getByText('What moved')).toBeDefined();
    // Retaining is not recomputing: the shell was asked for exactly the one comparison the reader pressed.
    expect(compareMock).toHaveBeenCalledTimes(1);
  });

  it('T3b keeps the comparison through the pages scenario C2 names', async () => {
    await openCompare();
    await runCompare();

    // C2 lists four destinations, and the shell unmounts Compare for every one of them: `App.tsx` renders
    // one page at a time from a single ternary chain. Walking more than one is the difference between
    // proving the retention once and proving it is not a special case of the page the test visited.
    // Bundle & History is not walked here because the page reads three list commands this file's bridge
    // mock does not answer, and stubbing them would be a harness change with no bearing on what is under
    // test: the comparison is not stored in the page, so which page the reader went to cannot matter. The
    // installed capture covers that leg on the real shell.
    const c2Pages = ['Overview page', 'Analyze page', 'Release Gate page'];
    for (const page of c2Pages) {
      fireEvent.click(screen.getByRole('button', { name: page }));
      fireEvent.click(screen.getByRole('button', { name: 'Compare page' }));
      await screen.findByRole('region', { name: 'Build comparison' });
    }

    expect(compareMock).toHaveBeenCalledTimes(1);
    const [base, target] = pair();
    expect([base.value, target.value]).toEqual(['snap-old', 'snap-new']);
  });

  it('T4 restores the pair the result belongs to and keeps querying its own handle', async () => {
    await openCompare();
    await runCompare();
    const [beforeBase, beforeTarget] = pair();
    const shown = [beforeBase.value, beforeTarget.value];
    expect(shown).toEqual(['snap-old', 'snap-new']);

    fireEvent.click(screen.getByRole('button', { name: 'Analyze page' }));
    await screen.findByRole('heading', { level: 1, name: 'Analyze' });
    fireEvent.click(screen.getByRole('button', { name: 'Compare page' }));
    await screen.findByRole('region', { name: 'Build comparison' });

    const [afterBase, afterTarget] = pair();
    expect([afterBase.value, afterTarget.value]).toEqual(shown);
    // Every paged read still addresses the diff the reader computed - not a fresh handle, not a stale one.
    const handles = new Set(sectionChangesMock.mock.calls.map(([request]) => request.diffId));
    expect([...handles]).toEqual(['cmp-7f3a-1']);
    const symbolHandles = new Set(symbolChangesMock.mock.calls.map(([request]) => request.diffId));
    expect([...symbolHandles]).toEqual(['cmp-7f3a-1']);
  });

  it('T5 labels a restored comparison as the old pair the moment a selector moves', async () => {
    await openCompare();
    await runCompare();

    fireEvent.click(screen.getByRole('button', { name: 'Overview page' }));
    fireEvent.click(screen.getByRole('button', { name: 'Compare page' }));
    await screen.findByRole('region', { name: 'Build comparison' });

    fireEvent.click(screen.getByRole('button', { name: 'Swap' }));
    const lastGood = await screen.findByRole('region', { name: 'Last good comparison' });
    expect(
      within(lastGood).getByText(/The pair selected above is a different one; press Compare to move to it/),
    ).toBeDefined();
    // Swapping is choosing, not comparing.
    expect(compareMock).toHaveBeenCalledTimes(1);
  });

  it('T6 keeps a failed attempt and the surviving report attributed separately after a return', async () => {
    await openCompare();
    await runCompare();

    fireEvent.click(screen.getByRole('button', { name: 'Overview page' }));
    fireEvent.click(screen.getByRole('button', { name: 'Compare page' }));
    await screen.findByRole('region', { name: 'Build comparison' });

    compareMock.mockResolvedValueOnce(fail(PAIR_REFUSED));
    fireEvent.click(screen.getByRole('button', { name: 'Swap' }));
    fireEvent.click(screen.getByRole('button', { name: 'Compare' }));

    expect(await screen.findByText('ERR-DIFF-5001')).toBeDefined();
    const lastGood = await screen.findByRole('region', { name: 'Last good comparison' });
    expect(
      within(lastGood).getByText(/The attempt above produced no result, so nothing here was replaced/),
    ).toBeDefined();
  });

  it('T7 starts a fresh session showing no comparison, which is all a session with nothing persisted can show', async () => {
    await openCompare();

    // What this proves is the visible half of scenario C5: a new mount of the shell has no report, no
    // stale-labelled report, and asks the shell for no change rows. What it cannot prove is the negative
    // about storage - a test of absence needs the diff, and the diff adds no `localStorage`, no IPC read and
    // no schema field for a comparison, so there is nowhere a handle could be resurrected from. The installed
    // round proves the process half by relaunching the app against a store that already holds two builds.
    expect(compareMock).not.toHaveBeenCalled();
    expect(screen.queryByRole('region', { name: 'Build comparison' })).toBeNull();
    expect(screen.queryByRole('region', { name: 'Last good comparison' })).toBeNull();
    expect(sectionChangesMock).not.toHaveBeenCalled();
  });

  it('T8 records a comparison that lands while the reader is on another page', async () => {
    const pending = deferred<CompareSummaryDto>();
    compareMock.mockReturnValue(pending.promise);
    render(<App />);
    fireEvent.click(await screen.findByRole('button', { name: 'Compare page' }));
    await screen.findByRole('combobox', { name: 'New / Target' });
    await waitFor(() =>
      expect((screen.getByRole('button', { name: 'Compare' }) as HTMLButtonElement).disabled).toBe(
        false,
      ),
    );

    fireEvent.click(screen.getByRole('button', { name: 'Compare' }));
    // The reader does not wait: they move on, and the page that asked is gone before the answer arrives.
    fireEvent.click(screen.getByRole('button', { name: 'Overview page' }));
    await screen.findByRole('heading', { level: 1, name: 'Overview' });
    pending.resolve(ok(comparison()));

    fireEvent.click(screen.getByRole('button', { name: 'Compare page' }));
    const report = await screen.findByRole('region', { name: 'Build comparison' });
    expect(within(report).getByText('What moved')).toBeDefined();
  });

  it('T9 keeps the accessible names of the pickers, the action and the ready state together', async () => {
    await openCompare();
    await screen.findByRole('region', { name: 'Ready to compare' });

    const selectors = screen.getByRole('region', { name: 'Build selection' });
    expect(within(selectors).getByRole('combobox', { name: 'Old / Base' })).toBeDefined();
    expect(within(selectors).getByRole('combobox', { name: 'New / Target' })).toBeDefined();
    expect(within(selectors).getByRole('button', { name: 'Compare' })).toBeDefined();
    // The ready state explains the page; it does not become a live region that speaks over the reader's
    // own typing, and it does not hide the controls behind a dialog role.
    expect(screen.queryByRole('alert')).toBeNull();
    expect(screen.queryByRole('status', { name: 'Ready to compare' })).toBeNull();
  });
});
