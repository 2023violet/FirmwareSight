/**
 * P5's Help and About surface, and the honest version of "where is the documentation".
 *
 * Prompt §14 asks for four things a stranger can reach without a browser: what the tool is, which
 * platforms it claims, the current version, and known limitations. Three of those four are documents
 * this repository keeps, and a desktop package installs a binary rather than a tree of markdown. So
 * the claims these tests protect are narrow:
 *
 * - the identity block is the **running application's** answer, asked for once and rendered verbatim;
 *   before it arrives the screen says it has not been told, and it never shows a version the front end
 *   kept for itself (D1: the workspace version is the single source);
 * - a failed identity read is a typed error in that one section, and the parts of the page that need no
 *   reply — what it reads, local first, getting started — stay on screen;
 * - a pointer the reader cannot follow is not printed as if it worked: the documents that ship only in
 *   the source distribution are flagged on the line that names them;
 * - there is no external link, because this build has no URL a person without the source could open,
 *   and no network to open it with (`AGENTS.md` 7);
 * - the first-use guidance is a panel, not a gate, and hiding it is the reader's decision for the
 *   session (prompt §13).
 *
 * The Diagnostics section (prompt §20) is held to three more:
 *
 * - it shows the five facts the exported file is built from, and it shows them from
 *   `collect_diagnostics` rather than from the identity call, so the two reads cannot substitute for
 *   each other and neither one's failure blanks the other;
 * - the export action reports what the dialog actually did, and declining to choose or to overwrite is
 *   a decision the screen states as a decision, not an error;
 * - nothing the section renders can hold a path. The payload has no such field (that is proven in
 *   Rust, `apps/desktop/src-tauri/tests/diagnostics.rs`), so what this file checks is that the screen
 *   does not invent one — no directory, no drive letter, no separator anywhere in the section's text.
 */

import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { App } from './App';
import { exportDiagnostics, getAppIdentity, getDiagnostics, setWindowTitle } from './ipc/bridge';
import type { IpcOutcome } from './ipc/bridge';
import type {
  AppIdentityDto,
  DiagnosticsDto,
  DiagnosticsStoreDto,
  ErrorEnvelopeDto,
  ExportOutcomeDto,
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
  getAppIdentity: vi.fn(),
  getDiagnostics: vi.fn(),
  exportDiagnostics: vi.fn(),
  // Empty pages, because one test walks to History to prove the first-use panel does not come back.
  listHistoryBuilds: vi.fn(() =>
    Promise.resolve({ ok: true, value: { rows: [], total: 0, offset: 0, limit: 25, nextOffset: null } }),
  ),
  listHistoryGateRuns: vi.fn(() =>
    Promise.resolve({ ok: true, value: { rows: [], total: 0, offset: 0, limit: 25, nextOffset: null } }),
  ),
  listHistoryReleases: vi.fn(() =>
    Promise.resolve({ ok: true, value: { rows: [], total: 0, offset: 0, limit: 25, nextOffset: null } }),
  ),
}));

const identityMock = vi.mocked(getAppIdentity);
const titleMock = vi.mocked(setWindowTitle);
const diagnosticsMock = vi.mocked(getDiagnostics);
const exportMock = vi.mocked(exportDiagnostics);

function ok<T>(value: T): IpcOutcome<T> {
  return { ok: true, value };
}

function fail<T>(envelope: ErrorEnvelopeDto): IpcOutcome<T> {
  return { ok: false, envelope };
}

/**
 * A node's text with the source's line breaks folded back into single spaces, so an assertion matches
 * the sentence the reader sees rather than the way the component happened to wrap it.
 */
function prose(node: Element | null): string {
  return (node?.textContent ?? '').replace(/\s+/g, ' ');
}

/** What this machine's shell answers: every field is the runtime's own, and none of them this file's. */
function identity(overrides: Partial<AppIdentityDto> = {}): AppIdentityDto {
  const base: AppIdentityDto = {
    productName: 'FirmwareSight',
    binaryName: 'firmwaresight.exe',
    appVersion: '0.6.0',
    identifier: 'com.firmwaresight.desktop',
    platform: 'windows-x86_64',
    storageSchemaVersion: 5n,
    storeFileName: 'firmwaresight-p0.sqlite',
  };
  return { ...base, ...overrides };
}

const IDENTITY_REFUSED: ErrorEnvelopeDto = {
  code: 'ERR-INTERNAL-9001',
  message: 'FirmwareSight could not read its own package information.',
  operationId: 'op-about-3',
  details: 'the embedded package info was empty',
  remediation: 'Restart FirmwareSight; if it persists, report this message.',
};

/**
 * What `collect_diagnostics` answers on a healthy machine.
 *
 * Every value here is the shape the Rust payload has, fields and all, because the section is only
 * under test if it is fed what the real command feeds it: `storeFileName` is a name rather than a path,
 * the counts are nullable, and `osVersion` says `not_reported` rather than guessing.
 */
function diagnostics(overrides: Partial<DiagnosticsDto> = {}): DiagnosticsDto {
  const base: DiagnosticsDto = {
    schema: 'firmwaresight-diagnostics-1',
    generatedAt: '2026-10-04T09:00:00Z',
    product: {
      productName: 'FirmwareSight',
      appVersion: '0.6.0',
      binaryName: 'firmwaresight.exe',
      identifier: 'com.firmwaresight.desktop',
      storeFileName: 'firmwaresight-p0.sqlite',
    },
    runtime: {
      osFamily: 'windows',
      architecture: 'x86_64',
      platform: 'windows-x86_64',
      osVersion: 'not_reported',
      tauriVersion: '2.10.2',
      webviewVersion: 'not_reported',
    },
    store: {
      schemaVersion: 5,
      supportedSchemaVersion: 5,
      health: 'healthy',
      healthSummary: null,
      healthErrorCode: null,
      journalMode: 'wal',
      counts: { projects: 1, builds: 3, gateRuns: 1, acceptedReviews: 0, releaseRecords: 0 },
      backupFiles: [],
    },
    git: { available: true, version: 'git version 2.45.1.windows.1' },
    support: {
      inputCohort: 'ELF firmware artifacts, with an optional GNU ld MAP file beside them',
      installChannel: 'msi',
    },
    policy: null,
    recentErrorCodes: [],
  };
  return { ...base, ...overrides };
}

/** The same payload with the store section moved, so a health path is one line of fixture. */
function withStore(overrides: Partial<DiagnosticsStoreDto>): DiagnosticsDto {
  const base = diagnostics();
  return { ...base, store: { ...base.store, ...overrides } };
}

const DIAGNOSTICS_REFUSED: ErrorEnvelopeDto = {
  code: 'ERR-STORAGE-4012',
  message: 'FirmwareSight could not ask the store how healthy it is.',
  operationId: 'op-diagnostics-2',
  details: null,
  remediation: 'Export diagnostics once it answers, then restart FirmwareSight.',
};

const EXPORT_REFUSED: ErrorEnvelopeDto = {
  code: 'ERR-DESKTOP-7004',
  message: 'The diagnostics file could not be written.',
  operationId: 'op-diagnostics-5',
  details: 'the chosen folder is read-only',
  remediation: 'Choose a folder you can write to and try again.',
};

/** The seven answers prompt §13 lists, titled the way the shared component titles them. */
const ANSWERS: readonly string[] = [
  'What FirmwareSight does',
  'First action',
  'What a MAP file changes',
  'The product flow',
  'The state words',
  'Local first',
  'Where a project\u2019s policy enters',
];

async function openHelp(): Promise<void> {
  render(<App />);
  fireEvent.click(await screen.findByRole('button', { name: 'Help page' }));
  await screen.findByRole('heading', { level: 1, name: 'Help' });
}

function aboutSection(): HTMLElement {
  return screen.getByRole('region', { name: 'About this application' });
}

function diagnosticsSection(): HTMLElement {
  return screen.getByRole('region', { name: 'Diagnostics' });
}

beforeEach(() => {
  identityMock.mockReset();
  titleMock.mockReset();
  diagnosticsMock.mockReset();
  exportMock.mockReset();
  identityMock.mockResolvedValue(ok(identity()));
  titleMock.mockResolvedValue(ok(null));
  diagnosticsMock.mockResolvedValue(ok(diagnostics()));
  exportMock.mockResolvedValue(
    ok({ status: 'written', fileName: 'firmwaresight-diagnostics.json', format: 'json' }),
  );
});

describe('Help says which application is running', () => {
  it('renders the identity the shell reported, verbatim', async () => {
    await openHelp();

    const section = await screen.findByRole('region', { name: 'About this application' });
    const rows = within(section).getAllByRole('definition');
    const values = rows.map((row) => row.textContent);

    // Each of the seven is the shell's own answer, and nothing here is a copy the front end kept.
    expect(values).toEqual([
      'FirmwareSight',
      '0.6.0',
      'firmwaresight.exe',
      'com.firmwaresight.desktop',
      'windows-x86_64',
      'v5',
      'firmwaresight-p0.sqlite',
    ]);
  });

  it('asks the shell once, and asks for nothing', async () => {
    await openHelp();
    await within(aboutSection()).findByText('0.6.0');

    expect(identityMock).toHaveBeenCalledTimes(1);
    expect(identityMock.mock.calls[0]).toEqual([]);
  });

  it('says it has not been told instead of showing a version it kept', async () => {
    let resolve!: (outcome: IpcOutcome<AppIdentityDto>) => void;
    identityMock.mockReturnValue(
      new Promise<IpcOutcome<AppIdentityDto>>((done) => {
        resolve = done;
      }),
    );
    render(<App />);
    fireEvent.click(await screen.findByRole('button', { name: 'Help page' }));

    const section = await screen.findByRole('region', { name: 'About this application' });
    // Seven rows, seven times the same sentence: the screen holds no default for any of them, because a
    // number written here would be a second source of truth and would drift with the workspace version.
    expect(within(section).getAllByText('not reported')).toHaveLength(7);
    expect(prose(section)).not.toContain('0.6.0');

    resolve(ok(identity({ appVersion: '0.7.0-local', storageSchemaVersion: 9n })));
    await within(section).findByText('0.7.0-local');
    expect(within(section).getByText('v9')).toBeDefined();
    expect(within(section).queryByText('not reported')).toBeNull();
  });

  it('names the store as a file and refuses to locate it', async () => {
    await openHelp();
    await within(aboutSection()).findByText('firmwaresight-p0.sqlite');

    const section = aboutSection();
    // Diagnostics exists now, and it does not answer this question either: prompt §19 keeps the
    // directory out of that payload, so the sentence that used to point the reader at it was a promise
    // no surface in this product could keep.
    expect(
      within(section).getByText(/absolute path stays inside the application/),
    ).toBeDefined();
    expect(prose(section)).not.toMatch(/[A-Za-z]:[\\/]/);
    expect(prose(section)).not.toMatch(/\\AppData\\/);
    expect(prose(section)).not.toContain('/Users/');
  });

  it('keeps a failed identity read inside that one section', async () => {
    identityMock.mockResolvedValue(fail(IDENTITY_REFUSED));
    await openHelp();

    const alert = await screen.findByRole('alert');
    expect(within(alert).getByText('ERR-INTERNAL-9001')).toBeDefined();
    expect(
      within(alert).getByText('Restart FirmwareSight; if it persists, report this message.'),
    ).toBeDefined();
    expect(within(alert).getByText('op-about-3')).toBeDefined();

    // The rest of the page needs no reply from the shell, so it does not disappear with one.
    expect(screen.getByRole('region', { name: 'What it can read' })).toBeDefined();
    expect(screen.getByRole('region', { name: 'Local-first statement' })).toBeDefined();
    expect(screen.getByRole('region', { name: 'Getting started' })).toBeDefined();
  });

  it('lets the shell name the window', async () => {
    await openHelp();
    await within(aboutSection()).findByText('0.6.0');

    expect(titleMock.mock.calls.map((call) => call[0])).toContain('Help');
  });
});

describe('Help bounds what it claims', () => {
  it('states what can be read, and names the document that is authoritative', async () => {
    await openHelp();

    const section = await screen.findByRole('region', { name: 'What it can read' });
    const words = prose(section);
    expect(words).toContain('ELF firmware artifacts');
    expect(words).toContain('GNU ld MAP');
    expect(words).toContain('refused rather than partly believed');
    expect(words).toContain('this screen is not a substitute for it');
  });

  it('states the local-first boundary in the same words the product holds to', async () => {
    await openHelp();

    const words = prose(await screen.findByRole('region', { name: 'Local-first statement' }));
    for (const claim of [
      'no account',
      'no telemetry',
      'no analytics sdk',
      'no update service',
      'no network listener',
    ]) {
      expect(words.toLowerCase()).toContain(claim);
    }
    expect(words).toContain('never copied into the application\u2019s own database');
  });

  it('lists a document that is not installed as not installed', async () => {
    await openHelp();

    const section = await screen.findByRole('region', { name: 'Where the documents live' });
    const items = within(section).getAllByRole('listitem');
    const flagged = items.filter((item) => prose(item).includes('not carried inside the installed package'));

    expect(items).toHaveLength(4);
    // Getting Started is on this screen, and so is Diagnostics now that Commit D shipped it. The two
    // that remain are source-tree documents a package does not carry.
    expect(flagged).toHaveLength(2);
    expect(prose(items[0] ?? null)).toContain('this screen, and the panel on Analyze');
    expect(prose(items[0] ?? null)).not.toContain('not carried');
    expect(prose(items[3] ?? null)).toContain('this screen, and the file you export from it');
    expect(prose(items[3] ?? null)).not.toContain('not carried');
    expect(prose(section)).toContain('04_TECH/20_PLATFORM_SUPPORT.md');
    expect(prose(section)).toContain('P5_VALIDATION/P5_KNOWN_LIMITATIONS.md');
    expect(prose(section)).not.toContain('not in this build yet');
  });

  it('gives a stranger no link to click and no service to trust', async () => {
    await openHelp();
    await within(aboutSection()).findByText('0.6.0');

    const page = screen.getByRole('main');
    expect(page.querySelectorAll('a')).toHaveLength(0);
    expect(page.querySelectorAll('iframe, form[action], [ping]')).toHaveLength(0);
    expect(prose(page)).not.toMatch(/https?:\/\//);
    expect(prose(page)).not.toMatch(/\blocalhost\b/i);
  });

  it('never writes a stage this build has not earned', async () => {
    await openHelp();
    await within(aboutSection()).findByText('0.6.0');

    const words = prose(screen.getByRole('main'));
    for (const claim of [/beta/i, /\brc\b/i, /\bga\b/i, /stable release/i, /certified/i, /production ready/i]) {
      expect(words).not.toMatch(claim);
    }
  });
});

describe('Help repeats what Analyze shows on a first run', () => {
  function terms(panel: HTMLElement): (string | null)[] {
    return within(panel)
      .getAllByRole('term')
      .map((term) => term.textContent);
  }

  it('answers all seven, in the same words Analyze uses', async () => {
    render(<App />);
    const panel = await screen.findByRole('region', { name: 'Getting started' });
    const onAnalyze = terms(panel);

    fireEvent.click(screen.getByRole('button', { name: 'Help page' }));
    await screen.findByRole('heading', { level: 1, name: 'Help' });
    const help = await screen.findByRole('region', { name: 'Getting started' });

    // One array feeds both surfaces, so two different answers to one question is a bug this catches.
    expect(onAnalyze).toEqual(ANSWERS);
    expect(terms(help)).toEqual(ANSWERS);
  });

  it('explains the five state words it will show', async () => {
    render(<App />);
    const panel = await screen.findByRole('region', { name: 'Getting started' });

    for (const word of ['PASS', 'REVIEW', 'BLOCK', 'UNKNOWN', 'N/A']) {
      // A state is icon plus label, so the word itself has to be readable text, not a colour (DESIGN.md 5).
      expect(within(panel).getAllByText(word, { exact: true }).length).toBeGreaterThan(0);
    }
    expect(prose(panel)).toContain('the evidence to evaluate it never arrived');
    expect(prose(panel)).toContain('the rule does not apply to this configuration');
  });

  it('names the project file a policy lives in, and where it enters the flow', async () => {
    render(<App />);
    const panel = await screen.findByRole('region', { name: 'Getting started' });

    expect(within(panel).getByText('firmwaresight.toml')).toBeDefined();
    expect(prose(panel)).toContain('the policy\u2019s own digest is recorded in the Gate run');

    // The flow is one list, in one direction: Analyze first and Bundle last, with no branch back out.
    const steps = [...panel.querySelectorAll('ol li')].map((item) =>
      (item.textContent ?? '').replace(/^then\s*/, ''),
    );
    expect(steps).toEqual(['Analyze', 'Compare', 'Release', 'Bundle']);
  });

  it('is a panel and not a gate: every Analyze control stays reachable', async () => {
    render(<App />);
    const panel = await screen.findByRole('region', { name: 'Getting started' });

    const choose = screen.getByRole('button', { name: 'Choose firmware artifact' });
    choose.focus();
    expect(document.activeElement).toBe(choose);
    expect(panel.closest('dialog')).toBeNull();
    expect(screen.queryByRole('alertdialog')).toBeNull();
    expect((screen.getByRole('button', { name: 'Analyze' }) as HTMLButtonElement).disabled).toBe(true);

    fireEvent.click(within(panel).getByRole('button', { name: 'Hide this' }));
    await waitFor(() =>
      expect(screen.queryByRole('region', { name: 'Getting started' })).toBeNull(),
    );
    expect(screen.getByRole('button', { name: 'Choose firmware artifact' })).toBeDefined();
    expect(screen.getByText(/Nothing has been analyzed/)).toBeDefined();
  });

  it('keeps the guidance hidden for the session, across a page change', async () => {
    render(<App />);
    const panel = await screen.findByRole('region', { name: 'Getting started' });
    fireEvent.click(within(panel).getByRole('button', { name: 'Hide this' }));
    await waitFor(() =>
      expect(screen.queryByRole('region', { name: 'Getting started' })).toBeNull(),
    );

    fireEvent.click(screen.getByRole('button', { name: 'Bundle & History page' }));
    await screen.findByRole('heading', { level: 1, name: 'Bundle & History' });
    fireEvent.click(screen.getByRole('button', { name: 'Analyze page' }));
    await screen.findByRole('heading', { level: 1, name: 'Analyze' });

    // Hiding was the reader's act, so nothing here puts it back; Help is where the words went.
    expect(screen.queryByRole('region', { name: 'Getting started' })).toBeNull();

    fireEvent.click(screen.getByRole('button', { name: 'Help page' }));
    const help = await screen.findByRole('region', { name: 'Getting started' });
    expect(within(help).getByText('What FirmwareSight does')).toBeDefined();
  });
});

describe('Help carries the diagnostics the shell can state', () => {
  it('states the five facts the exported file is built from', async () => {
    await openHelp();

    const section = await screen.findByRole('region', { name: 'Diagnostics' });
    await within(section).findByText('healthy');
    expect(within(section).getAllByRole('definition').map((row) => row.textContent)).toEqual([
      '0.6.0',
      'v5',
      'healthy',
      'available',
      'firmwaresight-p0.sqlite',
    ]);

    // One call, and it asks for nothing: the payload is the shell's own allowlist, so the screen holds
    // no argument with which to widen what gets reported (prompt §19).
    expect(diagnosticsMock).toHaveBeenCalledTimes(1);
    expect(diagnosticsMock.mock.calls[0]).toEqual([]);

    // The local-only boundary is part of the sentence rather than an implication of it (prompt §22).
    const words = prose(section);
    expect(words).toContain('no upload');
    expect(words).toContain('no telemetry');
  });

  it('holds no default for any of them until the shell answers', async () => {
    let resolveDiagnostics!: (outcome: IpcOutcome<DiagnosticsDto>) => void;
    diagnosticsMock.mockReturnValue(
      new Promise<IpcOutcome<DiagnosticsDto>>((done) => {
        resolveDiagnostics = done;
      }),
    );
    render(<App />);
    fireEvent.click(await screen.findByRole('button', { name: 'Help page' }));

    // The identity reply is the wave this waits on, and it is a different call from the one still open
    // below: About standing while Diagnostics has not answered is the point, because the two reads gate
    // their own sections and nothing else.
    await within(aboutSection()).findByText('0.6.0');
    const section = diagnosticsSection();
    expect(within(section).getAllByText('not reported')).toHaveLength(5);
    expect(prose(section)).not.toContain('0.6.0');

    resolveDiagnostics(
      ok(diagnostics({ product: { ...diagnostics().product, appVersion: '0.7.0-local' } })),
    );
    await within(section).findByText('0.7.0-local');
    expect(within(section).queryByText('not reported')).toBeNull();
  });

  it('describes a damaged store in words, and says plainly that it will not touch it', async () => {
    diagnosticsMock.mockResolvedValue(
      ok(withStore({ health: 'unhealthy', healthSummary: 'btree page 41 has an invalid cell count' })),
    );
    await openHelp();

    const section = await screen.findByRole('region', { name: 'Diagnostics' });
    await within(section).findByText('unhealthy');
    const words = prose(section);
    expect(words).toContain('btree page 41 has an invalid cell count');
    expect(words).toContain('does not repair, replace or delete the store');
    // The screen reports a damaged store and stops there: an action that "fixed" one would be a
    // destructive operation offered without confirmation (`AGENTS.md` 9).
    expect(within(section).queryByRole('button', { name: /repair|reset|delete/i })).toBeNull();
  });

  it('names the code when the check could not be asked, and never a message', async () => {
    diagnosticsMock.mockResolvedValue(
      ok(withStore({ health: 'unknown', healthErrorCode: 'ERR-STORAGE-4012' })),
    );
    await openHelp();

    const section = await screen.findByRole('region', { name: 'Diagnostics' });
    await within(section).findByText('unknown');
    const words = prose(section);
    expect(words).toContain('could not be asked');
    expect(words).toContain('ERR-STORAGE-4012');
  });

  it('keeps a failed diagnostics read inside the Diagnostics section', async () => {
    diagnosticsMock.mockResolvedValue(fail(DIAGNOSTICS_REFUSED));
    await openHelp();

    const section = await screen.findByRole('region', { name: 'Diagnostics' });
    const alert = within(section).getByRole('alert');
    expect(within(alert).getByText('ERR-STORAGE-4012')).toBeDefined();
    // The five rows stay, still saying they have not been told, and the sections that needed no reply
    // from this call keep everything they had.
    expect(within(section).getAllByText('not reported')).toHaveLength(5);
    await within(aboutSection()).findByText('0.6.0');
    expect(screen.getByRole('region', { name: 'Local-first statement' })).toBeDefined();
  });

  it('writes the file through the closed command and reports what it did', async () => {
    await openHelp();
    const section = await screen.findByRole('region', { name: 'Diagnostics' });
    await within(section).findByText('healthy');

    fireEvent.click(within(section).getByRole('button', { name: 'Export diagnostics' }));
    await within(section).findByText('Wrote firmwaresight-diagnostics.json.');

    // Argument-free like the read: the destination is chosen in the native dialog Rust opens, so the
    // WebView never names a folder it was not given (`ADR-0025`).
    expect(exportMock).toHaveBeenCalledTimes(1);
    expect(exportMock.mock.calls[0]).toEqual([]);
  });

  it('reports declining the dialog as the decision it is', async () => {
    let resolveExport!: (outcome: IpcOutcome<ExportOutcomeDto>) => void;
    exportMock.mockReturnValue(
      new Promise<IpcOutcome<ExportOutcomeDto>>((done) => {
        resolveExport = done;
      }),
    );
    await openHelp();

    const section = await screen.findByRole('region', { name: 'Diagnostics' });
    const button = within(section).getByRole('button', { name: 'Export diagnostics' });
    fireEvent.click(button);

    // The dialog is the shell's, so the screen says what it is waiting for and takes the click away
    // until the answer lands: a second click would open a second dialog.
    await within(section).findByText(/Waiting for the save dialog/);
    expect((button as HTMLButtonElement).disabled).toBe(true);

    resolveExport(ok({ status: 'cancelled', fileName: null, format: 'json' }));
    await within(section).findByText('Export cancelled. No file was written.');
    expect((button as HTMLButtonElement).disabled).toBe(false);
  });

  it('keeps the facts on screen when the export fails', async () => {
    exportMock.mockResolvedValue(fail(EXPORT_REFUSED));
    await openHelp();

    const section = await screen.findByRole('region', { name: 'Diagnostics' });
    fireEvent.click(within(section).getByRole('button', { name: 'Export diagnostics' }));

    const alert = await within(section).findByRole('alert');
    expect(within(alert).getByText('ERR-DESKTOP-7004')).toBeDefined();
    // A file that was not written says nothing about the store, so the rows above it do not move.
    expect(within(section).getByText('healthy')).toBeDefined();
    expect(within(section).queryByText(/No file was written/)).toBeNull();
  });

  it('prints no folder anywhere in the section', async () => {
    diagnosticsMock.mockResolvedValue(
      ok(withStore({ backupFiles: ['firmwaresight-p0.pre-migration-v4-to-v5.sqlite'] })),
    );
    await openHelp();

    const section = await screen.findByRole('region', { name: 'Diagnostics' });
    await within(section).findByText('healthy');
    const words = prose(section);
    // The payload cannot carry a path at all - proven in Rust, with every one of them planted in the
    // session first. What this checks is narrower and still worth holding: that the screen does not
    // assemble a location out of the pieces it was given.
    expect(words).not.toMatch(/[\\/]/);
    expect(words).not.toContain('AppData');
    expect(words).not.toContain('home');
  });
});
