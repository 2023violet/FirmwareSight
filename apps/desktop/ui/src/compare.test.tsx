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

describe('the rail and the pages it lists', () => {
  it('lists the stages this build has, and marks the page the reader is on', async () => {
    render(<App />);

    const rail = await screen.findByRole('navigation', { name: 'Pages' });
    // P3 added a third real page, so this list is the build's own inventory: an entry appears when the
    // stage exists and nowhere else does its word appear (prompt §43).
    expect(within(rail).getAllByRole('button').map((item) => item.textContent)).toEqual([
      'Analyze',
      'Compare',
      'Release',
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
    expect(memory.textContent).toContain('weakest basis elf-address-and-flags');
  });

  it('re-labels the same figures in KiB and asks the shell for nothing', async () => {
    await openCompare();
    await runCompare();
    const calls = compareMock.mock.calls.length;
    const changeCalls = sectionChangesMock.mock.calls.length;

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
    const row = within(table).getByText('.noinit').closest('tr') as HTMLTableRowElement;
    expect(within(row).getByText('Added')).toBeDefined();
    expect(within(row).getAllByText('Not present').length).toBe(2);
    expect(row.textContent).not.toContain('Old 0 bytes');
  });

  it('shows a removed row as absent on the target side', async () => {
    await openCompare();
    await runCompare();

    const table = await screen.findByRole('region', { name: 'Section Changes' });
    const row = within(table).getByText('.oldboot').closest('tr') as HTMLTableRowElement;
    expect(within(row).getByText('Removed')).toBeDefined();
    expect(within(row).getAllByText('Not present').length).toBeGreaterThan(0);
    expect(row.textContent).toContain('128 bytes');
  });

  it('keeps a size that was never recorded as Unknown with its reason', async () => {
    await openCompare();
    await runCompare();

    const table = await screen.findByRole('region', { name: 'Symbol Changes' });
    const row = within(table).getByText('g_threshold').closest('tr') as HTMLTableRowElement;
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

    const table = await screen.findByRole('region', { name: 'Symbol Changes' });
    expect(within(table).getByText(/Showing 1 to 3 of 3 symbols/)).toBeDefined();

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
    expect(screen.getByRole('button', { name: 'Show sections changes for .text' })).toBeDefined();
    expect(screen.getByRole('button', { name: 'Show sections changes for .noinit' })).toBeDefined();
    // The full tables are asked for with no kind filter, so nothing hides behind the ranking.
    const requests = sectionChangesMock.mock.calls.map(([request]) => request);
    expect(requests.some((request) => request.changeKind === null && request.limit === null)).toBe(
      true,
    );
    expect(await screen.findByRole('region', { name: 'Section Changes' })).toBeDefined();
    expect(screen.getByRole('region', { name: 'Symbol Changes' })).toBeDefined();
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

    const table = await screen.findByRole('region', { name: 'Section Changes' });
    const headers = within(table).getAllByRole('columnheader');
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
