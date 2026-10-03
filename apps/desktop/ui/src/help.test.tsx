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
 */

import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { App } from './App';
import { getAppIdentity, setWindowTitle } from './ipc/bridge';
import type { IpcOutcome } from './ipc/bridge';
import type { AppIdentityDto, ErrorEnvelopeDto } from './ipc/types';

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

beforeEach(() => {
  identityMock.mockReset();
  titleMock.mockReset();
  identityMock.mockResolvedValue(ok(identity()));
  titleMock.mockResolvedValue(ok(null));
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
    expect(
      within(section).getByText(/Where it sits on this machine is a Diagnostics question/),
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
    // Only Getting Started is reachable from an installed package, and it is on this screen.
    expect(flagged).toHaveLength(3);
    expect(prose(items[0] ?? null)).toContain('this screen, and the panel on Analyze');
    expect(prose(items[0] ?? null)).not.toContain('not carried');
    expect(prose(section)).toContain('04_TECH/20_PLATFORM_SUPPORT.md');
    expect(prose(section)).toContain('P5_VALIDATION/P5_KNOWN_LIMITATIONS.md');
    expect(prose(section)).toContain('not in this build yet');
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

    fireEvent.click(screen.getByRole('button', { name: 'History page' }));
    await screen.findByRole('heading', { level: 1, name: 'History' });
    fireEvent.click(screen.getByRole('button', { name: 'Analyze page' }));
    await screen.findByRole('heading', { level: 1, name: 'Analyze' });

    // Hiding was the reader's act, so nothing here puts it back; Help is where the words went.
    expect(screen.queryByRole('region', { name: 'Getting started' })).toBeNull();

    fireEvent.click(screen.getByRole('button', { name: 'Help page' }));
    const help = await screen.findByRole('region', { name: 'Getting started' });
    expect(within(help).getByText('What FirmwareSight does')).toBeDefined();
  });
});
