//! P1's detail screen: the tabs, the bounded pages, the Evidence Inspector, and the bytes/KiB
//! switch from US-001.
//!
//! The claims these tests exist to protect are narrow, and they are all about *where* a decision is
//! made:
//!
//! - the screen never holds a row list it filtered or sorted itself; a filter, a sort and a page are
//!   a request to the shell, because `04_TECH/14` 3 puts the whole table on the Rust side;
//! - the unit switch changes text and nothing else: no command runs, no address changes, and an
//!   unknown value stays the word `Unknown` instead of becoming `0 KiB`;
//! - a detail query that fails belongs to the details area only; the summary the reader already
//!   trusted stays on screen, still bound to the snapshot it came from.
//!
//! The bridge is mocked, the way the intake screen tests do it: the plumbing has its own file.

import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { App } from './App';
import {
  analyzeSelection,
  queryEvidence,
  querySections,
  querySymbols,
  selectArtifact,
} from './ipc/bridge';
import type { IpcOutcome } from './ipc/bridge';
import type {
  AnalysisSummaryDto,
  EvidencePageDto,
  EvidenceRequestDto,
  EvidenceRowDto,
  ErrorEnvelopeDto,
  SectionPageDto,
  SectionRequestDto,
  SelectionDto,
  SymbolPageDto,
  SymbolRequestDto,
} from './ipc/types';

vi.mock('./ipc/bridge', () => ({
  selectArtifact: vi.fn(),
  attachMap: vi.fn(),
  clearMap: vi.fn(),
  analyzeSelection: vi.fn(),
  querySections: vi.fn(),
  querySymbols: vi.fn(),
  queryEvidence: vi.fn(),
  setWindowTitle: vi.fn(() => Promise.resolve({ ok: true, value: null })),
  getAppIdentity: vi.fn(() => Promise.resolve({ ok: true, value: {} })),
  listHistoryBuilds: vi.fn(),
  listHistoryGateRuns: vi.fn(),
  listHistoryReleases: vi.fn(),
}));

const selectMock = vi.mocked(selectArtifact);
const analyzeMock = vi.mocked(analyzeSelection);
const sectionsMock = vi.mocked(querySections);
const symbolsMock = vi.mocked(querySymbols);
const evidenceMock = vi.mocked(queryEvidence);

function ok<T>(value: T): IpcOutcome<T> {
  return { ok: true, value };
}

function fail<T>(envelope: ErrorEnvelopeDto): IpcOutcome<T> {
  return { ok: false, envelope };
}

const SNAPSHOT_ID = 'snap-deadbeef-p0-normalize-1';

const NOT_FOUND: ErrorEnvelopeDto = {
  code: 'ERR-STORAGE-4005',
  message: 'no build was found with id snap-deadbeef-p0-normalize-1',
  operationId: 'op-77',
  details: null,
  remediation: 'Reload the project, then pick the build again.',
};

function selection(): SelectionDto {
  return {
    selectionId: 'sel-7f3a-2',
    fileName: 'app.elf',
    mapFileName: null,
    mapAttached: false,
  };
}

/** 5,432 bytes is 5.3 KiB, and the address below must survive the switch untouched. */
function summary(overrides: Partial<AnalysisSummaryDto> = {}): AnalysisSummaryDto {
  return {
    source: 'artifact',
    artifact: {
      fileName: 'app.elf',
      kind: 'elf',
      sha256: 'b'.repeat(64),
      byteSize: 5432,
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
      snapshotId: SNAPSHOT_ID,
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
        bytes: 4200,
        unattributed: [],
        reason: null,
      },
      runtimeRamFootprint: {
        state: 'unknown',
        classification: 'unknown',
        bytes: null,
        unattributed: [],
        reason: 'no RAM region evidence was supplied',
      },
      dualAccountedSections: ['section:3(.data)'],
      excludedMetadataBytes: 210,
    },
    sectionCount: 17,
    symbolCount: 32,
    capabilities: {
      elf: 'supported',
      sections: 'available',
      symbols: 'available',
      // The absence of debug info is a fact the screen must keep showing rather than crash on.
      debugInfo: 'unavailable',
      map: 'provided',
      objectAttribution: 'unavailable',
      git: 'unknown',
    },
    evidenceSummary: { total: 11, observed: 11, derived: 0, declared: 0, unknown: 0 },
    ...overrides,
  } as AnalysisSummaryDto;
}

function sectionPage(overrides: Partial<SectionPageDto> = {}): SectionPageDto {
  return {
    rows: [
      {
        index: 1,
        name: '.text',
        nameUnknownReason: null,
        role: 'Code',
        alloc: true,
        write: false,
        execute: true,
        virtualAddress: '0x08000000',
        virtualAddressUnknownReason: null,
        loadAddress: '0x08000000',
        loadAddressUnknownReason: null,
        fileOffset: '0x00001000',
        fileOffsetUnknownReason: null,
        fileSize: 60,
        memorySize: 60,
        memorySizeUnknownReason: null,
        region: null,
        regionUnknownReason: 'no memory-region evidence was supplied',
      },
      {
        index: 3,
        name: '.data',
        nameUnknownReason: null,
        role: 'InitializedData',
        alloc: true,
        write: true,
        execute: false,
        virtualAddress: '0x20000000',
        virtualAddressUnknownReason: null,
        loadAddress: '0x0800005c',
        loadAddressUnknownReason: null,
        fileOffset: '0x00002000',
        fileOffsetUnknownReason: null,
        fileSize: 4,
        memorySize: 4,
        memorySizeUnknownReason: null,
        region: 'RAM',
        regionUnknownReason: null,
      },
    ],
    total: 17,
    offset: 0,
    limit: 100,
    nextOffset: null,
    ...overrides,
  };
}

function symbolPage(overrides: Partial<SymbolPageDto> = {}): SymbolPageDto {
  return {
    rows: [
      {
        ordinal: 24,
        name: 'staging_area',
        nameUnknownReason: null,
        address: '0x20000008',
        addressUnknownReason: null,
        size: 64,
        sizeUnknownReason: null,
        kind: 'Object',
        binding: 'Global',
        sectionRef: 'Index(5)',
      },
      {
        // The shape the acceptance list cares about: no size, with the reason, never a zero.
        ordinal: 25,
        name: 'g_threshold',
        nameUnknownReason: null,
        address: '0x20000000',
        addressUnknownReason: null,
        size: null,
        sizeUnknownReason: 'the symbol entry records size 0',
        kind: 'Object',
        binding: 'Global',
        sectionRef: 'Index(3)',
      },
    ],
    total: 32,
    offset: 0,
    limit: 100,
    nextOffset: 2,
    ...overrides,
  };
}

function evidencePage(overrides: Partial<EvidencePageDto> = {}): EvidencePageDto {
  return {
    rows: [
      {
        id: 'ev-sha256',
        field: 'sha256',
        classification: 'observed',
        sourceType: 'FileSystem',
        sourceLocator: 'file:app.elf',
        rawValue: 'b'.repeat(64),
        rule: 'streaming-sha256/64KiB',
        confidence: null,
      },
      {
        id: 'ev-entry',
        field: 'entry_point',
        classification: 'observed',
        sourceType: 'ElfFileHeader',
        sourceLocator: 'elf.file_header.e_entry',
        rawValue: '0x080003f8',
        rule: 'elf-header/e_entry',
        confidence: null,
      },
    ],
    total: 11,
    offset: 0,
    limit: 100,
    nextOffset: null,
    ...overrides,
  };
}

/** Drive the screen to a last-good summary, then hand control back to the test. */
async function analyzeOk(overrides: Partial<AnalysisSummaryDto> = {}) {
  selectMock.mockResolvedValue(ok(selection()));
  analyzeMock.mockResolvedValue(ok(summary(overrides)));

  fireEvent.click(screen.getByRole('button', { name: 'Choose firmware artifact' }));
  await waitFor(() => expect(selectMock).toHaveBeenCalledTimes(1));
  fireEvent.click(screen.getByRole('button', { name: 'Analyze' }));
  await waitFor(() => expect(analyzeMock).toHaveBeenCalledTimes(1));
}

beforeEach(() => {
  sectionsMock.mockReset();
  symbolsMock.mockReset();
  evidenceMock.mockReset();
  analyzeMock.mockReset();
  selectMock.mockReset();
  sectionsMock.mockResolvedValue(ok(sectionPage()));
  symbolsMock.mockResolvedValue(ok(symbolPage()));
  evidenceMock.mockResolvedValue(ok(evidencePage()));
});

describe('before anything has been analyzed', () => {
  it('asks the shell for no rows at all', () => {
    render(<App />);

    expect(sectionsMock).not.toHaveBeenCalled();
    expect(symbolsMock).not.toHaveBeenCalled();
    expect(evidenceMock).not.toHaveBeenCalled();
  });
});

describe('after a summary exists', () => {
  it('asks for the largest stored payload and for the active tab, by snapshot id', async () => {
    render(<App />);
    await analyzeOk();

    await waitFor(() => expect(sectionsMock).toHaveBeenCalled());
    const requests = sectionsMock.mock.calls.map(([request]) => request);
    expect(requests.some((request) => request.snapshotId === SNAPSHOT_ID)).toBe(true);
    expect(
      requests.some(
        (request) => request.sort === 'fileSize' && request.direction === 'desc' && request.limit === 5,
      ),
    ).toBe(true);
    const contributors = await screen.findByRole('region', { name: 'Largest stored payload' });
    expect(within(contributors).getByText('.text')).toBeDefined();
    // One row answers one question: which section, what it holds, how many bytes. The role and the
    // size belong to the same row the name is on.
    const contributorRow = within(contributors).getByText('.text').closest('li');
    expect(contributorRow?.textContent).toContain('Code');
    expect(contributorRow?.textContent).toContain('60 bytes');

    const tabs = await screen.findByRole('tablist', { name: 'Analyze details' });
    expect(within(tabs).getAllByRole('tab').map((tab) => tab.textContent)).toEqual([
      'Sections',
      'Symbols',
      'Evidence',
    ]);
  });

  it('keeps the debug-info row visible and loads details anyway', async () => {
    render(<App />);
    await analyzeOk();

    const capabilities = await screen.findByRole('region', { name: 'Capabilities' });
    expect(within(capabilities).getByText('Debug info')).toBeDefined();
    expect(within(capabilities).getAllByText('unavailable').length).toBeGreaterThan(0);
    await waitFor(() => expect(sectionsMock).toHaveBeenCalled());
  });

  it('names the attribution capability by the scope it actually reports', async () => {
    // L19: this row answers whether one build can attribute rows to an object or module. Compare
    // asks a different question with similar words — whether any object file changed between two
    // builds — so the two labels must not collapse back into the one phrase.
    render(<App />);
    await analyzeOk();

    const capabilities = await screen.findByRole('region', { name: 'Capabilities' });
    expect(within(capabilities).getByText('Object/module attribution')).toBeDefined();
    expect(capabilities.textContent).not.toContain('Object attribution');
  });

  it('renders an unknown region as the word Unknown and its reason, never as a blank', async () => {
    render(<App />);
    await analyzeOk();

    const panel = await screen.findByRole('tabpanel', { name: 'Sections' });
    expect(within(panel).getAllByText('Unknown').length).toBeGreaterThan(0);
    const textRow = within(panel)
      .getByText('.text')
      .closest('tr');
    expect(textRow?.textContent ?? '').toContain('no memory-region evidence was supplied');
  });

  // L6 and L7: the file offset and the symbol address were the two numeric facts whose reason the
  // store threw away, so their cells read `Unknown` and nothing else. Migration 0005 keeps the
  // reason, and this is the assertion that says the window now uses it.
  it('names the reason for an unknown file offset and an unknown symbol address', async () => {
    const offsetReason = 'the section has no file range; it occupies no bytes on disk';
    const addressReason = 'the symbol entry is undefined and carries no value';
    sectionsMock.mockResolvedValue(
      ok(
        sectionPage({
          rows: [
            {
              index: 7,
              name: '.bss',
              nameUnknownReason: null,
              role: 'ZeroInitializedData',
              alloc: true,
              write: true,
              execute: false,
              virtualAddress: '0x20000040',
              virtualAddressUnknownReason: null,
              loadAddress: '0x20000040',
              loadAddressUnknownReason: null,
              fileOffset: null,
              fileOffsetUnknownReason: offsetReason,
              fileSize: 0,
              memorySize: 16,
              memorySizeUnknownReason: null,
              region: 'RAM',
              regionUnknownReason: null,
            },
          ],
          total: 1,
        }),
      ),
    );
    symbolsMock.mockResolvedValue(
      ok(
        symbolPage({
          rows: [
            {
              ordinal: 0,
              name: 'undefined_entry',
              nameUnknownReason: null,
              address: null,
              addressUnknownReason: addressReason,
              size: 4,
              sizeUnknownReason: null,
              kind: 'Object',
              binding: 'Global',
              sectionRef: 'Undefined',
            },
          ],
          total: 1,
        }),
      ),
    );

    render(<App />);
    await analyzeOk();

    const sections = await screen.findByRole('tabpanel', { name: 'Sections' });
    const sectionRow = within(sections).getByText('.bss').closest('tr');
    expect(sectionRow?.textContent ?? '').toContain('Unknown');
    expect(sectionRow?.textContent ?? '').toContain(offsetReason);

    fireEvent.click(screen.getByRole('tab', { name: 'Symbols' }));
    const symbols = await screen.findByRole('tabpanel', { name: 'Symbols' });
    const symbolRow = within(symbols).getByText('undefined_entry').closest('tr');
    expect(symbolRow?.textContent ?? '').toContain('Unknown');
    expect(symbolRow?.textContent ?? '').toContain(addressReason);
  });
});

describe('the bytes / KiB switch', () => {
  it('re-labels every byte figure and calls no command at all', async () => {
    render(<App />);
    await analyzeOk();
    await waitFor(() => expect(sectionsMock).toHaveBeenCalled());

    const callsBefore = [
      sectionsMock.mock.calls.length,
      symbolsMock.mock.calls.length,
      evidenceMock.mock.calls.length,
    ];

    fireEvent.click(screen.getByRole('radio', { name: 'KiB' }));

    expect(await screen.findByText('5.30 KiB')).toBeDefined();
    expect(screen.getByText('4.10 KiB')).toBeDefined();
    expect(screen.getAllByText('0.0586 KiB').length).toBeGreaterThan(0);
    expect([
      sectionsMock.mock.calls.length,
      symbolsMock.mock.calls.length,
      evidenceMock.mock.calls.length,
    ]).toEqual(callsBefore);
  });

  it('leaves addresses, offsets and counts exactly as they were', async () => {
    render(<App />);
    await analyzeOk();

    const panel = await screen.findByRole('tabpanel', { name: 'Sections' });
    const before = within(panel).getAllByText(/0x/).map((cell) => cell.textContent);
    expect(before.length).toBeGreaterThan(0);

    fireEvent.click(screen.getByRole('radio', { name: 'KiB' }));

    const after = within(panel).getAllByText(/0x/).map((cell) => cell.textContent);
    expect(after).toEqual(before);
    expect(after).toContain('0x08000000');
    expect(after).toContain('0x00001000');
  });

  it('never paints one tab rows under another tab columns', async () => {
    // React's own key check is what caught this: for the render between a tab click and the effect
    // that clears the page, the symbol table was handed the *sections* page, whose rows carry no
    // ordinal. A page that belongs to another tab must not be re-labelled, so the guard is
    // structural rather than a matter of clearing state in time.
    const lines: string[] = [];
    const spy = vi.spyOn(console, 'error').mockImplementation((...args: unknown[]) => {
      lines.push(args.map(String).join(' '));
    });

    render(<App />);
    await analyzeOk();
    await screen.findByRole('tabpanel', { name: 'Sections' });

    fireEvent.click(screen.getByRole('tab', { name: 'Symbols' }));
    await screen.findByRole('tabpanel', { name: 'Symbols' });

    spy.mockRestore();
    expect(lines.filter((line) => line.includes('unique "key"'))).toEqual([]);
  });

  it('shows Unknown rather than 0 KiB for a value that has no number', async () => {
    render(<App />);
    await analyzeOk();

    // The runtime footprint of this summary is unknown, and one symbol has no size.
    const memory = await screen.findByRole('region', { name: 'Memory' });
    fireEvent.click(screen.getByRole('radio', { name: 'KiB' }));

    expect(within(memory).getAllByText('Unknown').length).toBeGreaterThan(0);
    expect(within(memory).queryByText(/^0(\.00)? KiB$/)).toBeNull();

    fireEvent.click(screen.getByRole('tab', { name: 'Symbols' }));
    const symbols = await screen.findByRole('tabpanel', { name: 'Symbols' });
    expect(within(symbols).getAllByText('Unknown').length).toBeGreaterThan(0);
    expect(within(symbols).queryByText(/^0(\.00)? KiB$/)).toBeNull();
  });

  it('defaults to bytes', async () => {
    render(<App />);
    await analyzeOk();

    const bytes = (await screen.findByRole('radio', { name: 'Bytes' })) as HTMLInputElement;
    const kib = screen.getByRole('radio', { name: 'KiB' }) as HTMLInputElement;
    expect(bytes.checked).toBe(true);
    expect(kib.checked).toBe(false);
    expect(screen.getByText('5,432 bytes')).toBeDefined();
  });
});

describe('filtering, sorting and paging', () => {
  it('sends a filter to the shell instead of filtering the rows it already has', async () => {
    render(<App />);
    await analyzeOk();
    await screen.findByRole('tabpanel', { name: 'Sections' });

    const input = await screen.findByRole('textbox', { name: 'Filter sections by name' });
    fireEvent.change(input, { target: { value: 'ota' } });
    fireEvent.click(screen.getByRole('button', { name: 'Apply filter' }));

    await waitFor(() => {
      const last = sectionsMock.mock.calls.at(-1)?.[0] as SectionRequestDto;
      expect(last.filter).toBe('ota');
      expect(last.offset).toBe(0);
    });
  });

  it('clearing the filter and applying again asks for the whole table', async () => {
    // Observed in the real window: an empty filter box did not lift the filter that was already
    // applied, so the screen kept showing one row and said "1 of 1".
    render(<App />);
    await analyzeOk();
    fireEvent.click(screen.getByRole('tab', { name: 'Symbols' }));

    const input = await screen.findByRole('textbox', { name: 'Filter symbols by name' });
    fireEvent.change(input, { target: { value: 'task' } });
    fireEvent.click(screen.getByRole('button', { name: 'Apply filter' }));
    await waitFor(() => {
      const last = symbolsMock.mock.calls.at(-1)?.[0] as SymbolRequestDto;
      expect(last.filter).toBe('task');
    });

    fireEvent.change(screen.getByRole('textbox', { name: 'Filter symbols by name' }), {
      target: { value: '' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Apply filter' }));
    await waitFor(() => {
      const last = symbolsMock.mock.calls.at(-1)?.[0] as SymbolRequestDto;
      expect(last.filter).toBeNull();
    });
  });

  it('asks for the other direction when the same column is chosen twice', async () => {
    render(<App />);
    await analyzeOk();
    await screen.findByRole('tabpanel', { name: 'Sections' });

    // Re-queryed from the document each time: a page in flight replaces its own rows, and a node
    // held across that update belongs to the tree that was just removed.
    fireEvent.click(screen.getByRole('button', { name: 'Sort by Name' }));
    await waitFor(() => {
      const last = sectionsMock.mock.calls.at(-1)?.[0] as SectionRequestDto;
      expect(last.sort).toBe('name');
      expect(last.direction).toBe('asc');
    });

    await screen.findByRole('tabpanel', { name: 'Sections' });
    fireEvent.click(screen.getByRole('button', { name: 'Sort by Name' }));
    await waitFor(() => {
      const last = sectionsMock.mock.calls.at(-1)?.[0] as SectionRequestDto;
      expect(last.direction).toBe('desc');
    });
  });

  it('asks for the offset the shell reported, and says which rows are on screen', async () => {
    sectionsMock.mockResolvedValue(ok(sectionPage({ nextOffset: 10, rows: sectionPage().rows })));
    render(<App />);
    await analyzeOk();

    const status = await screen.findByText(/Showing 1 to 2 of 17/);
    expect(status).toBeDefined();

    fireEvent.click(screen.getByRole('button', { name: 'Next page' }));
    await waitFor(() => {
      const pageRequests = sectionsMock.mock.calls
        .map(([request]) => request)
        .filter((request) => request.offset === 10);
      expect(pageRequests.length).toBeGreaterThan(0);
    });

    fireEvent.click(screen.getByRole('button', { name: 'Previous page' }));
    await waitFor(() => {
      const back = sectionsMock.mock.calls
        .map(([request]) => request)
        .filter((request) => request.offset === 0);
      expect(back.length).toBeGreaterThan(1);
    });
  });

  it('queries symbols through the symbol command with its own sorts', async () => {
    render(<App />);
    await analyzeOk();

    fireEvent.click(screen.getByRole('tab', { name: 'Symbols' }));
    await waitFor(() => expect(symbolsMock).toHaveBeenCalled());

    const panel = await screen.findByRole('tabpanel', { name: 'Symbols' });
    fireEvent.click(within(panel).getByRole('button', { name: 'Sort by Size' }));
    await waitFor(() => {
      const last = symbolsMock.mock.calls.at(-1)?.[0] as SymbolRequestDto;
      expect(last.sort).toBe('size');
    });
    expect(within(panel).getByText('staging_area')).toBeDefined();
    expect(within(panel).getByText('64 bytes')).toBeDefined();
  });

  it('filters evidence by class through the evidence command', async () => {
    render(<App />);
    await analyzeOk();

    fireEvent.click(screen.getByRole('tab', { name: 'Evidence' }));
    await waitFor(() => expect(evidenceMock).toHaveBeenCalled());

    const classSelect = await screen.findByRole('combobox', { name: 'Evidence class' });
    fireEvent.change(classSelect, { target: { value: 'observed' } });

    await waitFor(() => {
      const last = evidenceMock.mock.calls.at(-1)?.[0] as EvidenceRequestDto;
      expect(last.classification).toBe('observed');
    });
  });
});

function evidenceRow(overrides: Partial<EvidenceRowDto> = {}): EvidenceRowDto {
  return {
    id: 'ev-basis',
    field: 'memory_basis',
    classification: 'observed',
    sourceType: 'ElfProgramHeader',
    sourceLocator: 'elf.section_header[1] + elf:sh_flags',
    rawValue: '0x08000030',
    rule: 'elf-address-and-flags',
    confidence: null,
    ...overrides,
  };
}

describe('the evidence inspector', () => {
  it('shows where a fact came from, rule and locator included', async () => {
    render(<App />);
    await analyzeOk();

    fireEvent.click(screen.getByRole('tab', { name: 'Evidence' }));
    const panel = await screen.findByRole('tabpanel', { name: 'Evidence' });

    fireEvent.click(within(panel).getByRole('button', { name: 'Inspect entry_point' }));

    const inspector = await screen.findByRole('region', { name: 'Evidence detail' });
    expect(within(inspector).getByText('elf.file_header.e_entry')).toBeDefined();
    expect(within(inspector).getByText('elf-header/e_entry')).toBeDefined();
    expect(within(inspector).getByText('0x080003f8')).toBeDefined();
    expect(within(inspector).getByText('observed')).toBeDefined();
  });

  it('names the legacy program-header source by what it actually observed', async () => {
    // L15 Option E keeps `ElfProgramHeader` in storage and `elf.program-header` on the wire forever, so
    // the only place the inaccuracy can still reach a human is this row. The caption is presentation only:
    // the row object handed to this test still carries the identifier it was stored under.
    const legacy = evidenceRow();
    evidenceMock.mockResolvedValue(ok(evidencePage({ rows: [legacy], total: 1 })));
    render(<App />);
    await analyzeOk();

    fireEvent.click(screen.getByRole('tab', { name: 'Evidence' }));
    const panel = await screen.findByRole('tabpanel', { name: 'Evidence' });
    fireEvent.click(within(panel).getByRole('button', { name: 'Inspect memory_basis' }));

    const inspector = await screen.findByRole('region', { name: 'Evidence detail' });
    expect(within(inspector).getByText('ELF address + flags evidence')).toBeDefined();
    expect(within(inspector).queryByText('ElfProgramHeader')).toBeNull();
    expect(within(inspector).queryByText('elf.program-header')).toBeNull();
    // the locator keeps saying where to go and check, which is the whole point of the panel
    expect(within(inspector).getByText('elf.section_header[1] + elf:sh_flags')).toBeDefined();
    expect(legacy.sourceType).toBe('ElfProgramHeader');
  });

  it('names the same source when it arrives in its wire spelling, and leaves every other name alone', async () => {
    // Two namespaces, one legacy identifier: storage writes the Debug name and `analysis:1` writes this one.
    const both = [
      evidenceRow({ id: 'ev-wire', sourceType: 'elf.program-header' }),
      evidenceRow({
        id: 'ev-symbol',
        field: 'symbol_table',
        sourceType: 'ElfSymbolTable',
        sourceLocator: 'elf.section_header[7].sh_offset',
      }),
    ];
    evidenceMock.mockResolvedValue(ok(evidencePage({ rows: both, total: 2 })));
    render(<App />);
    await analyzeOk();

    fireEvent.click(screen.getByRole('tab', { name: 'Evidence' }));
    const panel = await screen.findByRole('tabpanel', { name: 'Evidence' });

    fireEvent.click(within(panel).getByRole('button', { name: 'Inspect memory_basis' }));
    const captioned = await screen.findByRole('region', { name: 'Evidence detail' });
    expect(within(captioned).getByText('ELF address + flags evidence')).toBeDefined();

    fireEvent.click(within(panel).getByRole('button', { name: 'Inspect symbol_table' }));
    const plain = await screen.findByRole('region', { name: 'Evidence detail' });
    expect(within(plain).getByText('ElfSymbolTable')).toBeDefined();
  });

  it('places the inspector above the table, where the click already landed', async () => {
    // Observed in the real window: the panel sat below a table taller than the viewport, so
    // pressing Inspect produced no visible change at the default minimum window height.
    render(<App />);
    await analyzeOk();

    fireEvent.click(screen.getByRole('tab', { name: 'Evidence' }));
    const panel = await screen.findByRole('tabpanel', { name: 'Evidence' });
    fireEvent.click(within(panel).getByRole('button', { name: 'Inspect entry_point' }));

    const inspector = await screen.findByRole('region', { name: 'Evidence detail' });
    const table = within(panel).getByRole('table');
    const relation = inspector.compareDocumentPosition(table);
    expect(relation & Node.DOCUMENT_POSITION_FOLLOWING).toBe(Node.DOCUMENT_POSITION_FOLLOWING);
  });

  it('keeps the Inspect action in the first column, where no wide value can push it away', async () => {
    // Observed in the real window at the minimum supported width: the value and locator columns are
    // wider than the window, so the table owns its own scroll axis - and the action that opens the
    // inspector was the thing hanging off the right edge.
    render(<App />);
    await analyzeOk();

    fireEvent.click(screen.getByRole('tab', { name: 'Evidence' }));
    const panel = await screen.findByRole('tabpanel', { name: 'Evidence' });

    const firstRow = within(panel).getAllByRole('row')[1];
    const cells = within(firstRow as HTMLElement).getAllByRole('cell');
    expect(within(cells[0] as HTMLElement).getByRole('button', { name: 'Inspect sha256' })).toBeDefined();
  });
});

describe('the filter box', () => {
  it('offers no browser autofill for a name typed to query firmware', async () => {
    render(<App />);
    await analyzeOk();

    const input = await screen.findByRole('textbox', { name: 'Filter sections by name' });
    expect(input.getAttribute('autocomplete')).toBe('off');
  });
});

describe('a detail query that fails', () => {
  it('reports inside the details area and leaves the summary readable', async () => {
    sectionsMock.mockImplementation(async (request: SectionRequestDto) => {
      if (request.sort === 'fileSize') {
        return ok(sectionPage());
      }
      return fail(NOT_FOUND);
    });

    render(<App />);
    await analyzeOk();

    const alert = await screen.findByRole('alert', { name: 'Detail query failed' });
    expect(within(alert).getByText('ERR-STORAGE-4005')).toBeDefined();

    // The summary the reader already trusted is untouched: its artifact hash is still on screen.
    const artifact = await screen.findByRole('region', { name: 'Artifact' });
    expect(within(artifact).getByText('b'.repeat(64))).toBeDefined();
  });

  it('stays bound to the last good snapshot when a later analysis fails', async () => {
    render(<App />);
    await analyzeOk();
    await waitFor(() => expect(sectionsMock).toHaveBeenCalled());
    sectionsMock.mockClear();

    analyzeMock.mockResolvedValue(fail(NOT_FOUND));
    fireEvent.click(screen.getByRole('button', { name: 'Analyze' }));
    await waitFor(() => expect(analyzeMock.mock.calls.length).toBe(2));

    // The candidate that failed is named, and any detail read still belongs to the old snapshot.
    fireEvent.click(screen.getByRole('tab', { name: 'Evidence' }));
    await waitFor(() => expect(evidenceMock.mock.calls.length).toBeGreaterThan(0));
    const requests = evidenceMock.mock.calls.map(([request]) => request);
    expect(requests.every((request) => request.snapshotId === SNAPSHOT_ID)).toBe(true);
    expect(await screen.findByRole('note')).toBeDefined();
  });
});
