import { act, fireEvent, screen, waitFor, within } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { App } from './App';
import { Analyze } from './Analyze';
import { analyzeSelection, attachMap, clearMap, exportAnalysisVisual, selectArtifact } from './ipc/bridge';
import type { IpcOutcome } from './ipc/bridge';
import type { AnalysisSummaryDto, ErrorEnvelopeDto, SelectionDto } from './ipc/types';
import { render } from './test/render';
import { precedes } from './test/order';
import { acceptDroppedFiles, subscribeToDrops } from './ipc/dropEvents';
import type { DropEventDto } from './ipc/types';

vi.mock('./ipc/dropEvents', () => ({
  subscribeToDrops: vi.fn(() => () => undefined),
  acceptDroppedFiles: vi.fn(),
}));

// The bridge is the boundary; mocking it keeps these tests about what the screen shows rather than
// about Tauri's invoke plumbing, which `ipc/bridge.test.ts` covers.
//
// The three detail commands are stubbed with an empty page: this file is about intake, and a screen
// that shows a summary now also asks for its details. An empty page keeps that second surface quiet
// without hiding anything this file claims.
//
// The Compare commands are stubbed too, and the candidate list comes back empty: the navigation test
// opens that page, and a build history this file does not describe must not appear there either.
vi.mock('./ipc/bridge', () => ({
  selectArtifact: vi.fn(),
  attachMap: vi.fn(),
  clearMap: vi.fn(),
  analyzeSelection: vi.fn(),
  exportAnalysisVisual: vi.fn(() => Promise.resolve({ ok: true, value: { status: 'cancelled', fileName: null, format: 'svg' } })),
  querySections: vi.fn(() =>
    Promise.resolve({
      ok: true,
      value: { rows: [], total: 0, offset: 0, limit: 100, nextOffset: null },
    }),
  ),
  querySymbols: vi.fn(() =>
    Promise.resolve({
      ok: true,
      value: { rows: [], total: 0, offset: 0, limit: 100, nextOffset: null },
    }),
  ),
  queryEvidence: vi.fn(() =>
    Promise.resolve({
      ok: true,
      value: { rows: [], total: 0, offset: 0, limit: 100, nextOffset: null },
    }),
  ),
  listCompareCandidates: vi.fn(() =>
    Promise.resolve({
      ok: true,
      value: { rows: [], total: 0, offset: 0, limit: 25, nextOffset: null },
    }),
  ),
  compareSnapshots: vi.fn(() => Promise.resolve({ ok: true, value: {} })),
  querySectionChanges: vi.fn(() =>
    Promise.resolve({
      ok: true,
      value: { rows: [], total: 0, offset: 0, limit: 100, nextOffset: null },
    }),
  ),
  querySymbolChanges: vi.fn(() =>
    Promise.resolve({
      ok: true,
      value: { rows: [], total: 0, offset: 0, limit: 100, nextOffset: null },
    }),
  ),
  exportCompareJson: vi.fn(() =>
    Promise.resolve({ ok: true, value: { status: 'cancelled', fileName: null, format: 'json' } }),
  ),
  exportCompareHtml: vi.fn(() =>
    Promise.resolve({ ok: true, value: { status: 'cancelled', fileName: null, format: 'html' } }),
  ),
  // The shell names the window, so mounting <App/> at all calls this one. An outcome-shaped promise
  // keeps the title fix out of what this file claims without pretending the page asked for nothing.
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
  listHistoryBuilds: vi.fn(() =>
    Promise.resolve({
      ok: true,
      value: { rows: [], total: 0, offset: 0, limit: 25, nextOffset: null },
    }),
  ),
  listHistoryGateRuns: vi.fn(() =>
    Promise.resolve({
      ok: true,
      value: { rows: [], total: 0, offset: 0, limit: 25, nextOffset: null },
    }),
  ),
  listHistoryReleases: vi.fn(() =>
    Promise.resolve({
      ok: true,
      value: { rows: [], total: 0, offset: 0, limit: 25, nextOffset: null },
    }),
  ),
}));

const selectMock = vi.mocked(selectArtifact);
const attachMock = vi.mocked(attachMap);
const clearMock = vi.mocked(clearMap);
const analyzeMock = vi.mocked(analyzeSelection);
const exportAnalysisVisualMock = vi.mocked(exportAnalysisVisual);

function ok<T>(value: T): IpcOutcome<T> {
  return { ok: true, value };
}

function fail(envelope: ErrorEnvelopeDto): IpcOutcome<never> {
  return { ok: false, envelope };
}

/** Where the artifact actually sits. It must never reach the screen. */
const MACHINE_PATH = 'D:\\work\\secret-build\\output';

function selection(overrides: Partial<SelectionDto> = {}): SelectionDto {
  const base: SelectionDto = {
    selectionId: 'sel-7f3a-2',
    fileName: 'app.elf',
    mapFileName: null,
    mapAttached: false,
  };
  return { ...base, ...overrides };
}

const WITH_MAP: SelectionDto = selection({
  mapFileName: 'app.map',
  mapAttached: true,
});

const PARSE_FAILURE: ErrorEnvelopeDto = {
  code: 'ERR-PARSE-2002',
  message: 'Could not parse artifact as ELF.',
  operationId: 'op-1a2b',
  details: 'malformed artifact: truncated section table',
  remediation: 'Choose the linker ELF output rather than a stripped or truncated copy.',
};

/**
 * A summary spelled out field by field, exactly like the generated type.
 *
 * Nothing is omitted: a test fixture that skipped fields would hide a real shape change, and the
 * generated contract requires every one of them (nulls included).
 */
function summary(overrides: Partial<AnalysisSummaryDto> = {}): AnalysisSummaryDto {
  const base: AnalysisSummaryDto = {
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
      buildId: 'BuildId("9e42c1")',
      buildIdUnknownReason: null,
    },
    identity: {
      schema: 'firmwaresight.analyze/p0-internal-1',
      schemaStability: 'p0-internal',
      snapshotId: 'snap-deadbeef-p0-normalize-1',
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
        state: 'exact',
        classification: 'observed',
        bytes: 1024,
        unattributed: [],
        reason: null,
      },
      dualAccountedSections: ['section:3(.data)'],
      excludedMetadataBytes: 210,
      regionUsage: [],
    },
    sectionCount: 12,
    symbolCount: 34,
    capabilities: {
      elf: 'supported',
      sections: 'available',
      symbols: 'available',
      debugInfo: 'available',
      map: 'provided',
      objectAttribution: 'unavailable',
      git: 'unknown',
    },
    evidenceSummary: {
      total: 9,
      observed: 6,
      derived: 2,
      declared: 1,
      unknown: 0,
    },
  };
  return { ...base, ...overrides };
}

/** The no-MAP shape: a degraded optional input, not a failure. */
function summaryWithoutMap(): AnalysisSummaryDto {
  return summary({
    artifact: { ...summary().artifact, fileName: 'basic.elf' },
    capabilities: { ...summary().capabilities, map: 'not-provided' },
    memory: {
      ...summary().memory,
      layoutSource: 'none',
      weakestEvidenceBasis: 'elf-address-and-flags',
      admissibleForHardBlock: false,
    },
  });
}

beforeEach(() => {
  selectMock.mockReset();
  attachMock.mockReset();
  clearMock.mockReset();
  analyzeMock.mockReset();

  selectMock.mockResolvedValue(ok(selection()));
  attachMock.mockResolvedValue(ok(WITH_MAP));
  clearMock.mockResolvedValue(ok(selection()));
  analyzeMock.mockResolvedValue(ok(summary()));
});

async function chooseArtifact(name = 'app.elf', language: 'en' | 'zh-CN' = 'en') {
  render(<App />, { language });
  fireEvent.click(await screen.findByRole('button', {
    name: language === 'en' ? 'Choose firmware artifact' : '选择固件镜像文件',
  }));
  await screen.findByText(name);
}

async function analyze(button = 'Analyze') {
  fireEvent.click(screen.getByRole('button', { name: button }));
}

describe('P1-A0 intake screen', () => {
  it('is titled Analyze and names no fixture', async () => {
    render(<App />);

    const heading = await screen.findByRole('heading', { level: 1 });
    expect(heading.textContent).toBe('Analyze');
    expect(screen.queryByText('P0 Technical Summary')).toBeNull();
    expect(screen.queryByLabelText('Fixture')).toBeNull();
    expect(document.body.textContent).not.toContain('fixture');
  });

  it('offers to choose an artifact and refuses to analyze before one exists', async () => {
    render(<App />);

    await screen.findByRole('button', { name: 'Choose firmware artifact' });
    const analyze = screen.getByRole('button', { name: 'Analyze' }) as HTMLButtonElement;
    expect(analyze.disabled).toBe(true);
    expect(screen.getByText(/Nothing has been analyzed/)).toBeDefined();
  });

  it('shows the chosen file name and enables Analyze', async () => {
    await chooseArtifact();

    expect(screen.getByText('app.elf')).toBeDefined();
    expect((screen.getByRole('button', { name: 'Analyze' }) as HTMLButtonElement).disabled).toBe(
      false,
    );
    expect(screen.getByText('MAP: Not provided')).toBeDefined();
  });

  it('treats a cancelled artifact dialog as a normal outcome, not an error', async () => {
    selectMock.mockResolvedValue(ok(null));

    render(<App />);
    fireEvent.click(await screen.findByRole('button', { name: 'Choose firmware artifact' }));

    await waitFor(() => expect(selectMock).toHaveBeenCalled());
    expect(screen.queryByRole('alert')).toBeNull();
    expect(screen.queryByRole('status')).toBeNull();
    // Still nothing selected, so the screen keeps offering the same first step.
    expect(screen.getByText(/Nothing has been analyzed/)).toBeDefined();
    expect((screen.getByRole('button', { name: 'Analyze' }) as HTMLButtonElement).disabled).toBe(
      true,
    );
  });

  it('asks the shell to analyze a selection id, never a path or a name', async () => {
    await chooseArtifact();
    await analyze();

    await waitFor(() => expect(analyzeMock).toHaveBeenCalledWith('sel-7f3a-2'));
    const sentArgs = JSON.stringify(analyzeMock.mock.lastCall ?? []);
    expect(sentArgs).not.toContain(MACHINE_PATH);
    expect(sentArgs.toLowerCase()).not.toContain('path');
  });

  it('never renders the directory the artifact came from', async () => {
    await chooseArtifact();
    await analyze();
    await screen.findByText('5,432 bytes');

    const text = document.body.textContent ?? '';
    expect(text).not.toContain(MACHINE_PATH);
    expect(text).not.toContain('secret-build');
    expect(text).not.toContain('sel-7f3a-2');
  });

  it('shows the core facts for a user-chosen file', async () => {
    await chooseArtifact();
    await analyze();

    expect(await screen.findByText('5,432 bytes')).toBeDefined();
    expect(screen.getByText('Arm 32-bit little')).toBeDefined();
    expect(screen.getByText('0x080003f8')).toBeDefined();
    expect(screen.getByText('4,200 bytes')).toBeDefined();
    expect(screen.getByText('1,024 bytes')).toBeDefined();
    expect(screen.getByText('section:3(.data)')).toBeDefined();
    expect(screen.getByText('snap-deadbeef-p0-normalize-1')).toBeDefined();
  });

  it('shows projected Region evidence as exact facts in both catalogs', async () => {
    const base = summary();
    const withRegions = {
      ...base,
      memory: {
        ...base.memory,
        regionUsage: [
          {
            name: 'ROM <script>alert(1)</script>',
            kind: 'nonvolatile',
            origin: '0x8000000',
            capacityBytes: '1048576',
            usedBytes: '160',
            remainingBytes: null,
            remainingReason: 'The region table does not prove contiguous allocatable space.',
            usedPercent: { value: '0.0%', status: 'available' },
            state: 'exact',
            risk: 'within-capacity',
            evidenceSource: 'map',
            evidenceId: 'ev-map-regions',
            addressKind: 'load',
            sections: [
              {
                locator: 'elf.section_header[3]',
                name: '.data',
                bytes: '4',
                address: '0x20000000',
                addressKind: 'virtual',
                virtualAddress: '0x20000000',
                virtualAddressUnknownReason: null,
                loadAddress: '0x800005c',
                loadAddressUnknownReason: null,
                evidenceBasis: 'map-region-and-elf-load',
              },
            ],
            reasons: [],
          },
        ],
      },
    } as AnalysisSummaryDto;
    analyzeMock.mockResolvedValue(ok(withRegions));

    await chooseArtifact();
    await analyze();

    const table = await screen.findByRole('table', { name: 'Memory regions' });
    expect(within(table).getByRole('columnheader', { name: 'Mapped ranges' })).toBeDefined();
    expect(within(table).getByText('1048576 bytes')).toBeDefined();
    expect(within(table).getByText('160 bytes')).toBeDefined();
    expect(within(table).getByText(/Unknown/)).toBeDefined();
    expect(within(table).queryByText(/160 bytes free/)).toBeNull();

    const details = within(table).getByText('Inspect sections');
    expect(details.tagName).toBe('SUMMARY');
    details.focus();
    fireEvent.keyDown(details, { key: 'Enter' });
    fireEvent.click(details);
    expect(details.closest('details')?.open).toBe(true);
    expect(await within(table).findByText(/LMA 0x800005c/)).toBeDefined();
    expect(within(table).getByRole('rowheader', {
      name: 'ROM <script>alert(1)</script>',
    })).toBeDefined();
    expect(table.querySelector('script')).toBeNull();

  });

  it('localizes Region headings in Chinese', async () => {
    const base = summary();
    const withRegions = {
      ...base,
      memory: {
        ...base.memory,
        regionUsage: [{
          name: 'ROM',
          kind: 'nonvolatile',
          origin: '0x8000000',
          capacityBytes: '1048576',
          usedBytes: null,
          remainingBytes: null,
          remainingReason: 'unknown',
          usedPercent: { value: null, status: 'usage_unknown' },
          state: 'unknown',
          risk: 'unknown',
          evidenceSource: 'none',
          evidenceId: 'region-rom',
          addressKind: 'load',
          sections: [],
          reasons: [],
        }],
      },
    } as AnalysisSummaryDto;
    analyzeMock.mockResolvedValue(ok({
      ...base,
      memory: withRegions.memory,
    } as AnalysisSummaryDto));

    render(
      <Analyze
        unit="bytes"
        onUnitChange={() => undefined}
        selection={null}
        onSelectionChange={() => undefined}
        analyzedSelectionId="sel-7f3a-2"
        lastGood={withRegions}
        gettingStartedHidden
        onHideGettingStarted={() => undefined}
        onLastGoodChange={() => undefined}
      />,
      { language: 'zh-CN' },
    );

    expect(screen.getByRole('table', { name: '内存区域' })).toBeDefined();
    expect(screen.getByText('声明容量')).toBeDefined();
  });

  it('keeps last-good Region evidence attached to its previous analysis', async () => {
    const previous = summary({
      artifact: { ...summary().artifact, fileName: 'old.elf' },
      memory: {
        ...summary().memory,
        regionUsage: [{
          name: 'ROM', kind: 'nonvolatile', origin: '0x8000000', capacityBytes: '4096',
          usedBytes: null, remainingBytes: null, remainingReason: 'unknown',
          usedPercent: { value: null, status: 'usage_unknown' }, state: 'unknown',
          risk: 'unknown', evidenceSource: 'map', evidenceId: 'old-region', addressKind: 'load',
          sections: [], reasons: ['old evidence'],
        }],
      },
    });
    render(
      <Analyze
        unit="bytes"
        onUnitChange={() => undefined}
        selection={{ selectionId: 'sel-new', fileName: 'new.elf', mapFileName: null, mapAttached: false }}
        onSelectionChange={() => undefined}
        analyzedSelectionId="sel-old"
        lastGood={previous}
        gettingStartedHidden
        onHideGettingStarted={() => undefined}
        onLastGoodChange={() => undefined}
      />,
    );

    const stale = screen.getByRole('region', { name: 'Last good analysis' });
    expect(within(stale).getByText(/Previous analysis of old\.elf/)).toBeDefined();
    expect(within(stale).getByRole('table', { name: 'Memory regions' })).toBeDefined();
    expect(within(stale).getByText('4096 bytes')).toBeDefined();
    expect(stale.textContent).not.toContain('new-region');
  });

  it('labels evidence quality with a word and an icon, never colour alone', async () => {
    await chooseArtifact();
    await analyze();

    expect(await screen.findByText('Can enforce a hard budget')).toBeDefined();
    expect(screen.getAllByText('Exact').length).toBe(2);
    expect(screen.getByText('Git')).toBeDefined();
  });

  it('names the based on in the words a reader needs, not the identifier behind it', async () => {
    // `weakestEvidenceBasis` is a portable wire value, and a wire value in a sentence is the shape L20
    // closed on Compare and F2 found again on Release. The shell still sends what it sends; this page
    // stops quoting it.
    await chooseArtifact();
    await analyze();

    expect(
      await screen.findByText(
        'Based on MAP regions + ELF load evidence; memory regions come from the linker.',
      ),
    ).toBeDefined();
    expect(document.body.textContent).not.toContain('map-memory-configuration+elf-load');
  });

  it('reports a missing MAP as an absent optional input, not a failure', async () => {
    selectMock.mockResolvedValue(
      ok(selection({ fileName: 'basic.elf', selectionId: 'sel-basic-1' })),
    );
    analyzeMock.mockResolvedValue(ok(summaryWithoutMap()));

    await chooseArtifact('basic.elf');
    await analyze();
    await screen.findByText('5,432 bytes');

    const row = screen.getByText('MAP').closest('div');
    expect(row?.textContent).toContain('not-provided');
    expect(row?.textContent).not.toContain('BLOCK');
    expect(screen.getByText('Not strong enough for a hard budget')).toBeDefined();
    expect(screen.getByText(/Add the linker MAP to strengthen it/)).toBeDefined();
  });

  it('attaches a MAP through the shell and names the file it holds', async () => {
    await chooseArtifact();

    fireEvent.click(screen.getByRole('button', { name: 'Add MAP' }));
    await waitFor(() => expect(attachMock).toHaveBeenCalledWith('sel-7f3a-2'));

    expect(await screen.findByText('MAP: app.map')).toBeDefined();
    expect(screen.getByRole('button', { name: 'Remove MAP' })).toBeDefined();
  });

  it('detaches a MAP without dropping the artifact selection', async () => {
    selectMock.mockResolvedValue(ok(WITH_MAP));
    clearMock.mockResolvedValue(ok(selection()));

    await chooseArtifact();
    expect(await screen.findByText('MAP: app.map')).toBeDefined();

    fireEvent.click(screen.getByRole('button', { name: 'Remove MAP' }));
    await waitFor(() => expect(clearMock).toHaveBeenCalledWith('sel-7f3a-2'));

    expect(await screen.findByText('MAP: Not provided')).toBeDefined();
    expect(screen.getByText('app.elf')).toBeDefined();
    expect((screen.getByRole('button', { name: 'Analyze' }) as HTMLButtonElement).disabled).toBe(
      false,
    );
  });

  it('treats a cancelled MAP dialog as a normal outcome', async () => {
    attachMock.mockResolvedValue(ok(null));

    await chooseArtifact();
    fireEvent.click(screen.getByRole('button', { name: 'Add MAP' }));

    await waitFor(() => expect(attachMock).toHaveBeenCalled());
    expect(screen.queryByRole('alert')).toBeNull();
    expect(screen.getByText('MAP: Not provided')).toBeDefined();
  });

  it('announces the in-flight state and locks every control until it resolves', async () => {
    let settle: ((outcome: IpcOutcome<AnalysisSummaryDto>) => void) | undefined;
    analyzeMock.mockImplementationOnce(
      () =>
        new Promise<IpcOutcome<AnalysisSummaryDto>>((resolve) => {
          settle = resolve;
        }),
    );

    await chooseArtifact();
    await analyze();

    const status = await screen.findByRole('status');
    expect(status.textContent).toContain('Analyzing');
    for (const name of ['Analyze', 'Choose firmware artifact', 'Add MAP']) {
      expect((screen.getByRole('button', { name }) as HTMLButtonElement).disabled).toBe(true);
    }

    settle?.(ok(summary()));
    await screen.findByText('5,432 bytes');
    // The in-flight announcement is gone. P1 added a second live region - the detail area reports
    // its own page loads - so this asserts the text that was promised to disappear, which is the
    // claim, instead of that no live region exists anywhere on the screen.
    expect(screen.queryByText('Analyzing…')).toBeNull();
    expect((screen.getByRole('button', { name: 'Analyze' }) as HTMLButtonElement).disabled).toBe(
      false,
    );
  });

  it('presents a typed error with what happened, code, why, what to do and a diagnostics id', async () => {
    await chooseArtifact();
    analyzeMock.mockResolvedValue(fail(PARSE_FAILURE));
    await analyze();

    const alert = await screen.findByRole('alert');
    expect(alert.textContent).toContain('Could not parse artifact as ELF.');
    expect(alert.textContent).toContain('ERR-PARSE-2002');
    expect(alert.textContent).toContain('truncated section table');
    expect(alert.textContent).toContain('Choose the linker ELF output');
    expect(alert.textContent).toContain('op-1a2b');
  });

  it('marks the surviving report as previous when a new artifact is chosen but not analyzed', async () => {
    await chooseArtifact();
    await analyze();
    await screen.findByText('5,432 bytes');
    expect(screen.getByRole('region', { name: 'Analysis summary' })).toBeDefined();

    // E2E-F001: the selection alone changes nothing below the rule, so the screen has to say whose
    // numbers they are before anyone presses Analyze.
    selectMock.mockResolvedValue(ok(selection({ fileName: 'other.elf', selectionId: 'sel-other' })));
    fireEvent.click(screen.getByRole('button', { name: 'Choose firmware artifact' }));
    await screen.findByText('other.elf');

    const previous = await screen.findByRole('region', { name: 'Last good analysis' });
    expect(within(previous).getByText(/Previous analysis of app\.elf/)).toBeDefined();
    expect(within(previous).getByText(/It is not an analysis of other\.elf/)).toBeDefined();
    // The last-good facts survive untouched, and none of them is presented as the candidate's.
    expect(within(previous).getByText('5,432 bytes')).toBeDefined();
    expect(within(previous).getByText('app.elf')).toBeDefined();
    // Choosing is not analyzing: no second snapshot is asked for.
    expect(analyzeMock).toHaveBeenCalledTimes(1);
  });

  it('marks the previous analysis when both artifacts share one leaf name', async () => {
    const base = summary();
    analyzeMock.mockResolvedValue(
      ok(
        summary({
          artifact: { ...base.artifact, fileName: 'firmware.elf', byteSize: 5432 },
        }),
      ),
    );
    selectMock.mockResolvedValue(ok(selection({ fileName: 'firmware.elf', selectionId: 'sel-a' })));
    await chooseArtifact('firmware.elf');
    await analyze();
    await screen.findByText('5,432 bytes');

    // The same file name from another directory. A name comparison cannot see this change, which is
    // why the identity behind the marker is the selection handle.
    selectMock.mockResolvedValue(ok(selection({ fileName: 'firmware.elf', selectionId: 'sel-b' })));
    fireEvent.click(screen.getByRole('button', { name: 'Choose firmware artifact' }));
    await screen.findByRole('region', { name: 'Last good analysis' });

    const previous = screen.getByRole('region', { name: 'Last good analysis' });
    expect(within(previous).getByText(/Previous analysis of firmware\.elf/)).toBeDefined();
    expect(
      within(previous).getByText(/different file with the same name, and it has not been analyzed yet/),
    ).toBeDefined();
    // The copy the finding is about: with one leaf name, naming the candidate would say
    // "not an analysis of firmware.elf" under "Previous analysis of firmware.elf" and explain nothing.
    expect(within(previous).queryByText(/It is not an analysis of firmware\.elf/)).toBeNull();
    expect(within(previous).getByText('5,432 bytes')).toBeDefined();

    // Analyzing the candidate replaces the report and retires the marker.
    analyzeMock.mockResolvedValue(
      ok(
        summary({
          artifact: { ...base.artifact, fileName: 'firmware.elf', byteSize: 6789, sha256: 'c'.repeat(64) },
          identity: { ...base.identity, snapshotId: 'snap-cafebabe-p0-normalize-1' },
        }),
      ),
    );
    fireEvent.click(screen.getByRole('button', { name: 'Analyze' }));

    const current = await screen.findByRole('region', { name: 'Analysis summary' });
    expect(within(current).getByText('6,789 bytes')).toBeDefined();
    expect(screen.queryByRole('region', { name: 'Last good analysis' })).toBeNull();
  });

  it('keeps the last good analysis visible when the current candidate fails', async () => {
    await chooseArtifact();
    await analyze();
    await screen.findByText('5,432 bytes');

    analyzeMock.mockResolvedValue(fail(PARSE_FAILURE));
    await analyze();

    const alert = await screen.findByRole('alert');
    expect(alert.textContent).toContain('ERR-PARSE-2002');

    // The previous result is still on screen - and still labelled as the previous result.
    const lastGood = await screen.findByRole('region', { name: 'Last good analysis' });
    expect(within(lastGood).getByText('5,432 bytes')).toBeDefined();
    expect(within(lastGood).getByText('app.elf')).toBeDefined();
    expect(screen.queryByRole('region', { name: 'Visual report export' })).toBeNull();
  });

  it('exports only the successful current selection', async () => {
    await chooseArtifact();
    await analyze();
    const exports = await screen.findByRole('region', { name: 'Visual report export' });
    fireEvent.click(within(exports).getByRole('button', { name: 'Export SVG' }));
    await waitFor(() => expect(exportAnalysisVisualMock).toHaveBeenCalledWith(
      'sel-7f3a-2',
      'snap-deadbeef-p0-normalize-1',
      'svg',
    ));
    selectMock.mockResolvedValue(ok(selection({ fileName: 'other.elf', selectionId: 'sel-other' })));
    fireEvent.click(screen.getByRole('button', { name: 'Choose firmware artifact' }));
    await screen.findByText('other.elf');
    expect(screen.queryByRole('region', { name: 'Visual report export' })).toBeNull();
  });

  it('does not attach a late export outcome to a different successful analysis', async () => {
    let finish!: (value: Awaited<ReturnType<typeof exportAnalysisVisual>>) => void;
    exportAnalysisVisualMock.mockReturnValueOnce(new Promise((resolve) => { finish = resolve; }));
    const subject = (id: string) => <Analyze
      unit="bytes" onUnitChange={() => undefined}
      selection={selection({ selectionId: id })} onSelectionChange={() => undefined}
      analyzedSelectionId={id}
      lastGood={summary({ identity: { ...summary().identity, snapshotId: id } })}
      gettingStartedHidden onHideGettingStarted={() => undefined}
      onLastGoodChange={() => undefined}
    />;
    const mounted = render(subject('old-subject'));
    fireEvent.click(screen.getByRole('button', { name: 'Export SVG' }));
    await waitFor(() => expect(exportAnalysisVisualMock).toHaveBeenCalled());
    mounted.rerender(subject('new-subject'));
    await act(async () => { finish(ok({ status: 'written', fileName: 'old-subject-report.svg', format: 'svg' })); });
    expect(screen.queryByText(/old-subject-report/)).toBeNull();
    expect((screen.getByRole('button', { name: 'Export SVG' }) as HTMLButtonElement).disabled).toBe(false);
  });

  it('never presents the last good analysis as the failed candidate', async () => {
    await chooseArtifact();
    await analyze();
    await screen.findByText('5,432 bytes');

    // A different file now fails; the surviving report must not be read as its result.
    selectMock.mockResolvedValue(ok(selection({ fileName: 'other.elf', selectionId: 'sel-other' })));
    fireEvent.click(screen.getByRole('button', { name: 'Choose firmware artifact' }));
    await screen.findByText('other.elf');

    analyzeMock.mockResolvedValue(fail(PARSE_FAILURE));
    fireEvent.click(screen.getByRole('button', { name: 'Analyze' }));

    const alert = await screen.findByRole('alert');
    expect(alert.textContent).toContain('ERR-PARSE-2002');

    const lastGood = await screen.findByRole('region', { name: 'Last good analysis' });
    expect(within(lastGood).getByText('app.elf')).toBeDefined();
    expect(within(lastGood).queryByText('other.elf')).toBeNull();
    expect(within(lastGood).getByText(/Previous analysis of app\.elf/)).toBeDefined();
  });

  it('replaces the last good analysis when a new candidate succeeds', async () => {
    await chooseArtifact();
    await analyze();
    await screen.findByText('5,432 bytes');

    selectMock.mockResolvedValue(
      ok(selection({ fileName: 'basic.elf', selectionId: 'sel-basic-2' })),
    );
    fireEvent.click(screen.getByRole('button', { name: 'Choose firmware artifact' }));
    await screen.findByText('basic.elf');

    analyzeMock.mockResolvedValue(ok(summaryWithoutMap()));
    fireEvent.click(screen.getByRole('button', { name: 'Analyze' }));

    const report = await screen.findByRole('region', { name: 'Analysis summary' });
    expect(within(report).getByText('basic.elf')).toBeDefined();
    expect(screen.queryByRole('region', { name: 'Last good analysis' })).toBeNull();
  });

  it('reports an unknown selection as a typed error and keeps the screen usable', async () => {
    await chooseArtifact();
    analyzeMock.mockResolvedValue(
      fail({
        code: 'ERR-INPUT-0001',
        message: 'That selection is no longer available.',
        operationId: 'op-stale',
        details: 'the shell holds paths for the current run only',
        remediation: 'Choose the artifact in the file dialog again.',
      }),
    );
    await analyze();

    const alert = await screen.findByRole('alert');
    expect(alert.textContent).toContain('That selection is no longer available.');
    expect(alert.textContent).toContain('Choose the artifact in the file dialog again');
    expect(
      (screen.getByRole('button', { name: 'Choose firmware artifact' }) as HTMLButtonElement).disabled,
    ).toBe(false);
    expect(screen.queryByText('5,432 bytes')).toBeNull();
  });

  it('says what a session that has done nothing yet has done: nothing', async () => {
    render(<App />);

    const empty = await screen.findByRole('region', { name: 'No analysis yet' });
    expect(empty.textContent).toContain('Nothing has been analyzed yet.');
    expect(screen.queryByRole('region', { name: 'Analysis summary' })).toBeNull();
  });

  it('offers no way into a stage this build does not have', async () => {
    render(<App />);
    await screen.findByRole('button', { name: 'Choose firmware artifact' });

    // P1 could claim there was nothing to navigate to. P2 made Compare a real page, P3 made Release
    // one, P5 made History one, and U1 adds Overview because the facts it summarizes are the facts the
    // shell already holds. The claim is still exactly as wide as the build: the rail lists the stages
    // that exist, and a stage that does not exist gets no entry, no link and none of its words.
    const rail = await screen.findByRole('navigation', { name: 'Pages' });
    expect(within(rail).getAllByRole('button').map((item) => item.textContent)).toEqual([
      'Overview',
      'Analyze',
      'Compare',
      'Release Gate',
      'Bundle & History',
    ]);
    // Help is a description of the product and not a stage of the workflow, so it is listed under its
    // own heading instead of after Release, where it would read as a fifth step (prompt §14).
    // B1-U1 adds Settings to the same group: it is a surface that exists today - two stored
    // preferences, read and written by the shell - and the rail still lists nothing that does not.
    const about = screen.getByRole('navigation', { name: 'Help and about' });
    expect(within(about).getAllByRole('button').map((item) => item.textContent)).toEqual([
      'Help',
      'Settings',
    ]);
    expect(document.querySelectorAll('a')).toHaveLength(0);

    fireEvent.click(within(rail).getByRole('button', { name: 'Compare page' }));
    await screen.findByRole('heading', { level: 1, name: 'Compare' });
    const body = document.body.textContent ?? '';
    // `Bundle` and `Gate` left this list in U1 for a reason worth stating: they were once words for
    // stages that did not exist, and now they name shipped surfaces - the Release Bundle section and
    // the rail's own "Release Gate" / "Bundle & History". A rail label is not an advertisement for
    // something unbuilt. The four words below still name nothing this product has.
    for (const stage of ['Settings', 'SBOM', 'Pricing', 'Cloud']) {
      expect(body).not.toMatch(new RegExp(`\\b${stage}\\b`));
    }
    // `Gate` is Release's word and appears on no other page. U1 scopes the check to the page rather than
    // the document, because the rail now names the stage "Release Gate" on every screen: the claim that
    // survives is that Compare itself says nothing about gate verdicts (prompt §43).
    const page = document.querySelector('main')?.textContent ?? '';
    expect(page).not.toMatch(/\bGate\b/);
  });
});

/**
 * U1R / U1-V2-06.
 *
 * The finding was not that a retained result is shown - that is deliberate, and the report below has
 * always said so. It is that the pills at the top of the page took their words from the retained
 * snapshot whenever one existed, so a failed attempt could read "ELF supported / MAP provided" in the
 * affirmative green of the previous build while the row underneath it said `MAP: Not provided`.
 *
 * Every assertion here is about an accessible name or a visible word, never about a pixel: the same
 * contract has to hold at 1440, 1056 and 1024, and a screenshot-derived test would only prove the
 * size it was written against.
 */
describe('U1R capability strip: current attempt, not retained evidence', () => {
  const strip = () => screen.getByRole('group', { name: 'Input capabilities' });

  /** The three words a successful analysis of the standard fixture earns. */
  async function expectCurrentChips() {
    const inputs = await screen.findByRole('group', { name: 'Input capabilities' });
    expect(within(inputs).getByText('ELF supported')).toBeDefined();
    expect(within(inputs).getByText('MAP provided')).toBeDefined();
    expect(within(inputs).getByText('Git unknown')).toBeDefined();
    expect(within(inputs).queryByText(/retained below/)).toBeNull();
  }

  function expectNoBorrowedChips(inputs: HTMLElement) {
    for (const word of ['ELF supported', 'MAP provided', 'Git unknown']) {
      expect(within(inputs).queryByText(word)).toBeNull();
    }
  }

  it('T1 states the current capability words after a successful analysis', async () => {
    await chooseArtifact();
    await analyze();
    await screen.findByText('5,432 bytes');

    await expectCurrentChips();
    // The header line belongs to the same attempt, so it names that artifact's figures.
    expect(screen.getByText('12 sections')).toBeDefined();
    expect(screen.getByText('34 symbols')).toBeDefined();
  });

  it('T2 withholds the previous words when the selection moves without an analysis', async () => {
    await chooseArtifact();
    await analyze();
    await screen.findByText('5,432 bytes');
    await expectCurrentChips();

    selectMock.mockResolvedValue(ok(selection({ fileName: 'other.elf', selectionId: 'sel-other' })));
    fireEvent.click(screen.getByRole('button', { name: 'Choose firmware artifact' }));
    await screen.findByText('other.elf');

    const inputs = strip();
    expectNoBorrowedChips(inputs);
    expect(within(inputs).getByText('other.elf selected, not analyzed yet')).toBeDefined();
    expect(within(inputs).getByText('Previous result retained below')).toBeDefined();
    // Choosing is not analyzing, and the header no longer quotes the retained snapshot.
    expect(screen.queryByText('12 sections')).toBeNull();
    expect(analyzeMock).toHaveBeenCalledTimes(1);
  });

  it('T3 withholds the previous words when the current attempt fails', async () => {
    await chooseArtifact();
    await analyze();
    await screen.findByText('5,432 bytes');
    await expectCurrentChips();

    analyzeMock.mockResolvedValue(fail(PARSE_FAILURE));
    await analyze();
    await screen.findByRole('alert');

    const inputs = strip();
    expectNoBorrowedChips(inputs);
    expect(within(inputs).getByText('Current analysis failed')).toBeDefined();
    expect(within(inputs).getByText('Previous result retained below')).toBeDefined();
    expect(screen.queryByText('12 sections')).toBeNull();
  });

  it('T4 keeps the retained analysis visible below, and keeps it labelled as retained', async () => {
    await chooseArtifact();
    await analyze();
    await screen.findByText('5,432 bytes');

    analyzeMock.mockResolvedValue(fail(PARSE_FAILURE));
    await analyze();
    await screen.findByRole('alert');

    const lastGood = await screen.findByRole('region', { name: 'Last good analysis' });
    expect(within(lastGood).getByText(/Previous analysis of app\.elf/)).toBeDefined();
    expect(within(lastGood).getByText('5,432 bytes')).toBeDefined();
    expect(within(lastGood).getByText('app.elf')).toBeDefined();
    // The tables under the report are the same retained snapshot, and say so in their own name.
    expect(
      screen.getByRole('region', { name: 'Snapshot details of the last good analysis' }),
    ).toBeDefined();
  });

  it('T5 still reports the failure that actually happened', async () => {
    await chooseArtifact();
    await analyze();
    await screen.findByText('5,432 bytes');

    analyzeMock.mockResolvedValue(fail(PARSE_FAILURE));
    await analyze();

    const alert = await screen.findByRole('alert');
    expect(alert.textContent).toContain('Analysis failed');
    expect(alert.textContent).toContain('Could not parse artifact as ELF.');
    expect(alert.textContent).toContain('ERR-PARSE-2002');
    expect(alert.textContent).toContain('truncated section table');
    expect(alert.textContent).toContain('op-1a2b');
  });

  it('T6 returns the current words when a later attempt succeeds', async () => {
    await chooseArtifact();
    await analyze();
    await screen.findByText('5,432 bytes');

    analyzeMock.mockResolvedValue(fail(PARSE_FAILURE));
    await analyze();
    await screen.findByRole('alert');
    expectNoBorrowedChips(strip());

    const base = summary();
    analyzeMock.mockResolvedValue(
      ok(
        summary({
          artifact: { ...base.artifact, byteSize: 6789, sha256: 'c'.repeat(64) },
          identity: { ...base.identity, snapshotId: 'snap-cafebabe-p0-normalize-1' },
        }),
      ),
    );
    await analyze();
    await screen.findByText('6,789 bytes');

    await expectCurrentChips();
    expect(screen.getByRole('region', { name: 'Analysis summary' })).toBeDefined();
    expect(screen.queryByRole('region', { name: 'Last good analysis' })).toBeNull();
    expect(screen.getByRole('region', { name: 'Snapshot details' })).toBeDefined();
    expect(screen.getByText('12 sections')).toBeDefined();
  });
});

/**
 * U1P §21. The hierarchy a reader meets, asserted as document order.
 *
 * These three tests are the only ones in the suite that say "before" rather than "present", because the
 * finding they answer was not that a fact was missing but that it arrived in the wrong order: the shell
 * scrolled its own navigation, Analyze opened with an identity dossier, and a failed attempt buried its
 * recovery action under the previous build's numbers. An ordering assertion is also the kind that survives
 * a window resize, which a coordinate assertion would not.
 */
describe('U1P page hierarchy', () => {
  it('keeps the navigation outside the page it does not scroll with', async () => {
    render(<App />);
    const rail = await screen.findByRole('navigation', { name: 'Pages' });

    // The rail is a sibling of the page, never a passenger inside it: `main` is the region that scrolls,
    // so anything living inside it travels with long content. §7A.
    const page = document.querySelector('main');
    expect(page).not.toBeNull();
    expect(page?.contains(rail)).toBe(false);
    expect(rail.closest('main')).toBeNull();
  });

  it('puts the result figures and the analysis tables above the identity dossier', async () => {
    await chooseArtifact();
    await analyze();

    const result = await screen.findByRole('region', { name: 'Analysis result' });
    const details = screen.getByRole('region', { name: 'Snapshot details' });
    const report = screen.getByRole('region', { name: 'Analysis summary' });

    expect(precedes(result, details)).toBe(true);
    expect(precedes(details, report)).toBe(true);
    // The dossier is still the dossier: moving it down did not remove the facts that make an analysis
    // checkable, which is the difference between secondary and gone.
    expect(within(report).getByText('SHA-256')).toBeDefined();
    expect(within(report).getByText('Parser')).toBeDefined();
  });

  it('keeps the failure and its recovery ahead of what the last attempt left behind', async () => {
    await chooseArtifact();
    await analyze();
    await screen.findByText('5,432 bytes');

    analyzeMock.mockResolvedValue(fail(PARSE_FAILURE));
    await analyze();
    const alert = await screen.findByRole('alert');

    // The recovery move is inside the failure, where the reader who needs it is looking (§13C), and the
    // retained result stays reachable underneath it without opening the page (§13D).
    expect(within(alert).getByRole('button', { name: 'Choose another artifact' })).toBeDefined();
    const retained = await screen.findByRole('region', { name: 'Last good analysis result' });
    expect(precedes(alert, retained)).toBe(true);
    expect(
      screen.getByRole('region', { name: 'Snapshot details of the last good analysis' }),
    ).toBeDefined();
  });
});


describe('M1 drag-and-drop intake', () => {
  const subscribeMock = vi.mocked(subscribeToDrops);
  const acceptMock = vi.mocked(acceptDroppedFiles);
  let emit: (event: DropEventDto) => void = () => undefined;

  function dropEvent(over: Partial<DropEventDto>): DropEventDto {
    return { phase: 'dropped', token: null, artifactName: null, mapName: null, refusal: null, ...over };
  }

  beforeEach(() => {
    acceptMock.mockReset();
    subscribeMock.mockImplementation((handler) => {
      emit = handler;
      return () => undefined;
    });
  });

  it('shows where a drop will land while files are over the window', async () => {
    render(<App />);
    await screen.findByRole('heading', { level: 1 });
    act(() => emit(dropEvent({ phase: 'hover' })));
    expect(screen.getByText('Drop an ELF, its MAP, or both to analyze')).toBeDefined();
    act(() => emit(dropEvent({ phase: 'leave' })));
    expect(screen.queryByText('Drop an ELF, its MAP, or both to analyze')).toBeNull();
  });

  it('stages a dropped ELF and MAP like the dialog and analyzes them once', async () => {
    acceptMock.mockResolvedValue(ok(WITH_MAP));
    render(<App />);
    await screen.findByRole('heading', { level: 1 });

    act(() => emit(dropEvent({ token: 'drop-1', artifactName: 'app.elf', mapName: 'app.map' })));

    await waitFor(() => expect(analyzeMock).toHaveBeenCalledTimes(1));
    expect(acceptMock).toHaveBeenCalledWith('drop-1', null);
    expect(analyzeMock).toHaveBeenCalledWith(WITH_MAP.selectionId);
    expect((await screen.findAllByText('app.elf')).length).toBeGreaterThan(0);
    expect(document.body.textContent).not.toContain(MACHINE_PATH);
  });

  it('attaches a lone MAP to the selection on screen', async () => {
    acceptMock.mockResolvedValue(ok(WITH_MAP));
    await chooseArtifact();
    act(() => emit(dropEvent({ token: 'drop-2', mapName: 'app.map' })));
    await waitFor(() => expect(acceptMock).toHaveBeenCalledWith('drop-2', selection().selectionId));
  });

  it('refuses a lone MAP when nothing is selected, without staging anything', async () => {
    render(<App />);
    await screen.findByRole('heading', { level: 1 });
    act(() => emit(dropEvent({ token: 'drop-3', mapName: 'app.map' })));
    expect(screen.getByRole('alert').textContent).toContain('A MAP needs a firmware file');
    expect(acceptMock).not.toHaveBeenCalled();
  });

  it('says why a drop was refused and can be dismissed', async () => {
    render(<App />);
    await screen.findByRole('heading', { level: 1 });
    act(() => emit(dropEvent({ phase: 'refused', refusal: 'directory' })));
    expect(screen.getByRole('alert').textContent).toContain('Folders can’t be dropped');
    fireEvent.click(screen.getByRole('button', { name: 'Dismiss' }));
    expect(screen.queryByRole('alert')).toBeNull();
  });

  it('keeps a staging failure on screen with its diagnostics id', async () => {
    acceptMock.mockResolvedValue(fail(PARSE_FAILURE));
    render(<App />);
    await screen.findByRole('heading', { level: 1 });
    act(() => emit(dropEvent({ token: 'drop-4', artifactName: 'bad.elf' })));
    expect(await screen.findByText('The dropped files could not be opened')).toBeDefined();
    expect(analyzeMock).not.toHaveBeenCalled();
  });
});
