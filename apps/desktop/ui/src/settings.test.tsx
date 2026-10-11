/**
 * B1-U1: the application's own settings, on screen.
 *
 * What these tests hold the page to is the boundary the whole unit is about — a preference changes how
 * this window reads and starts, and nothing else:
 *
 * - the values come from the shell, never from a copy the front end keeps (T01, T02). The document on
 *   disk is Rust's; the page asks, and renders the answer plus the reason that came with it;
 * - a first start is stated as a first start (T01). Defaults that are not named are indistinguishable
 *   from settings that were never chosen, which is how a page ends up looking like it does nothing;
 * - a value that could not be stored is shown as a refusal with its code, and the word "saved" does not
 *   appear anywhere near it (T05);
 * - a damaged, future or unreachable document explains itself and says what happened to the bytes that
 *   were there (T04) - the four sentences are the difference between "the settings are gone" and
 *   "the settings are still yours";
 * - reset is asked before it is done, and it is stated as the one narrow thing it is (T06);
 * - the unit is one state shared by every surface that shows it, so a choice made on History is what
 *   Settings then lists as current, whichever way the reader moved (T02);
 * - and no control is rendered for a setting this build does not have. There is no theme switch and no
 *   language switch, because B1-U2 and B1-U3 have not shipped one (T01, and `AGENTS.md` 11).
 */

import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { App } from './App';
import {
  getApplicationPreferences,
  listHistoryBuilds,
  listHistoryGateRuns,
  listHistoryReleases,
  resetApplicationPreferences,
  setWindowTitle,
  updateApplicationPreferences,
} from './ipc/bridge';
import type { IpcOutcome } from './ipc/bridge';
import type { ErrorEnvelopeDto, PreferencesDto } from './ipc/types';

vi.mock('./ipc/bridge', () => ({
  selectArtifact: vi.fn(),
  attachMap: vi.fn(),
  clearMap: vi.fn(),
  analyzeSelection: vi.fn(),
  querySections: vi.fn(),
  querySymbols: vi.fn(),
  queryEvidence: vi.fn(),
  setWindowTitle: vi.fn(() => Promise.resolve({ ok: true, value: null })),
  listHistoryBuilds: vi.fn(),
  listHistoryGateRuns: vi.fn(),
  listHistoryReleases: vi.fn(),
  getApplicationPreferences: vi.fn(),
  updateApplicationPreferences: vi.fn(),
  resetApplicationPreferences: vi.fn(),
  getAppIdentity: vi.fn(() => Promise.resolve({ ok: true, value: {} })),
  getDiagnostics: vi.fn(),
  exportDiagnostics: vi.fn(),
  collectDiagnostics: vi.fn(),
  openProjectConfig: vi.fn(),
  saveProjectPolicy: vi.fn(),
  runReleaseGate: vi.fn(),
  acceptReview: vi.fn(),
  getGateRun: vi.fn(),
  listFixtures: vi.fn(),
  getAnalysisSummary: vi.fn(),
  listCompareCandidates: vi.fn(),
  compareSnapshots: vi.fn(),
  querySectionChanges: vi.fn(),
  querySymbolChanges: vi.fn(),
  exportCompareJson: vi.fn(),
  exportCompareHtml: vi.fn(),
  prepareReleaseBundle: vi.fn(),
  chooseBundleDestination: vi.fn(),
  exportReleaseBundle: vi.fn(),
  attachReleaseFile: vi.fn(),
  setReleaseAttachmentKind: vi.fn(),
  removeReleaseAttachment: vi.fn(),
  getReleaseAttachments: vi.fn(),
}));

const preferencesMock = vi.mocked(getApplicationPreferences);
const updateMock = vi.mocked(updateApplicationPreferences);
const resetMock = vi.mocked(resetApplicationPreferences);
const titleMock = vi.mocked(setWindowTitle);
const buildsMock = vi.mocked(listHistoryBuilds);
const gateRunsMock = vi.mocked(listHistoryGateRuns);
const releasesMock = vi.mocked(listHistoryReleases);

function ok<T>(value: T): IpcOutcome<T> {
  return { ok: true, value };
}

/** One stored document, with the four fields the page reads spelled out. */
function prefs(over: Partial<PreferencesDto> = {}): PreferencesDto {
  return {
    sizeUnit: 'bytes',
    startupPage: 'analyze',
    recovery: 'read',
    keptUnreadableCopy: false,
    fileName: 'settings.json',
    ...over,
  };
}

const SAVE_REFUSED: ErrorEnvelopeDto = {
  code: 'ERR-PREFS-8002',
  message: 'Your preferences were not saved: the preferences could not be written: disk full',
  operationId: 'op-prefs-1',
  details: null,
  remediation:
    'Close whatever program holds the preferences file, then save again; the previously saved \
preferences are still in place',
};

beforeEach(() => {
  preferencesMock.mockResolvedValue(ok(prefs({ recovery: 'missing' })));
  updateMock.mockImplementation(async (sizeUnit: string, startupPage: string) =>
    ok(prefs({ sizeUnit, startupPage })),
  );
  resetMock.mockResolvedValue(ok(prefs()));
  // History answers with empty pages, so the second surface a unit can be chosen on opens at all.
  const page = { rows: [], total: 0, offset: 0, limit: 25, nextOffset: null };
  buildsMock.mockResolvedValue(ok(page));
  gateRunsMock.mockResolvedValue(ok(page));
  releasesMock.mockResolvedValue(ok(page));
});

async function openSettings() {
  render(<App />);
  fireEvent.click(await screen.findByRole('button', { name: 'Settings page' }));
  await screen.findByRole('heading', { level: 1, name: 'Settings' });
}

/**
 * A radio, as the input rather than as a generic element.
 *
 * This project registers no jest-dom matchers (`src/test/setup.ts` only tears the tree down), so a
 * control's state is read from the DOM the same way the other page tests read it: `.checked`.
 */
function unitRadio(label: string): HTMLInputElement {
  return screen.getByRole('radio', { name: label }) as HTMLInputElement;
}

function startRadio(label: string): HTMLInputElement {
  return screen.getByRole('radio', { name: label }) as HTMLInputElement;
}

describe('the Settings page', () => {
  it('opens on the defaults and says that nothing has been stored yet', async () => {
    await openSettings();

    expect(unitRadio('Bytes').checked).toBe(true);
    expect(startRadio('Analyze').checked).toBe(true);
    expect(
      await screen.findByText('Nothing has been stored yet, so this window is using FirmwareSight’s defaults.'),
    ).toBeDefined();
    // The page names the document it writes, and names no folder: a settings screen has no reason to
    // print where the person's profile lives.
    const page = document.body.querySelector('main');
    expect(page?.textContent).toContain('settings.json');
    expect(page?.textContent).not.toMatch(/[\\/]/);
  });

  it('offers no control for a setting this build does not have', async () => {
    await openSettings();

    const words = document.body.textContent ?? '';
    // Theme and language are B1-U2 and B1-U3. A switch that goes nowhere would teach the reader that
    // nothing here works, which is worse than an empty stretch of page.
    for (const absent of ['Theme', 'Dark', 'Light', 'Language', 'Chinese', 'English', 'Font', 'Density']) {
      expect(words).not.toContain(absent);
    }
    expect(screen.queryByRole('combobox')).toBeNull();
  });

  it('applies the unit the document holds before the reader touches anything', async () => {
    preferencesMock.mockResolvedValue(ok(prefs({ sizeUnit: 'kib', recovery: 'read' })));
    await openSettings();

    // The stored value, not the front end's own default, is what the page opens on.
    await waitFor(() => expect(unitRadio('KiB').checked).toBe(true));
  });

  it('stores a unit chosen here, and says so only once the shell agrees', async () => {
    preferencesMock.mockResolvedValue(ok(prefs({ sizeUnit: 'kib', startupPage: 'history' })));
    await openSettings();
    await waitFor(() => expect(unitRadio('KiB').checked).toBe(true));

    fireEvent.click(unitRadio('Bytes'));

    await waitFor(() => expect(updateMock).toHaveBeenCalledWith('bytes', 'history'));
    // Both words travel together: the document is one file, and a half-stored pair is not a setting.
    expect(screen.getByText('Preferences saved.')).toBeDefined();
    expect(unitRadio('Bytes').checked).toBe(true);
  });

  it('shares one unit with the page that shows a table', async () => {
    render(<App />);
    fireEvent.click(await screen.findByRole('button', { name: 'Bundle & History page' }));
    const switchOnHistory = await screen.findByRole('group', { name: 'Size units' });
    fireEvent.click(within(switchOnHistory).getByRole('radio', { name: 'KiB' }) as HTMLInputElement);
    await waitFor(() => expect(updateMock).toHaveBeenCalledWith('kib', 'analyze'));

    fireEvent.click(screen.getByRole('button', { name: 'Settings page' }));
    await screen.findByRole('heading', { level: 1, name: 'Settings' });
    // The same state, not a second copy: choosing it on one surface is what the other lists as current.
    await waitFor(() => expect(unitRadio('KiB').checked).toBe(true));
  });

  it('names a start page chosen here and says when it takes effect', async () => {
    await openSettings();

    fireEvent.click(startRadio('Bundle & History'));

    await waitFor(() => expect(updateMock).toHaveBeenCalledWith('bytes', 'history'));
    expect(startRadio('Bundle & History').checked).toBe(true);
    expect(
      screen.getByText(/takes effect the next time this window is opened/),
    ).toBeDefined();
    // Opening Settings never restores a page out of a stored preference mid-session.
    expect(screen.getByRole('heading', { level: 1, name: 'Settings' })).toBeDefined();
  });

  it('shows a refusal as a refusal, and never as saved', async () => {
    updateMock.mockResolvedValue({ ok: false, envelope: SAVE_REFUSED });
    await openSettings();

    fireEvent.click(unitRadio('KiB'));

    const alert = await screen.findByRole('alert');
    expect(within(alert).getByText('ERR-PREFS-8002')).toBeDefined();
    expect(screen.queryByText('Preferences saved.')).toBeNull();
    // The window still does what the reader asked; the stored document is what did not change.
    expect(unitRadio('KiB').checked).toBe(true);
    expect(
      within(alert).getByText(/previously saved preferences are still in place/),
    ).toBeDefined();
  });

  it('says what happened to bytes it could not read', async () => {
    preferencesMock.mockResolvedValue(
      ok(prefs({ recovery: 'corrupted', keptUnreadableCopy: true })),
    );
    await openSettings();

    expect(
      await screen.findByText(/could not be read[\s\S]*kept beside them/),
    ).toBeDefined();
  });

  it('refuses to interpret a document from a newer version', async () => {
    preferencesMock.mockResolvedValue(ok(prefs({ recovery: 'futureVersion', startupPage: 'analyze' })));
    await openSettings();

    expect(
      await screen.findByText(/written by a newer version of FirmwareSight/),
    ).toBeDefined();
  });

  it('says plainly when this host has nowhere to store anything', async () => {
    preferencesMock.mockResolvedValue(ok(prefs({ recovery: 'nowhereToStore' })));
    await openSettings();

    expect(await screen.findByText(/no configuration folder, so nothing can be stored/)).toBeDefined();
  });

  it('asks before a reset, and only stores the defaults once the answer is yes', async () => {
    preferencesMock.mockResolvedValue(ok(prefs({ sizeUnit: 'kib', startupPage: 'history' })));
    await openSettings();
    await waitFor(() => expect(unitRadio('KiB').checked).toBe(true));

    fireEvent.click(screen.getByRole('button', { name: 'Reset application preferences' }));
    // The question is on screen and nothing has been sent yet.
    await screen.findByRole('group', { name: 'Reset both settings to their defaults?' });
    expect(resetMock).not.toHaveBeenCalled();
    expect(document.activeElement?.textContent).toBe('Reset preferences');

    fireEvent.click(screen.getByRole('button', { name: 'Reset preferences' }));

    await waitFor(() => expect(resetMock).toHaveBeenCalledTimes(1));
    expect(unitRadio('Bytes').checked).toBe(true);
    expect(
      screen.getByText('Preferences reset to this version’s defaults.'),
    ).toBeDefined();
    expect(screen.queryByRole('group', { name: 'Reset both settings to their defaults?' })).toBeNull();
  });

  it('leaves the reset alone when the answer is no', async () => {
    await openSettings();

    fireEvent.click(screen.getByRole('button', { name: 'Reset application preferences' }));
    fireEvent.click(await screen.findByRole('button', { name: 'Keep them' }));

    expect(resetMock).not.toHaveBeenCalled();
    expect(updateMock).not.toHaveBeenCalled();
    expect(screen.getByRole('button', { name: 'Reset application preferences' })).toBeDefined();
  });

  it('names the window after the page that is standing in it', async () => {
    await openSettings();

    await waitFor(() => expect(titleMock).toHaveBeenCalledWith('Settings'));
  });

  it('is walkable with the keyboard in the order the page reads', async () => {
    await openSettings();
    await screen.findByRole('radio', { name: 'Bytes' });

    // Every control is a native one, so Tab reaches it and Space or Enter acts on it without a key
    // handler of our own. What is checkable here is the order: a reader who tabs through the page
    // meets the controls in the sequence the sections are printed in, and each one is already
    // labelled by the text beside it rather than by a title a screen reader has to guess at.
    const controls = Array.from(
      document.querySelectorAll('main button, main input'),
    ) as (HTMLButtonElement | HTMLInputElement)[];
    expect(controls.map((element) => element.tagName.toLowerCase())).toEqual([
      'button',
      'input',
      'input',
      'input',
      'input',
      'button',
    ]);
    const named = controls.map((element) =>
      element.tagName === 'INPUT'
        ? // An input carries no text of its own: the label bound to it by `htmlFor` is its name, and
          // that binding is what makes the radio announceable rather than a bare dot.
          (element.labels?.[0]?.textContent ?? '')
        : (element.textContent ?? ''),
    );
    expect(named).toEqual([
      'Open Help and About',
      'Bytes',
      'KiB',
      'Analyze',
      'Bundle & History',
      'Reset application preferences',
    ]);
    // No control hides itself behind a pointer-only affordance, and none is a div wearing a role.
    for (const element of controls) {
      expect(element.tabIndex).toBeGreaterThanOrEqual(0);
    }
    expect(document.querySelectorAll('main [role=button]')).toHaveLength(0);
  });
});
