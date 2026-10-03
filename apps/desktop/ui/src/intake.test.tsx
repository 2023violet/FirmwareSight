import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { App } from './App';
import { analyzeSelection, attachMap, clearMap, selectArtifact } from './ipc/bridge';
import type { IpcOutcome } from './ipc/bridge';
import type { AnalysisSummaryDto, ErrorEnvelopeDto, SelectionDto } from './ipc/types';

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
    },
    sectionCount: 12,
    symbolCount: 34,
    capabilities: {
      elf: 'supported',
      sections: 'available',
      symbols: 'available',
      debugInfo: 'available',
      map: 'provided',
      objectAttribution: 'available',
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

async function chooseArtifact(name = 'app.elf') {
  render(<App />);
  fireEvent.click(await screen.findByRole('button', { name: 'Choose firmware artifact' }));
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

  it('labels evidence quality with a word and an icon, never colour alone', async () => {
    await chooseArtifact();
    await analyze();

    expect(await screen.findByText('Admissible for a hard limit')).toBeDefined();
    expect(screen.getAllByText('Exact').length).toBe(2);
    expect(screen.getByText('Git')).toBeDefined();
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
    expect(screen.getByText('Not admissible for a hard limit')).toBeDefined();
    expect(screen.getByText(/Supply the linker MAP to strengthen it/)).toBeDefined();
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
    expect(empty.textContent).toContain('Nothing has been analyzed in this session yet.');
    expect(screen.queryByRole('region', { name: 'Analysis summary' })).toBeNull();
  });

  it('offers no way into a stage this build does not have', async () => {
    render(<App />);
    await screen.findByRole('button', { name: 'Choose firmware artifact' });

    // P1 could claim there was nothing to navigate to. P2 made Compare a real page, P3 made Release
    // one, and P5 makes History one because the rows it shows were already being stored. The claim is
    // still exactly as wide as the build: the rail lists the stages that exist, and a stage that does
    // not exist gets no entry, no link and none of its words.
    const rail = await screen.findByRole('navigation', { name: 'Pages' });
    expect(within(rail).getAllByRole('button').map((item) => item.textContent)).toEqual([
      'Analyze',
      'Compare',
      'Release',
      'History',
    ]);
    // Help is a description of the product and not a stage of the workflow, so it is listed under its
    // own heading instead of after Release, where it would read as a fifth step (prompt §14).
    const about = screen.getByRole('navigation', { name: 'Help and about' });
    expect(within(about).getAllByRole('button').map((item) => item.textContent)).toEqual(['Help']);
    expect(document.querySelectorAll('a')).toHaveLength(0);

    fireEvent.click(within(rail).getByRole('button', { name: 'Compare page' }));
    await screen.findByRole('heading', { level: 1, name: 'Compare' });
    const body = document.body.textContent ?? '';
    for (const stage of ['Bundle', 'Settings', 'SBOM', 'Pricing', 'Cloud']) {
      expect(body).not.toMatch(new RegExp(`\\b${stage}\\b`));
    }
    // `Gate` is Release's word and appears on no other page: a rail entry names a stage, it does not
    // advertise one that has not been built (prompt §43).
    expect(body).not.toMatch(/\bGate\b/);
  });
});
