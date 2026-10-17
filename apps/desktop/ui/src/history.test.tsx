/**
 * P5's History page: three bounded tables over rows the application already wrote.
 *
 * The claims this file exists to protect are the ones that separate a history *viewer* from a second
 * copy of the product:
 *
 * - every row, page and filter answer comes from the shell. The screen never holds a list it sliced
 *   itself, because prompt §17 puts the stored population behind a bounded read in Rust, and a filter
 *   that narrowed only the page on screen would silently lie about the size of the store;
 * - one table's state belongs to that table: the builds filter cannot leak into the Gate list, and a
 *   slow reply cannot paint itself under a newer question (prompt §17, `04_TECH/14` 3);
 * - a stored disposition and its finding counts are shown as stored, and a word the reader cannot map
 *   stays unknown instead of being promoted to a verdict (prompt §16, DESIGN.md 5);
 * - nothing here can change a stored row: the page asks for three reads and offers no write, no delete
 *   and no re-run (prompt §17);
 * - a shortened id is a convenience, so opening a row has to give the whole text back, and no host
 *   path may appear anywhere on the page (prompt §16).
 *
 * The bridge is mocked the way the other page tests mock it; the plumbing has its own file.
 */

import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { App } from './App';
import { clearFallbacks, fallbacks } from './i18n';
import {
  compareSnapshots,
  getApplicationPreferences,
  getBaselinePin,
  listCompareCandidates,
  listHistoryBuilds,
  listHistoryGateRuns,
  listHistoryReleases,
  pinBaseline,
  setWindowTitle,
  unpinBaseline,
} from './ipc/bridge';
import type { IpcOutcome } from './ipc/bridge';
import type {
  CandidatePageDto,
  CompareCandidateDto,
  ErrorEnvelopeDto,
  HistoryGateRunPageDto,
  HistoryGateRunRowDto,
  HistoryPageRequestDto,
  HistoryReleasePageDto,
  HistoryReleaseRowDto,
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
  // R5: a comparison started from History opens Compare, which reads its candidates and runs once.
  listCompareCandidates: vi.fn(),
  compareSnapshots: vi.fn(),
  querySectionChanges: vi.fn(),
  querySymbolChanges: vi.fn(),
}));

const buildsMock = vi.mocked(listHistoryBuilds);
const runsMock = vi.mocked(listHistoryGateRuns);
const releasesMock = vi.mocked(listHistoryReleases);
const titleMock = vi.mocked(setWindowTitle);

function ok<T>(value: T): IpcOutcome<T> {
  return { ok: true, value };
}

function fail<T>(envelope: ErrorEnvelopeDto): IpcOutcome<T> {
  return { ok: false, envelope };
}

/**
 * A node's text with the source's line breaks folded back into single spaces.
 *
 * A sentence the component wraps over three lines in its JSX is one sentence on the screen, and an
 * assertion that had to match the wrapping would break on a reformat rather than on a changed claim.
 */
function prose(node: Element | null): string {
  return (node?.textContent ?? '').replace(/\s+/g, ' ');
}

const BUILD_ID = `build-${'b'.repeat(40)}`;
const SNAPSHOT_ID = `snap-${'s'.repeat(40)}`;
const SHA256 = 'a'.repeat(64);
const RUN_ID = `gate-${'r'.repeat(40)}`;
const RELEASE_ID = `rel-${'e'.repeat(40)}`;
const MANIFEST_SHA = 'c'.repeat(64);
const BASELINE_ID = `build-${'f'.repeat(40)}`;

/**
 * The three tables carry distinct file names on purpose: several tests ask the whole document whether
 * a row is on screen, and a name two tables share would answer a question nobody asked.
 */
const BUILD_FILE = 'app.elf';
const GATE_FILE = 'judged.elf';
const RELEASE_FILE = 'bundled.elf';

/** A stored build: name and digest only, because where the artifact sat is an intake fact. */
function candidate(overrides: Partial<CompareCandidateDto> = {}): CompareCandidateDto {
  const base: CompareCandidateDto = {
    buildId: BUILD_ID,
    snapshotId: SNAPSHOT_ID,
    fileName: BUILD_FILE,
    sha256: SHA256,
    byteSize: 5432,
    architecture: 'armv7em-none-eabihf',
    importedAt: '2026-09-30T08:12:04Z',
    nonvolatile: { state: 'exact', bytes: 4096 },
    runtimeRam: { state: 'partial', bytes: 1024 },
  };
  return { ...base, ...overrides };
}

function gateRow(overrides: Partial<HistoryGateRunRowDto> = {}): HistoryGateRunRowDto {
  const base: HistoryGateRunRowDto = {
    runId: RUN_ID,
    buildId: BUILD_ID,
    fileName: GATE_FILE,
    baselineBuildId: BASELINE_ID,
    baselineFileName: 'boot.elf',
    disposition: 'REVIEW',
    counts: { pass: 6, review: 1, block: 0, unknown: 2, notApplicable: 1 },
    storedAt: '2026-10-01T09:40:11Z',
  };
  return { ...base, ...overrides };
}

function releaseRow(overrides: Partial<HistoryReleaseRowDto> = {}): HistoryReleaseRowDto {
  const base: HistoryReleaseRowDto = {
    releaseId: RELEASE_ID,
    releaseVersion: '2.1.0',
    buildId: BUILD_ID,
    fileName: RELEASE_FILE,
    gateRunId: RUN_ID,
    manifestSha256: MANIFEST_SHA,
    storedAt: '2026-10-02T07:05:33Z',
  };
  return { ...base, ...overrides };
}

function buildPage(
  rows: readonly CompareCandidateDto[],
  overrides: Partial<CandidatePageDto> = {},
): IpcOutcome<CandidatePageDto> {
  return ok({
    rows: [...rows],
    total: rows.length,
    offset: 0,
    limit: 25,
    nextOffset: null,
    ...overrides,
  });
}

function runPage(
  rows: readonly HistoryGateRunRowDto[],
  overrides: Partial<HistoryGateRunPageDto> = {},
): IpcOutcome<HistoryGateRunPageDto> {
  return ok({
    rows: [...rows],
    total: rows.length,
    offset: 0,
    limit: 25,
    nextOffset: null,
    ...overrides,
  });
}

function releasePage(
  rows: readonly HistoryReleaseRowDto[],
  overrides: Partial<HistoryReleasePageDto> = {},
): IpcOutcome<HistoryReleasePageDto> {
  return ok({
    rows: [...rows],
    total: rows.length,
    offset: 0,
    limit: 25,
    nextOffset: null,
    ...overrides,
  });
}

const READ_REFUSED: ErrorEnvelopeDto = {
  code: 'ERR-STORAGE-4009',
  message: 'The stored Gate runs could not be read.',
  operationId: 'op-hist-7',
  details: 'the store is busy',
  remediation: 'Try again after the running operation finishes.',
};

/** A read the test finishes by hand, so an arrival order can be forced rather than wished for. */
function deferred<T>() {
  let resolve!: (outcome: IpcOutcome<T>) => void;
  const promise = new Promise<IpcOutcome<T>>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}

async function openHistory(): Promise<void> {
  render(<App />);
  fireEvent.click(await screen.findByRole('button', { name: 'History page' }));
  await screen.findByRole('heading', { level: 1, name: 'History' });
  await screen.findByText(BUILD_FILE);
}

/**
 * U1P §12 shows one stored entity at a time, so a test that means to read a second table says which one
 * it is looking at, exactly as a reader does. The switch is a button with `aria-pressed`, not a link and
 * not a tab, so this is the whole of what reaching a table costs.
 */
async function showView(label: 'Builds' | 'Gate runs' | 'Release records'): Promise<void> {
  // R5: the three lists are Tabs.
  fireEvent.click(screen.getByRole('tab', { name: label }));
}

/** The builds table, once its first page has painted. */
async function buildsRegion(): Promise<HTMLElement> {
  await showView('Builds');
  await screen.findByText(BUILD_FILE);
  return screen.getByRole('region', { name: 'Stored builds' });
}

async function runsRegion(): Promise<HTMLElement> {
  await showView('Gate runs');
  await screen.findByText(GATE_FILE);
  return screen.getByRole('region', { name: 'Stored Gate runs' });
}

async function releasesRegion(): Promise<HTMLElement> {
  await showView('Release records');
  await screen.findByText(RELEASE_FILE);
  return screen.getByRole('region', { name: 'Stored release records' });
}

async function filterForm(noun: string): Promise<HTMLElement> {
  return await screen.findByRole('form', { name: `Filter ${noun}` });
}

/** The row action that opens a stored row in the Inspector (R5: replaces the expanding Details row). */
async function rowAction(region: HTMLElement): Promise<HTMLElement> {
  return (await within(region).findAllByRole('button', { name: /^Show the stored record of / }))[0] as HTMLElement;
}

/** Open one table's first row in the Inspector and hand back the control that did it. */
async function openFirstRow(region: HTMLElement): Promise<HTMLElement> {
  const action = await rowAction(region);
  fireEvent.click(action);
  await screen.findByRole('complementary', { name: 'Evidence' });
  return action;
}

/**
 * The cell of a row that sits under a named column.
 *
 * A stored row is read by its columns, so an assertion says which column it means instead of counting
 * positions — the row action leads its row precisely so that it cannot be clipped, and a count would
 * make that fix look like a breakage.
 */
function cellOf(row: HTMLElement, header: string): HTMLElement {
  const table = row.closest('table');
  if (table === null) {
    throw new Error(`a ${header} cell needs a table`);
  }
  const headers = within(table as HTMLElement).getAllByRole('columnheader');
  const index = headers.findIndex((cell) => (cell.textContent ?? '').trim() === header);
  if (index < 0) {
    throw new Error(`no column is named ${header}`);
  }
  // Row headers and cells together, in column order (the row's own name is a `th`).
  const cell = Array.from(row.querySelectorAll('th,td'))[index];
  if (cell === undefined) {
    throw new Error(`${header} has no cell in this row`);
  }
  return cell as HTMLElement;
}

beforeEach(() => {
  buildsMock.mockReset();
  runsMock.mockReset();
  releasesMock.mockReset();
  titleMock.mockReset();

  buildsMock.mockResolvedValue(buildPage([candidate()]));
  runsMock.mockResolvedValue(runPage([gateRow()]));
  releasesMock.mockResolvedValue(releasePage([releaseRow()]));
  titleMock.mockResolvedValue(ok(null));
});

describe('History states what a row is', () => {
  it('says these are stored facts and not the artifact now open', async () => {
    await openHistory();

    // The whole header, not the heading's own box: `PageHeader` puts the title and the metadata line in
    // one block and the subhead beside it, so the sentence a reader sees under the title lives one level
    // up from the `h1`.
    const header = (await screen.findByRole('heading', { level: 1, name: 'History' })).closest(
      'header',
    );
    const words = prose(header ?? null);
    expect(words).toContain('Stored builds, Gate runs and releases');
    expect(words).toContain('not the file open on Analyze');
    expect(words).toContain('Rows outlive moved files');
    expect(words).toContain('nothing here edits them');
  });

  it('lists the three stored populations the shell returned', async () => {
    await openHistory();

    const builds = await buildsRegion();
    await waitFor(() => expect(within(builds).getAllByRole('row')).toHaveLength(2));
    expect(within(builds).getByRole('columnheader', { name: 'Artifact' })).toBeDefined();
    expect(within(builds).getByText(BUILD_FILE)).toBeDefined();

    const runs = await runsRegion();
    await waitFor(() => expect(within(runs).getAllByRole('row')).toHaveLength(2));
    expect(within(runs).getByRole('columnheader', { name: 'Disposition' })).toBeDefined();

    const releases = await releasesRegion();
    await waitFor(() => expect(within(releases).getAllByRole('row')).toHaveLength(2));
    expect(within(releases).getByText('2.1.0')).toBeDefined();
  });

  it('asks the shell for the rows instead of slicing a list it holds', async () => {
    await openHistory();

    // One request per table, and the request names an offset and a filter and nothing else. No limit:
    // the ceiling is Rust's, so asking from here cannot widen it (prompt §17).
    const wanted: HistoryPageRequestDto = { offset: 0, limit: null, filter: null };
    expect(buildsMock.mock.calls.map((call) => call[0])).toEqual([wanted]);
    expect(runsMock.mock.calls.map((call) => call[0])).toEqual([wanted]);
    expect(releasesMock.mock.calls.map((call) => call[0])).toEqual([wanted]);
  });

  it('lets the shell name the window, because a page may not write its own title', async () => {
    await openHistory();

    expect(titleMock.mock.calls.map((call) => call[0])).toContain('History');
  });
});

describe('History filters and pages through the shell', () => {
  it('sends a filter as a request and repaints from what comes back', async () => {
    await openHistory();

    buildsMock.mockResolvedValueOnce(
      buildPage([candidate({ fileName: 'bootloader.elf', buildId: BASELINE_ID })], { total: 1 }),
    );

    const form = await filterForm('builds');
    fireEvent.change(within(form).getByRole('searchbox'), { target: { value: 'bootloader' } });
    fireEvent.click(within(form).getByRole('button', { name: 'Apply' }));

    await screen.findByText('bootloader.elf');
    expect(buildsMock).toHaveBeenCalledTimes(2);
    expect(buildsMock.mock.calls[1]?.[0]).toEqual({ offset: 0, limit: null, filter: 'bootloader' });
    // A filter narrows the stored population, so the row it excluded leaves the screen.
    expect(screen.queryByText(BUILD_FILE)).toBeNull();
  });

  it('keeps one table\u2019s filter its own', async () => {
    await openHistory();

    const form = await filterForm('builds');
    fireEvent.change(within(form).getByRole('searchbox'), { target: { value: 'app' } });
    fireEvent.click(within(form).getByRole('button', { name: 'Apply' }));
    await waitFor(() => expect(buildsMock).toHaveBeenCalledTimes(2));

    // The Gate and release reads still answer the question they were asked, so they were not asked again.
    expect(runsMock).toHaveBeenCalledTimes(1);
    expect(releasesMock).toHaveBeenCalledTimes(1);
    expect(runsMock.mock.calls[0]?.[0]).toEqual({ offset: 0, limit: null, filter: null });
  });

  it('does not filter on a keystroke', async () => {
    await openHistory();
    await showView('Gate runs');

    const form = await filterForm('Gate runs');
    fireEvent.change(within(form).getByRole('searchbox'), { target: { value: 'gate' } });

    // A half-typed word must not narrow a table the reader was still reading.
    expect(runsMock).toHaveBeenCalledTimes(1);

    fireEvent.click(within(form).getByRole('button', { name: 'Apply' }));
    await waitFor(() => expect(runsMock).toHaveBeenCalledTimes(2));
    expect(runsMock.mock.calls[1]?.[0]).toEqual({ offset: 0, limit: null, filter: 'gate' });
  });

  it('returns to the first page when a filter is applied from a later one', async () => {
    buildsMock.mockResolvedValue(buildPage([candidate()], { total: 40, limit: 25, nextOffset: 25 }));
    await openHistory();

    const builds = await buildsRegion();
    fireEvent.click(within(builds).getByRole('button', { name: 'Next page' }));
    await waitFor(() =>
      expect(buildsMock.mock.calls[1]?.[0]).toEqual({ offset: 25, limit: null, filter: null }),
    );

    const form = await filterForm('builds');
    fireEvent.change(within(form).getByRole('searchbox'), { target: { value: 'app' } });
    fireEvent.click(within(form).getByRole('button', { name: 'Apply' }));

    // Page 25 of a filtered population is not what "filter this" asks for (prompt §17).
    await waitFor(() => expect(buildsMock).toHaveBeenCalledTimes(3));
    expect(buildsMock.mock.calls[2]?.[0]).toEqual({ offset: 0, limit: null, filter: 'app' });
  });

  it('pages with the offset the shell named and never repeats a row', async () => {
    buildsMock.mockResolvedValue(buildPage([candidate()], { total: 2, limit: 1, nextOffset: 1 }));
    await openHistory();

    const builds = await buildsRegion();
    expect(within(builds).getByText('Showing 1 to 1 of 2 builds')).toBeDefined();
    expect(
      (within(builds).getByRole('button', { name: 'Previous page' }) as HTMLButtonElement).disabled,
    ).toBe(true);

    buildsMock.mockResolvedValueOnce(
      buildPage(
        [candidate({ buildId: BASELINE_ID, snapshotId: `snap-${'q'.repeat(40)}`, fileName: 'next.elf' })],
        { total: 2, offset: 1, limit: 1 },
      ),
    );
    fireEvent.click(within(builds).getByRole('button', { name: 'Next page' }));
    await within(builds).findByText('next.elf');

    expect(buildsMock.mock.calls[1]?.[0]).toEqual({ offset: 1, limit: null, filter: null });
    expect(within(builds).queryByText(BUILD_FILE)).toBeNull();
    expect(within(builds).getByText('Showing 2 to 2 of 2 builds')).toBeDefined();
    expect(
      (within(builds).getByRole('button', { name: 'Next page' }) as HTMLButtonElement).disabled,
    ).toBe(true);
  });

  it('paints only the newest answer when an older reply arrives last', async () => {
    await openHistory();

    const late = deferred<CandidatePageDto>();
    const newer = deferred<CandidatePageDto>();
    buildsMock.mockReturnValueOnce(late.promise);
    buildsMock.mockReturnValueOnce(newer.promise);

    const form = await filterForm('builds');
    const box = within(form).getByRole('searchbox');
    fireEvent.change(box, { target: { value: 'one' } });
    fireEvent.click(within(form).getByRole('button', { name: 'Apply' }));
    fireEvent.change(box, { target: { value: 'two' } });
    fireEvent.click(within(form).getByRole('button', { name: 'Apply' }));

    newer.resolve(
      buildPage([candidate({ fileName: 'newer.elf', buildId: BASELINE_ID })], { total: 1 }),
    );
    await screen.findByText('newer.elf');

    // The superseded answer now arrives, and the screen must not go back to it.
    late.resolve(buildPage([candidate({ fileName: 'stale.elf' })], { total: 1 }));
    await act(async () => {
      await Promise.resolve();
    });
    expect(screen.queryByText('stale.elf')).toBeNull();
    expect(screen.getByText('newer.elf')).toBeDefined();
  });

  it('changes only the numbers when the unit moves, and asks the shell for nothing', async () => {
    await openHistory();
    const builds = await buildsRegion();
    await openFirstRow(builds);
    expect(screen.getByText('5,432 bytes')).toBeDefined();

    fireEvent.click(within(screen.getByRole('group', { name: 'Size units' })).getByLabelText('KiB'));

    // US-001 addendum 3: a unit is a way of writing one number, so no read runs and the figure does
    // not move - only its rendering. R5: the Inspector closes on a unit change (its text was written in
    // the old unit, the R3 rule), and the same row reopened reads in the new one.
    await waitFor(() => expect(screen.queryByRole('complementary', { name: 'Evidence' })).toBeNull());
    await openFirstRow(builds);
    expect(screen.getByText('5.30 KiB')).toBeDefined();
    expect(screen.queryByText('5,432 bytes')).toBeNull();
    expect(buildsMock).toHaveBeenCalledTimes(1);
    expect(runsMock).toHaveBeenCalledTimes(1);
    expect(releasesMock).toHaveBeenCalledTimes(1);
  });
});

describe('History shows stored verdicts as stored', () => {
  async function firstGateRow(): Promise<HTMLElement> {
    const runs = await runsRegion();
    return (await rowAction(runs)).closest('tr') as HTMLElement;
  }

  it('gives the disposition a word and an icon, not a colour', async () => {
    await openHistory();
    const row = await firstGateRow();

    expect(row.textContent).toContain('Needs review');
    expect(row.querySelector('svg')).not.toBeNull();
  });

  it('shows every state count, including the ones that are zero', async () => {
    await openHistory();
    const row = await firstGateRow();

    // A zero finding is a fact about the run, so it stays on the screen (DESIGN.md 5).
    for (const [label, value] of [
      ['Pass', 6],
      ['Review', 1],
      ['Block', 0],
      ['Unknown', 2],
      ['Not applicable', 1],
    ] as const) {
      expect(row.textContent).toContain(`${label}${String(value)}`);
    }
  });

  it('calls an unmappable stored word unknown instead of a verdict', async () => {
    runsMock.mockResolvedValue(runPage([gateRow({ disposition: 'MIGRATED' })]));
    await openHistory();

    expect((await firstGateRow()).textContent).toContain('Unknown disposition');
  });

  it('names a run with no baseline as none, and never invents one', async () => {
    runsMock.mockResolvedValue(runPage([gateRow({ baselineBuildId: null, baselineFileName: null })]));
    await openHistory();

    const row = await firstGateRow();
    // Seven columns (the row action and six stored ones), and the baseline is one of them by name rather
    // than by count: no column is dropped to make the row fit, which is what §6 forbids.
    expect(row.querySelectorAll('th,td')).toHaveLength(7);
    expect(cellOf(row, 'Baseline').textContent).toBe('None');
  });

  it('leaves an unrecorded budget unknown through a unit change', async () => {
    buildsMock.mockResolvedValue(
      buildPage([candidate({ nonvolatile: { state: 'unknown', bytes: null } })]),
    );
    await openHistory();
    const builds = await buildsRegion();

    const row = within(builds).getAllByRole('row')[1] as HTMLElement;
    expect(row.textContent).toContain('Unknown');
    // `Unknown` is not a zero, and a unit change must not turn it into one.
    expect(row.textContent).not.toMatch(/\b0 (bytes|KiB)\b/);

    fireEvent.click(within(screen.getByRole('group', { name: 'Size units' })).getByLabelText('KiB'));
    expect(row.textContent).toContain('Unknown');
    expect(row.textContent).not.toMatch(/\b0 (bytes|KiB)\b/);
  });
});

describe('the row action stays reachable', () => {
  // F2R-02: `Details` was clipped at the default installed window width in all three table families,
  // because a stored row of ids, digests and times is wider than the frozen 1024 minimum and the action
  // was the last thing on it. The position is the contract; the pixel proof is the installed run.
  it('leads each stored row with its action, and its header with the same column', async () => {
    await openHistory();

    for (const region of [await buildsRegion(), await runsRegion(), await releasesRegion()]) {
      const header = within(region).getAllByRole('row')[0] as HTMLElement;
      expect((within(header).getAllByRole('columnheader')[0]?.textContent ?? '').trim()).toBe(
        'Row actions',
      );
      const row = within(region).getAllByRole('row')[1] as HTMLElement;
      const first = row.querySelector('th,td') as HTMLElement;
      expect(within(first).getByRole('button', { name: /^Show the stored record of / })).toBeDefined();
    }
  });

  it('gives the scroll axis to a wrapper around each stored table', async () => {
    // The same mechanism F2R-01 removed from the detail tables: a table pinned to the width of its pane
    // gets laid out below its own minimum, and the column that gives way is the one that was readable.
    // The wrapper is what lets the table keep its intrinsic width, so the tree says which element owns
    // the scroll.
    await openHistory();

    for (const region of [await buildsRegion(), await runsRegion(), await releasesRegion()]) {
      const table = within(region).getByRole('table');
      // R5: the shared DataTable's wrapper owns the scroll axis.
      expect(table.parentElement?.className).toContain('wrap');
      expect(table.tagName).toBe('TABLE');
    }
  });
});

describe('History opens a row without leaving the table', () => {
  it('gives the whole digest back when the row is opened, and takes it away again', async () => {
    await openHistory();
    const builds = await buildsRegion();

    // A shortened id is a convenience, so before the row is opened the screen holds the shortened text
    // and none of the full one.
    expect(screen.queryByText(SHA256)).toBeNull();

    const action = await openFirstRow(builds);
    const inspector = screen.getByRole('complementary', { name: 'Evidence' });
    expect(within(inspector).getByText(SHA256)).toBeDefined();
    expect(within(inspector).getByText(BUILD_ID)).toBeDefined();
    expect(within(inspector).getByText(SNAPSHOT_ID)).toBeDefined();
    expect(inspector.textContent).toContain('FirmwareSight store (read-only)');

    fireEvent.click(within(inspector).getByRole('button', { name: 'Close evidence panel' }));
    await waitFor(() => expect(screen.queryByText(SHA256)).toBeNull());
    expect(document.activeElement).toBe(action);
  });

  it('opens the ids a stored Gate run and a release record are cited by', async () => {
    await openHistory();

    const runs = await runsRegion();
    await openFirstRow(runs);
    await screen.findByText(RUN_ID);
    expect(screen.getByText(BASELINE_ID)).toBeDefined();

    const releases = await releasesRegion();
    await openFirstRow(releases);
    await screen.findByText(RELEASE_ID);
    expect(screen.getByText(MANIFEST_SHA)).toBeDefined();
    expect(screen.getByText(BUILD_ID)).toBeDefined();
  });

  it('reaches a row action with the keyboard, because it is a button and not a click target', async () => {
    await openHistory();
    const builds = await buildsRegion();
    const details = await rowAction(builds);

    expect(details.tagName).toBe('BUTTON');
    expect(details.getAttribute('href')).toBeNull();
    expect(details.getAttribute('tabindex')).toBeNull();
    details.focus();
    expect(document.activeElement).toBe(details);

    fireEvent.click(details);
    expect(await screen.findByText(SHA256)).toBeDefined();
  });
});

describe('History has no way to change a stored row', () => {
  it('offers no write, delete or re-run control and no link out', async () => {
    await openHistory();

    const page = screen.getByRole('main');
    for (const verb of [
      /delete/i,
      /remove/i,
      /edit/i,
      /rename/i,
      /clear/i,
      /re-?run/i,
      /run gate/i,
      /accept review/i,
      /export/i,
      /save/i,
    ]) {
      expect(within(page).queryAllByRole('button', { name: verb })).toEqual([]);
    }
    expect(page.querySelectorAll('a')).toHaveLength(0);
    // The only field on the page is the active view's bounded filter. U1P §12 shows one stored entity at
    // a time, so the three filters that used to sit on screen together are now one per view.
    expect(within(page).getAllByRole('searchbox')).toHaveLength(1);
    // R5: the one write is the pinned baseline, which appends to its own log and changes no stored row.
  });

  it('shows no host path anywhere on the page', async () => {
    await openHistory();
    const builds = await buildsRegion();
    await openFirstRow(builds);

    const text = screen.getByRole('main').textContent ?? '';
    expect(text).not.toMatch(/[A-Za-z]:[\\/]/);
    expect(text).not.toMatch(/(^|[\s(])\\\\[\w.-]+\\/);
    expect(text).not.toMatch(/\/(Users|home|Volumes|mnt)\//);
    expect(text).not.toContain('fixtures');
  });
});

describe('History when a read has nothing to show or cannot show it', () => {
  it('distinguishes a store that is empty from a filter that matched nothing', async () => {
    buildsMock.mockResolvedValue(buildPage([]));
    render(<App />);
    fireEvent.click(await screen.findByRole('button', { name: 'History page' }));

    const builds = await screen.findByRole('region', { name: 'Stored builds' });
    expect(
      within(builds).getByText(
        'No build has been stored yet. Analyze an artifact and it appears here.',
      ),
    ).toBeDefined();

    buildsMock.mockResolvedValueOnce(buildPage([]));
    const form = within(builds).getByRole('form');
    fireEvent.change(within(form).getByRole('searchbox'), { target: { value: 'nope' } });
    fireEvent.click(within(form).getByRole('button', { name: 'Apply' }));

    await screen.findByText('No stored build matches \u201cnope\u201d.');
  });

  it('keeps a failed read inside the one view it belongs to', async () => {
    runsMock.mockResolvedValue(fail(READ_REFUSED));
    await openHistory();

    // U1P §12 shows one stored entity at a time, so the refusal is read where it lives — inside the Gate
    // runs view — and the isolation this test has always proved is now proved one view at a time: the
    // other two still show exactly what the shell returned for them.
    await showView('Gate runs');
    const alert = await screen.findByRole('alert');
    expect(within(alert).getByText('ERR-STORAGE-4009')).toBeDefined();
    expect(within(alert).getByText('the store is busy')).toBeDefined();
    expect(within(alert).getByText('Try again after the running operation finishes.')).toBeDefined();
    expect(within(alert).getByText('op-hist-7')).toBeDefined();

    const builds = await buildsRegion();
    expect(within(builds).getByText(BUILD_FILE)).toBeDefined();
    const releases = await releasesRegion();
    expect(within(releases).getByText('2.1.0')).toBeDefined();
    expect(runsMock).toHaveBeenCalledTimes(1);
  });

  it('announces that it is loading before a table has answered', async () => {
    buildsMock.mockReturnValue(new Promise<never>(() => undefined));
    render(<App />);
    fireEvent.click(await screen.findByRole('button', { name: 'History page' }));

    const builds = await screen.findByRole('region', { name: 'Stored builds' });
    expect(within(builds).getByRole('status').textContent).toContain('Loading builds');
    // Rows that answer the previous question are gone before the new one is asked, so a slow reply
    // cannot be mistaken for the current one.
    expect(within(builds).queryAllByRole('table')).toEqual([]);
  });
});

describe('U1P History hierarchy', () => {
  it('names the page once, then shows the one stored list the reader asked for', async () => {
    await openHistory();

    // §12: one canonical heading. The page once answered to one name here and another in the rail (R5 names it
    // "History" in both), so the window title and the screen no longer disagree about the same place.
    expect(screen.getByRole('heading', { level: 1, name: 'History' })).toBeDefined();

    const views = screen.getByRole('tablist', { name: 'Stored history' });
    expect(
      within(views)
        .getAllByRole('tab')
        .map((tab) => [tab.textContent, tab.getAttribute('aria-selected')]),
    ).toEqual([
      ['Builds', 'true'],
      ['Gate runs', 'false'],
      ['Release records', 'false'],
    ]);

    // The other two lists are not parked off-screen, they are not mounted, so a reader who navigates by
    // landmark cannot land on a table they did not ask for.
    const builds = screen.getByRole('region', { name: 'Stored builds' });
    expect(precedes(views, builds)).toBe(true);
    expect(screen.queryByRole('region', { name: 'Stored Gate runs' })).toBeNull();
    expect(screen.queryByRole('region', { name: 'Stored release records' })).toBeNull();

    fireEvent.click(within(views).getByRole('tab', { name: 'Gate runs' }));
    expect(screen.queryByRole('region', { name: 'Stored builds' })).toBeNull();
    expect(await screen.findByRole('region', { name: 'Stored Gate runs' })).toBeDefined();
  });

  it('closes an open row when the view changes, so no detail outlives the list it came from', async () => {
    await openHistory();
    const builds = await buildsRegion();
    await openFirstRow(builds);
    expect(screen.getByText(SHA256)).toBeDefined();

    await showView('Gate runs');
    expect(screen.queryByText(SHA256)).toBeNull();

    await showView('Builds');
    await buildsRegion();
    expect(screen.queryByRole('complementary', { name: 'Evidence' })).toBeNull();
  });
});

describe('the same rows read in Chinese', () => {
  /** The stored choice, in the shape the boundary actually carries. */
  function preferences(language: 'en' | 'zh-CN') {
    return {
      ok: true as const,
      value: {
        sizeUnit: 'bytes',
        startupPage: 'history',
        theme: 'system',
        language,
        recovery: 'read' as const,
        keptUnreadableCopy: false,
        fileName: 'settings.json',
      },
    };
  }

  /**
   * The stored columns of every row: artifact, snapshot, digest, stored time, format.
   *
   * Compared by position, because the table has one shape in either language and only two of its
   * columns hold a word at all. Column 0 is the row's own action (详情 / Open), and the last two pair a
   * qualifier with a byte count, so those three are the catalog's business rather than this test's.
   */
  function dataCells(region: HTMLElement): string[][] {
    const body = region.querySelector('tbody');
    return [...(body?.querySelectorAll('tr') ?? [])].map((row) =>
      // R5: row action, select, then artifact, digest, stored time and format.
      [...row.querySelectorAll('th,td')].map(prose).slice(2, 6),
    );
  }

  function headOf(region: HTMLElement): string[] {
    return [...(region.querySelectorAll('thead th') ?? [])].map(prose);
  }

  /** Open History straight from the stored startup page, in whichever language is asked for. */
  let view: ReturnType<typeof render>;
  async function openIn(language: 'en' | 'zh-CN'): Promise<HTMLElement> {
    vi.mocked(getApplicationPreferences).mockResolvedValueOnce(preferences(language));
    view = render(<App />);
    const name = language === 'en' ? 'Stored builds' : '已保存的构建';
    const region = await screen.findByRole('region', { name });
    // The region is on screen before its rows are, and an empty table has no headers to compare. The
    // file name is data, so this wait reads the same in either language.
    await within(region).findByText(BUILD_FILE);
    return region;
  }

  it('names the table, its columns and its pager in Chinese', async () => {
    clearFallbacks();
    buildsMock.mockResolvedValue(
      buildPage([
        candidate(),
        candidate({ buildId: BASELINE_ID, snapshotId: `snap-${'q'.repeat(40)}`, fileName: 'next.elf' }),
      ]),
    );

    const region = await openIn('zh-CN');
    expect(headOf(region)).toContain('镜像文件');
    expect(within(region).getByText('显示构建：第 1 到 2 个，共 2 个')).toBeDefined();
    expect(within(region).getByRole('button', { name: '上一页' }).hasAttribute('disabled')).toBe(true);
    // This page's own word for the action, next to the shared component's word for the sentence.
    expect(within(region).getByRole('button', { name: '应用' })).toBeDefined();
    expect(fallbacks()).toEqual([]);
  });

  it('moves no row, no id and no count when the language changes, and changes every column name', async () => {
    buildsMock.mockResolvedValue(
      buildPage([
        candidate(),
        candidate({ buildId: BASELINE_ID, snapshotId: `snap-${'q'.repeat(40)}`, fileName: 'next.elf' }),
      ]),
    );

    const english = await openIn('en');
    const englishCells = dataCells(english);
    const englishHead = headOf(english);
    expect(document.documentElement.lang).toBe('en');
    view.unmount();

    const chinese = await openIn('zh-CN');
    expect(document.documentElement.lang).toBe('zh-CN');

    // Five stored columns across two rows, identical in both languages: an id, a digest, a timestamp
    // and a file name are Core's bytes, and a translation that moved one would be a data bug.
    const chineseCells = dataCells(chinese);
    expect(englishCells).toHaveLength(2);
    expect(chineseCells).toEqual(englishCells);
    expect(englishCells[0]?.[0]).toBe('app.elf');
    expect(englishCells[1]?.[0]).toBe('next.elf');

    // Every header moved, so the equality above is not two tables that failed to render.
    const chineseHead = headOf(chinese);
    expect(chineseHead).toHaveLength(englishHead.length);
    expect(chineseHead).not.toEqual(englishHead);
    expect(chineseHead).toContain('镜像文件');
    expect(englishHead).toContain('Artifact');
  });
});

describe('R5 pinned baseline and comparisons from History', () => {
  const OLDER = candidate({
    buildId: BASELINE_ID,
    snapshotId: `snap-${'o'.repeat(40)}`,
    fileName: 'old.elf',
    sha256: 'c'.repeat(64),
    importedAt: '2026-09-01T08:00:00Z',
  });

  function pinOf(row: CompareCandidateDto) {
    return {
      buildId: row.buildId,
      snapshotId: row.snapshotId,
      fileName: row.fileName,
      sha256: row.sha256,
      pinnedAt: '2026-10-17T09:00:00Z',
    };
  }

  it('says no baseline is pinned until a person pins one, then names it with when it was pinned', async () => {
    vi.mocked(pinBaseline).mockResolvedValue(ok(pinOf(candidate())));
    await openHistory();
    const line = screen.getByRole('region', { name: 'Pinned baseline' });
    expect(line.textContent).toContain('No baseline is pinned');

    fireEvent.click(screen.getByRole('button', { name: `Pin ${BUILD_FILE} · ${SHA256.slice(0, 12)} as baseline` }));
    await waitFor(() => expect(line.textContent).toContain(`${BUILD_FILE} · ${SHA256.slice(0, 12)}`));
    expect(line.textContent).toContain('pinned 2026-10-17T09:00:00Z');
    expect(vi.mocked(pinBaseline)).toHaveBeenCalledWith(SNAPSHOT_ID);
    // The pinned row is marked, and offers no second pin.
    const builds = await buildsRegion();
    expect(within(builds).getByText(/Pinned/)).toBeDefined();
    expect(within(builds).queryByRole('button', { name: /as baseline$/ })).toBeNull();
  });

  it('unpins, and says so, without touching a stored row', async () => {
    vi.mocked(pinBaseline).mockResolvedValue(ok(pinOf(candidate())));
    vi.mocked(unpinBaseline).mockResolvedValue(ok(true));
    await openHistory();
    fireEvent.click(screen.getByRole('button', { name: /as baseline$/ }));
    fireEvent.click(await screen.findByRole('button', { name: 'Unpin' }));
    const line = screen.getByRole('region', { name: 'Pinned baseline' });
    await waitFor(() => expect(line.textContent).toContain('No baseline is pinned'));
    expect(buildsMock).toHaveBeenCalledTimes(1);
  });

  it('reports a refused pin as a typed error and keeps the pin it had', async () => {
    vi.mocked(pinBaseline).mockResolvedValue(fail(READ_REFUSED));
    await openHistory();
    fireEvent.click(screen.getByRole('button', { name: /as baseline$/ }));
    expect(await screen.findByRole('alert', { name: 'Baseline pin failed' })).toBeDefined();
    expect(screen.getByRole('region', { name: 'Pinned baseline' }).textContent).toContain('No baseline is pinned');
  });

  it('compares the two selected builds on Compare, the earlier one as the base', async () => {
    buildsMock.mockResolvedValue(buildPage([candidate(), OLDER]));
    vi.mocked(listCompareCandidates).mockResolvedValue(
      ok({ rows: [candidate(), OLDER], total: 2, offset: 0, limit: 25, nextOffset: null }),
    );
    vi.mocked(compareSnapshots).mockResolvedValue(
      fail({ code: 'ERR-DIFF-5001', message: 'stub', operationId: 'op-c', details: null, remediation: null }),
    );
    await openHistory();
    const compare = screen.getByRole('button', { name: /^Compare the two selected/ }) as HTMLButtonElement;
    expect(compare.disabled).toBe(true);
    // The older build is ticked first and the newer second, so neither tick order rule could pass for the
    // earlier-stored-is-base rule (R5 review S3).
    fireEvent.click(screen.getByRole('checkbox', { name: /^Select old\.elf/ }));
    fireEvent.click(screen.getByRole('checkbox', { name: `Select ${BUILD_FILE} · ${SHA256.slice(0, 12)} for comparison` }));
    expect(compare.disabled).toBe(false);
    fireEvent.click(compare);

    await screen.findByRole('heading', { level: 1, name: 'Compare' });
    await waitFor(() =>
      expect(vi.mocked(compareSnapshots)).toHaveBeenCalledWith({
        baseSnapshotId: OLDER.snapshotId,
        targetSnapshotId: SNAPSHOT_ID,
      }),
    );
    // Once: the request is not repeated on a later render, nor on a later visit to Compare.
    await act(async () => {
      await Promise.resolve();
    });
    expect(vi.mocked(compareSnapshots)).toHaveBeenCalledTimes(1);
    fireEvent.click(screen.getByRole('button', { name: 'History page' }));
    await screen.findByRole('heading', { level: 1, name: 'History' });
    fireEvent.click(screen.getByRole('button', { name: 'Compare page' }));
    await screen.findByRole('heading', { level: 1, name: 'Compare' });
    await act(async () => {
      await Promise.resolve();
    });
    expect(vi.mocked(compareSnapshots)).toHaveBeenCalledTimes(1);
  });

  it('cannot compare with the current build before Analyze has proved one, and says why', async () => {
    await openHistory();
    expect(
      (screen.getByRole('button', { name: `Compare ${BUILD_FILE} · ${SHA256.slice(0, 12)} with the current build` }) as HTMLButtonElement)
        .disabled,
    ).toBe(true);
    expect(screen.getByRole('region', { name: 'Pinned baseline' }).textContent).toContain(
      'The current build is the one Analyze last proved',
    );
  });
});

describe('R5 command palette and keyboard shortcuts', () => {
  const palette = () => screen.getByRole('dialog', { name: 'Command palette' });

  it('opens on Ctrl+K and from the product bar, filters as you type, and runs the active command on Enter', async () => {
    vi.mocked(listCompareCandidates).mockResolvedValue(
      ok({ rows: [], total: 0, offset: 0, limit: 25, nextOffset: null }),
    );
    await openHistory();
    fireEvent.keyDown(document, { key: 'k', ctrlKey: true });
    const input = within(palette()).getByRole('combobox', { name: 'Search commands' });
    expect(document.activeElement).toBe(input);
    fireEvent.change(input, { target: { value: 'compare' } });
    const options = within(palette()).getAllByRole('option');
    expect(options[0]?.textContent).toContain('Compare');
    expect(options.every((option) => /compare/i.test(option.textContent ?? ''))).toBe(true);
    fireEvent.keyDown(input, { key: 'Enter' });
    expect(screen.queryByRole('dialog', { name: 'Command palette' })).toBeNull();
    expect(await screen.findByRole('heading', { level: 1, name: 'Compare' })).toBeDefined();

    fireEvent.click(screen.getByRole('button', { name: 'Commands (Ctrl+K)' }));
    expect(palette()).toBeDefined();
  });

  it('closes on Esc and gives focus back to where it was', async () => {
    await openHistory();
    const tab = screen.getByRole('tab', { name: 'Gate runs' });
    tab.focus();
    fireEvent.keyDown(document, { key: 'K', ctrlKey: true });
    fireEvent.keyDown(within(palette()).getByRole('combobox'), { key: 'Escape' });
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Command palette' })).toBeNull());
    expect(document.activeElement).toBe(tab);
  });

  it('lists comparing with the pinned baseline with the reason it cannot run yet, and does nothing on it', async () => {
    await openHistory();
    fireEvent.keyDown(document, { key: 'k', ctrlKey: true });
    const option = within(palette()).getByRole('option', { name: /Compare the current build with the pinned baseline/ });
    expect(option.getAttribute('aria-disabled')).toBe('true');
    expect(option.textContent).toContain('No baseline is pinned. Pin one on History.');
    fireEvent.click(option);
    expect(palette()).toBeDefined();
  });

  it('moves to the filter on "/" and between views on "[" and "]", but never while typing', async () => {
    await openHistory();
    await buildsRegion();
    fireEvent.keyDown(document.body, { key: ']' });
    expect(screen.getByRole('tab', { name: 'Gate runs' }).getAttribute('aria-selected')).toBe('true');
    fireEvent.keyDown(document.body, { key: '[' });
    expect(screen.getByRole('tab', { name: 'Builds' }).getAttribute('aria-selected')).toBe('true');

    fireEvent.keyDown(document.body, { key: '/' });
    const field = screen.getByRole('searchbox');
    expect(document.activeElement).toBe(field);
    // Typed into the field, the same keys are text.
    fireEvent.keyDown(field, { key: ']' });
    expect(screen.getByRole('tab', { name: 'Builds' }).getAttribute('aria-selected')).toBe('true');
  });

  it('opens the firmware chooser once on Ctrl+O, from any page', async () => {
    const { selectArtifact } = await import('./ipc/bridge');
    vi.mocked(selectArtifact).mockResolvedValue({ ok: true, value: null });
    await openHistory();
    fireEvent.keyDown(document, { key: 'o', ctrlKey: true });
    expect(await screen.findByRole('heading', { level: 1, name: 'Analyze' })).toBeDefined();
    await waitFor(() => expect(vi.mocked(selectArtifact)).toHaveBeenCalledTimes(1));
    await act(async () => {
      await Promise.resolve();
    });
    expect(vi.mocked(selectArtifact)).toHaveBeenCalledTimes(1);
  });

  it('switches the theme from the palette and stores the choice', async () => {
    const { updateApplicationPreferences } = await import('./ipc/bridge');
    await openHistory();
    fireEvent.keyDown(document, { key: 'k', ctrlKey: true });
    fireEvent.click(within(palette()).getByRole('option', { name: /^Theme\s*Dark/ }));
    await waitFor(() =>
      expect(vi.mocked(updateApplicationPreferences)).toHaveBeenLastCalledWith('bytes', 'analyze', 'dark', 'system'),
    );
    expect(document.documentElement.getAttribute('data-fs-theme')).toBe('dark');
  });
});

describe('R5 review fixes', () => {
  it('says the pin is Unknown when it cannot be read, never that none is pinned, and reads it again on request', async () => {
    vi.mocked(getBaselinePin)
      .mockResolvedValueOnce(fail(READ_REFUSED))
      .mockResolvedValueOnce(
        ok({ buildId: BUILD_ID, snapshotId: SNAPSHOT_ID, fileName: BUILD_FILE, sha256: SHA256, pinnedAt: '2026-10-17T09:00:00Z' }),
      );
    await openHistory();
    const line = screen.getByRole('region', { name: 'Pinned baseline' });
    expect(line.textContent).toContain('could not be read');
    expect(line.textContent).not.toContain('No baseline is pinned');
    expect(screen.getByRole('alert', { name: 'Reading the pinned baseline failed' })).toBeDefined();
    fireEvent.click(within(line).getByRole('button', { name: 'Read the pin again' }));
    await waitFor(() => expect(line.textContent).toContain(`${BUILD_FILE} · ${SHA256.slice(0, 12)}`));
    expect(screen.queryByRole('alert', { name: 'Reading the pinned baseline failed' })).toBeNull();
  });

  it('keeps a pin made while the first read was still in flight', async () => {
    let finish!: (value: IpcOutcome<null>) => void;
    vi.mocked(getBaselinePin).mockReturnValueOnce(new Promise((resolve) => { finish = resolve; }));
    vi.mocked(pinBaseline).mockResolvedValue(
      ok({ buildId: BUILD_ID, snapshotId: SNAPSHOT_ID, fileName: BUILD_FILE, sha256: SHA256, pinnedAt: '2026-10-17T09:00:00Z' }),
    );
    await openHistory();
    fireEvent.click(screen.getByRole('button', { name: /as baseline$/ }));
    const line = screen.getByRole('region', { name: 'Pinned baseline' });
    await waitFor(() => expect(line.textContent).toContain(BUILD_FILE));
    await act(async () => {
      finish(ok(null));
    });
    expect(line.textContent).toContain(BUILD_FILE);
  });

  it('opens no palette over an open Sheet', async () => {
    await openHistory();
    fireEvent.click(screen.getByRole('button', { name: 'Settings page' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Reset application preferences' }));
    await screen.findByRole('dialog', { name: 'Reset all four settings to their defaults?' });
    fireEvent.keyDown(document, { key: 'k', ctrlKey: true });
    expect(screen.queryByRole('dialog', { name: 'Command palette' })).toBeNull();
    fireEvent.keyDown(document, { key: 'o', ctrlKey: true });
    expect(screen.getByRole('heading', { level: 1, name: 'Settings' })).toBeDefined();
  });
});
