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
import {
  listHistoryBuilds,
  listHistoryGateRuns,
  listHistoryReleases,
  setWindowTitle,
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

/** The builds table, once its first page has painted. */
async function buildsRegion(): Promise<HTMLElement> {
  await screen.findByText(BUILD_FILE);
  return screen.getByRole('region', { name: 'Stored builds' });
}

async function runsRegion(): Promise<HTMLElement> {
  await screen.findByText(GATE_FILE);
  return screen.getByRole('region', { name: 'Stored Gate runs' });
}

async function releasesRegion(): Promise<HTMLElement> {
  await screen.findByText(RELEASE_FILE);
  return screen.getByRole('region', { name: 'Stored release records' });
}

async function filterForm(noun: string): Promise<HTMLElement> {
  return await screen.findByRole('form', { name: `Filter ${noun}` });
}

/** Open one table's first row and hand back the control that did it. */
async function openFirstRow(region: HTMLElement): Promise<HTMLElement> {
  const details = await within(region).findByRole('button', { name: 'Details' });
  fireEvent.click(details);
  await waitFor(() => expect(details.getAttribute('aria-expanded')).toBe('true'));
  return details;
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

    const header = (await screen.findByRole('heading', { level: 1, name: 'History' })).parentElement;
    const words = prose(header ?? null);
    expect(words).toContain('What this computer has already stored');
    expect(words).toContain('not the artifact currently open on Analyze');
    expect(words).toContain('a row stays readable after the firmware file it came from has moved');
    expect(words).toContain('Nothing on this page changes a stored row.');
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
    fireEvent.change(within(form).getByRole('textbox'), { target: { value: 'bootloader' } });
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
    fireEvent.change(within(form).getByRole('textbox'), { target: { value: 'app' } });
    fireEvent.click(within(form).getByRole('button', { name: 'Apply' }));
    await waitFor(() => expect(buildsMock).toHaveBeenCalledTimes(2));

    // The Gate and release reads still answer the question they were asked, so they were not asked again.
    expect(runsMock).toHaveBeenCalledTimes(1);
    expect(releasesMock).toHaveBeenCalledTimes(1);
    expect(runsMock.mock.calls[0]?.[0]).toEqual({ offset: 0, limit: null, filter: null });
  });

  it('does not filter on a keystroke', async () => {
    await openHistory();

    const form = await filterForm('Gate runs');
    fireEvent.change(within(form).getByRole('textbox'), { target: { value: 'gate' } });

    // The hint is the only thing that moved: a half-typed word must not narrow a table the reader was
    // still reading.
    expect(within(form).getByText('Filtering by \u201cgate\u201d')).toBeDefined();
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
    fireEvent.change(within(form).getByRole('textbox'), { target: { value: 'app' } });
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
    const box = within(form).getByRole('textbox');
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
    // not move - only its rendering.
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
    const details = await within(runs).findByRole('button', { name: 'Details' });
    return details.closest('tr') as HTMLElement;
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

    const cells = within(await firstGateRow()).getAllByRole('cell');
    expect(cells[2]?.textContent).toBe('None');
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

describe('History opens a row without leaving the table', () => {
  it('gives the whole digest back when the row is opened, and takes it away again', async () => {
    await openHistory();
    const builds = await buildsRegion();
    const details = await within(builds).findByRole('button', { name: 'Details' });

    // A shortened id is a convenience, so before the row is opened the screen holds the shortened text
    // and none of the full one.
    expect(details.getAttribute('aria-expanded')).toBe('false');
    expect(screen.queryByText(SHA256)).toBeNull();

    fireEvent.click(details);
    await waitFor(() => expect(details.getAttribute('aria-expanded')).toBe('true'));
    expect(screen.getByText(SHA256)).toBeDefined();
    expect(screen.getByText(BUILD_ID)).toBeDefined();
    expect(screen.getByText(SNAPSHOT_ID)).toBeDefined();
    expect(within(builds).getByText(/The file name is all FirmwareSight keeps visible here/)).toBeDefined();

    fireEvent.click(details);
    await waitFor(() => expect(details.getAttribute('aria-expanded')).toBe('false'));
    expect(screen.queryByText(SHA256)).toBeNull();
  });

  it('opens the ids a stored Gate run and a release record are cited by', async () => {
    await openHistory();

    const runs = await runsRegion();
    fireEvent.click(await within(runs).findByRole('button', { name: 'Details' }));
    await screen.findByText(RUN_ID);
    expect(within(runs).getByText(BASELINE_ID)).toBeDefined();

    const releases = await releasesRegion();
    fireEvent.click(await within(releases).findByRole('button', { name: 'Details' }));
    await screen.findByText(RELEASE_ID);
    expect(within(releases).getByText(MANIFEST_SHA)).toBeDefined();
    expect(within(releases).getByText(BUILD_ID)).toBeDefined();
  });

  it('reaches a row action with the keyboard, because it is a button and not a click target', async () => {
    await openHistory();
    const builds = await buildsRegion();
    const details = await within(builds).findByRole('button', { name: 'Details' });

    expect(details.tagName).toBe('BUTTON');
    expect(details.getAttribute('href')).toBeNull();
    expect(details.getAttribute('tabindex')).toBeNull();
    details.focus();
    expect(document.activeElement).toBe(details);

    fireEvent.click(details);
    await waitFor(() => expect(details.getAttribute('aria-expanded')).toBe('true'));
    expect(screen.getByText(SHA256)).toBeDefined();
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
    // The only fields on the page are the three bounded filters.
    expect(within(page).getAllByRole('textbox')).toHaveLength(3);
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
    fireEvent.change(within(form).getByRole('textbox'), { target: { value: 'nope' } });
    fireEvent.click(within(form).getByRole('button', { name: 'Apply' }));

    await screen.findByText('No stored build matches \u201cnope\u201d.');
  });

  it('keeps two healthy tables on screen when the third cannot be read', async () => {
    runsMock.mockResolvedValue(fail(READ_REFUSED));
    await openHistory();

    const alert = await screen.findByRole('alert');
    expect(within(alert).getByText('ERR-STORAGE-4009')).toBeDefined();
    expect(within(alert).getByText('the store is busy')).toBeDefined();
    expect(within(alert).getByText('Try again after the running operation finishes.')).toBeDefined();
    expect(within(alert).getByText('op-hist-7')).toBeDefined();

    // The failure belongs to one table: the other two still show what the shell returned.
    expect(screen.getByRole('region', { name: 'Stored builds' })).toBeDefined();
    expect(screen.getByRole('region', { name: 'Stored release records' })).toBeDefined();
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
