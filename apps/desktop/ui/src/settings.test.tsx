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
 * - the language is a fourth stored word, and it is the only preference that repaints the page that
 *   sets it (`ADR-0032` D8). What the reader is shown is the catalog, so a broken switch here is
 *   visible here rather than only in a Rust assertion;
 * - and no control is rendered for a setting this build does not have. Unit, theme, language and
 *   start page are all honoured by something; font scale and density are not, so they are not on the
 *   page (T01, and `AGENTS.md` 11).
 */

import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

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

/** One stored document, with the fields the page reads spelled out. */
function prefs(over: Partial<PreferencesDto> = {}): PreferencesDto {
  return {
    sizeUnit: 'bytes',
    startupPage: 'analyze',
    theme: 'system',
    language: 'system',
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
  updateMock.mockImplementation(
    async (sizeUnit: string, startupPage: string, theme: string, language: string) =>
      ok(prefs({ sizeUnit, startupPage, theme, language })),
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
 * A radio inside the group that offers it, as the input rather than as a generic element.
 *
 * This project registers no jest-dom matchers (`src/test/setup.ts` only tears the tree down), so a
 * control's state is read from the DOM the same way the other page tests read it: `.checked`.
 *
 * The group is no longer optional. "System" is a theme and a language, and an unscoped name lookup
 * would return both — which is not a test finding a bug, it is a test asking a question the page
 * cannot answer. A group label is what makes the name unique, and it is the same label a screen
 * reader announces before the choice.
 */
function radio(role: 'group' | 'radiogroup', groupLabel: string, label: string): HTMLInputElement {
  const group = screen.getByRole(role, { name: groupLabel });
  return within(group).getByRole('radio', { name: label }) as HTMLInputElement;
}

function unitRadio(label: string): HTMLInputElement {
  return radio('group', 'Size units', label);
}

function startRadio(label: string): HTMLInputElement {
  return radio('radiogroup', 'Start page', label);
}

function themeRadio(label: string): HTMLInputElement {
  return radio('radiogroup', 'Theme', label);
}

function languageRadio(label: string): HTMLInputElement {
  return radio('radiogroup', 'Language', label);
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
    // The page names the document it writes, and names no folder: a settings screen has no reason to
    // print where the person's profile lives. The shape checked for is a path, not the character —
    // `N/A` is a Gate word the catalog carries, and a slash inside a word is not a directory.
    const page = document.body.querySelector('main');
    expect(page?.textContent).toContain('settings.json');
    expect(page?.textContent).not.toContain('\\');
    expect(page?.textContent).not.toMatch(/[A-Za-z]:[\\/]/);
    expect(page?.textContent).not.toMatch(/[\\/][\w.-]+[\\/]/);
  });

  it('offers no control for a setting this build does not have', async () => {
    await openSettings();

    const words = document.body.textContent ?? '';
    // Font scale and density are still not honoured by any surface, and no catalog claims to be
    // Traditional Chinese (`ADR-0032` D1 refuses that claim, so the page must not imply it either).
    // Language and theme are not on this list because both are real now: `ADR-0031` gave this build a
    // second palette and `ADR-0032` a second catalog. The rule is unchanged; only what goes nowhere
    // moved.
    for (const absent of ['Font', 'Density', '繁體', 'Traditional']) {
      expect(words).not.toContain(absent);
    }
    expect(screen.queryByRole('combobox')).toBeNull();
    // And the four choices that are honoured are all on screen.
    expect(unitRadio('Bytes')).toBeDefined();
    expect(themeRadio('System')).toBeDefined();
    expect(languageRadio('System')).toBeDefined();
    expect(startRadio('Analyze')).toBeDefined();
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

    await waitFor(() =>
      expect(updateMock).toHaveBeenCalledWith('bytes', 'history', 'system', 'system'),
    );
    // Both words travel together: the document is one file, and a half-stored pair is not a setting.
    expect(screen.getByText('Preferences saved.')).toBeDefined();
    expect(unitRadio('Bytes').checked).toBe(true);
  });

  it('shares one unit with the page that shows a table', async () => {
    render(<App />);
    fireEvent.click(await screen.findByRole('button', { name: 'History page' }));
    const switchOnHistory = await screen.findByRole('group', { name: 'Size units' });
    fireEvent.click(within(switchOnHistory).getByRole('radio', { name: 'KiB' }) as HTMLInputElement);
    await waitFor(() => expect(updateMock).toHaveBeenCalledWith('kib', 'analyze', 'system', 'system'));

    fireEvent.click(screen.getByRole('button', { name: 'Settings page' }));
    await screen.findByRole('heading', { level: 1, name: 'Settings' });
    // The same state, not a second copy: choosing it on one surface is what the other lists as current.
    await waitFor(() => expect(unitRadio('KiB').checked).toBe(true));
  });

  it('names a start page chosen here and says when it takes effect', async () => {
    await openSettings();

    fireEvent.click(startRadio('History'));

    await waitFor(() =>
      expect(updateMock).toHaveBeenCalledWith('bytes', 'history', 'system', 'system'),
    );
    expect(startRadio('History').checked).toBe(true);
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
    preferencesMock.mockResolvedValue(
      ok(prefs({ sizeUnit: 'kib', startupPage: 'history', theme: 'dark' })),
    );
    await openSettings();
    await waitFor(() => expect(unitRadio('KiB').checked).toBe(true));

    fireEvent.click(screen.getByRole('button', { name: 'Reset application preferences' }));
    // The question is on screen and nothing has been sent yet. It names four settings and lists the
    // theme and the language among them, because a confirmation that under-promises what it is about
    // to discard is the copy defect this round would otherwise have shipped.
    await screen.findByRole('dialog', { name: 'Reset all four settings to their defaults?' });
    expect(
      screen.getByText(
        'Size units go back to bytes, the theme goes back to System, the language goes back to System, and the start page goes back to Analyze.',
      ),
    ).toBeDefined();
    expect(resetMock).not.toHaveBeenCalled();
    expect(document.activeElement?.textContent).toBe('Reset preferences');

    fireEvent.click(screen.getByRole('button', { name: 'Reset preferences' }));

    await waitFor(() => expect(resetMock).toHaveBeenCalledTimes(1));
    expect(unitRadio('Bytes').checked).toBe(true);
    expect(startRadio('Analyze').checked).toBe(true);
    expect(themeRadio('System').checked).toBe(true);
    expect(languageRadio('System').checked).toBe(true);
    expect(
      screen.getByText('Preferences reset to this version’s defaults.'),
    ).toBeDefined();
    expect(
      screen.queryByRole('dialog', { name: 'Reset all four settings to their defaults?' }),
    ).toBeNull();
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
      'input',
      'input',
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
      // The palette and the language are separate groups, and "System" appears in both because it
      // means the same thing in both: follow the desktop. A group label is what tells the reader
      // which one they are tabbing through, which is why the list above asks for a group.
      'System',
      'Light',
      'Dark',
      'System',
      '简体中文',
      'English',
      'Analyze',
      'History',
      'Reset application preferences',
    ]);
    // No control hides itself behind a pointer-only affordance, and none is a div wearing a role.
    for (const element of controls) {
      expect(element.tabIndex).toBeGreaterThanOrEqual(0);
    }
    expect(document.querySelectorAll('main [role=button]')).toHaveLength(0);
  });
});

/**
 * Make the window believe the desktop is in this palette.
 *
 * `window.matchMedia` is what `ADR-0031` D4 reads, and a test that leaves it to the host cannot say
 * which way the platform answered. The returned `change` is the OS flipping while the window is
 * open, which is the half of the contract a cold-start test cannot reach.
 */
function platformIs(dark: boolean) {
  const listeners = new Set<(event: MediaQueryListEvent) => void>();
  const query = {
    matches: dark,
    media: '(prefers-color-scheme: dark)',
    addEventListener: (_type: string, callback: (event: MediaQueryListEvent) => void) => {
      listeners.add(callback);
    },
    removeEventListener: (_type: string, callback: (event: MediaQueryListEvent) => void) => {
      listeners.delete(callback);
    },
  };
  vi.stubGlobal('matchMedia', () => query);
  return {
    change(next: boolean) {
      query.matches = next;
      for (const callback of listeners) {
        callback({ matches: next, media: query.media } as MediaQueryListEvent);
      }
    },
  };
}

/** The attribute is the whole mechanism: its presence is dark, its absence is light. */
function paintedPalette(): string | null {
  return document.documentElement.getAttribute('data-fs-theme');
}

describe('the theme', () => {
  afterEach(() => {
    document.documentElement.removeAttribute('data-fs-theme');
    vi.unstubAllGlobals();
  });

  it('paints the palette the document already holds, before anything is clicked', async () => {
    platformIs(false);
    preferencesMock.mockResolvedValue(ok(prefs({ theme: 'dark', recovery: 'read' })));
    await openSettings();

    expect(paintedPalette()).toBe('dark');
    expect(themeRadio('Dark').checked).toBe(true);
    expect(themeRadio('System').checked).toBe(false);
  });

  it('follows the desktop while the choice is System', () => {
    platformIs(false);
    render(<App />);
    expect(paintedPalette()).toBeNull();
  });

  it('keeps following after the desktop changes, and says what it is following', async () => {
    const platform = platformIs(false);
    await openSettings();
    expect(paintedPalette()).toBeNull();

    platform.change(true);
    await waitFor(() => expect(paintedPalette()).toBe('dark'));
    expect(themeRadio('System').checked).toBe(true);
    // The sentence names the desktop's answer rather than the window's, so a reader who sets the
    // OS back to light sees the claim change, and is not told a thing about this window that is
    // really a thing about their desktop.
    expect(
      await screen.findByText('System is following your desktop, which is set to dark.'),
    ).toBeDefined();
  });

  it('says so when the desktop cannot be read, instead of calling the fallback System', async () => {
    vi.stubGlobal('matchMedia', undefined);
    await openSettings();

    expect(paintedPalette()).toBeNull();
    expect(
      await screen.findByText(
        'System is set to follow your desktop, but this window cannot read that setting, so it is light. Choosing Light or Dark overrides it.',
      ),
    ).toBeDefined();
  });

  it('stops following the desktop the moment the reader names a palette', async () => {
    const platform = platformIs(false);
    await openSettings();

    fireEvent.click(themeRadio('Light'));
    await waitFor(() => expect(updateMock).toHaveBeenCalled());
    expect(paintedPalette()).toBeNull();

    platform.change(true);
    await waitFor(() => expect(themeRadio('Light').checked).toBe(true));
    // `light` was chosen, so the desktop going dark is not this window's business. The stored word
    // is still `light`, which is the part a restart has to honour.
    expect(paintedPalette()).toBeNull();
    const lastCall = updateMock.mock.calls.at(-1);
    expect(lastCall?.[2]).toBe('light');
  });

  it('stores the theme beside the other three and claims it only when the shell agrees', async () => {
    platformIs(false);
    await openSettings();

    fireEvent.click(themeRadio('Dark'));

    await waitFor(() =>
      expect(updateMock).toHaveBeenCalledWith('bytes', 'analyze', 'dark', 'system'),
    );
    expect(paintedPalette()).toBe('dark');
    expect(await screen.findByText('Preferences saved.')).toBeDefined();
  });

  it('says a refused theme did not stick, without pretending the window reverted', async () => {
    platformIs(false);
    updateMock.mockResolvedValue({ ok: false, envelope: SAVE_REFUSED });
    await openSettings();

    fireEvent.click(themeRadio('Dark'));

    await waitFor(() => expect(screen.getByText('The preferences were not stored')).toBeDefined());
    // The palette on screen is the one the reader asked for, and the page says the save failed. The
    // alternative - snapping the window back under their hand - would report a store the shell
    // never accepted. This is the same policy the unit switch already runs on.
    expect(paintedPalette()).toBe('dark');
    expect(screen.queryByText('Preferences saved.')).toBeNull();
  });

  it('comes back with the rest of the defaults on a reset', async () => {
    const platform = platformIs(true);
    preferencesMock.mockResolvedValue(ok(prefs({ theme: 'dark', recovery: 'read' })));
    updateMock.mockImplementation(
      async (sizeUnit: string, startupPage: string, theme: string, language: string) =>
        ok(prefs({ sizeUnit, startupPage, theme, language })),
    );
    resetMock.mockResolvedValue(ok(prefs()));
    await openSettings();
    expect(paintedPalette()).toBe('dark');

    fireEvent.click(screen.getByRole('button', { name: 'Reset application preferences' }));
    fireEvent.click(screen.getByRole('button', { name: 'Reset preferences' }));

    await waitFor(() => expect(paintedPalette()).toBe('dark'));
    // Reset stores `system`; with the desktop dark, `system` still paints dark - which is the
    // point: the choice went back to following, not to light.
    expect(themeRadio('System').checked).toBe(true);
    platform.change(false);
    await waitFor(() => expect(paintedPalette()).toBeNull());
  });
});

/**
 * What the desktop reports, as the only thing the window can ask it.
 *
 * `navigator.languages` is the whole host interface here — no command, no permission, no host path
 * (`AGENTS.md` 7). It is set rather than left to the runner because a test that resolved from an
 * unset property would pass or fail depending on the machine running it, which is the opposite of
 * what a resolver is for.
 */
function desktopSpeaks(languages: readonly string[] | undefined) {
  Object.defineProperty(window.navigator, 'languages', {
    value: languages,
    configurable: true,
  });
  Object.defineProperty(window.navigator, 'language', {
    value: languages === undefined || languages.length === 0 ? '' : (languages[0] ?? ''),
    configurable: true,
  });
}

/** The language a document declares, which is what a screen reader picks its voice from. */
function declaredLanguage(): string | null {
  return document.documentElement.getAttribute('lang');
}

describe('the language', () => {
  afterEach(() => {
    document.documentElement.removeAttribute('lang');
    // Delete the own property so the prototype's own answer comes back, rather than restoring a
    // hard-coded guess about what the next test's host will say.
    delete (window.navigator as unknown as Record<string, unknown>)['languages'];
    delete (window.navigator as unknown as Record<string, unknown>)['language'];
  });

  it('opens in the language the document already holds', async () => {
    desktopSpeaks(['en-US']);
    preferencesMock.mockResolvedValue(ok(prefs({ language: 'zh-CN', recovery: 'read' })));
    render(<App />);

    // The nav is in the stored catalog too, so the entry is named in Chinese from the first frame —
    // a shell that stayed English around a Chinese page would be two products in one window.
    fireEvent.click(await screen.findByRole('button', { name: '设置 页面' }));
    expect(await screen.findByRole('heading', { level: 1, name: '设置' })).toBeDefined();
    expect(declaredLanguage()).toBe('zh-CN');
  });

  it('follows the desktop while the choice is System, and says which language it followed', async () => {
    desktopSpeaks(['zh-CN', 'en-US']);
    preferencesMock.mockResolvedValue(ok(prefs({ language: 'system', recovery: 'read' })));
    render(<App />);
    fireEvent.click(await screen.findByRole('button', { name: '设置 页面' }));
    await screen.findByRole('heading', { level: 1, name: '设置' });

    expect(
      (
        within(screen.getByRole('radiogroup', { name: '语言' })).getByRole('radio', {
          // The three language names are not translated, in either catalog: a reader who cannot parse
          // the screen in front of them has to recognise the way out. Only the group is localised.
          name: 'System',
        }) as HTMLInputElement
      ).checked,
    ).toBe(true);
    expect(declaredLanguage()).toBe('zh-CN');
    // The sentence names the desktop's own answer, not the window's, and it is read out of the
    // Chinese catalog because that is the catalog on screen.
    expect(screen.getByText('界面跟随你的桌面语言，当前为简体中文。')).toBeDefined();
  });

  it('keeps a named language whatever the desktop says', async () => {
    desktopSpeaks(['zh-CN']);
    preferencesMock.mockResolvedValue(ok(prefs({ language: 'en', recovery: 'read' })));
    await openSettings();

    expect(languageRadio('English').checked).toBe(true);
    expect(declaredLanguage()).toBe('en');
    expect(
      screen.getByText(
        'The interface is English because you chose it, and that is kept whatever your desktop says.',
      ),
    ).toBeDefined();
  });

  it('does not call a German desktop English', async () => {
    desktopSpeaks(['de-DE']);
    preferencesMock.mockResolvedValue(ok(prefs({ language: 'system', recovery: 'read' })));
    await openSettings();

    // English is what the window runs in, and that is a fact about this build's catalog set, not a
    // claim about the reader's machine. The false sentence would be "your desktop language is
    // English"; the true one says which of the two it is.
    expect(declaredLanguage()).toBe('en');
    expect(screen.getByText(/no catalog for, so the interface is in English/)).toBeDefined();
    expect(screen.queryByText(/follows your desktop language, which is English/)).toBeNull();
  });

  it('says so when the desktop cannot be read at all', async () => {
    desktopSpeaks(undefined);
    preferencesMock.mockResolvedValue(ok(prefs({ language: 'system', recovery: 'read' })));
    await openSettings();

    expect(declaredLanguage()).toBe('en');
    expect(
      screen.getByText(
        'System is set to follow your desktop, but this window cannot read that setting, so it is English.',
      ),
    ).toBeDefined();
  });

  it('switches the words under the reader’s hand and stores the fourth word', async () => {
    desktopSpeaks(['en-US']);
    await openSettings();

    fireEvent.click(languageRadio('简体中文'));

    await waitFor(() =>
      expect(updateMock).toHaveBeenCalledWith('bytes', 'analyze', 'system', 'zh-CN'),
    );
    // This page is itself rendered from the catalog, so the click is visible on the control that took
    // it. A settings screen that stayed in English while reporting a language change would be showing
    // a state it is not in.
    await waitFor(() =>
      expect(screen.getByRole('heading', { level: 1, name: '设置' })).toBeDefined(),
    );
    expect(declaredLanguage()).toBe('zh-CN');
    expect(await screen.findByText('设置已保存。')).toBeDefined();
    // The three choices keep their own names in both catalogs: a reader who cannot parse the
    // interface on screen still has to be able to leave it.
    const group = screen.getByRole('radiogroup', { name: '语言' });
    expect(
      within(group)
        .getAllByRole('radio')
        .map((input) => (input as HTMLInputElement).labels?.[0]?.textContent),
    ).toEqual(['System', '简体中文', 'English']);
    expect(within(group).getByRole('radio', { name: '简体中文' })).toBeDefined();
  });

  it('says a refused language did not stick, without pulling the words back', async () => {
    desktopSpeaks(['en-US']);
    updateMock.mockResolvedValue({ ok: false, envelope: SAVE_REFUSED });
    await openSettings();

    fireEvent.click(languageRadio('简体中文'));

    await waitFor(() => expect(screen.getByText('设置没有保存')).toBeDefined());
    expect(declaredLanguage()).toBe('zh-CN');
    expect(screen.queryByText('设置已保存。')).toBeNull();
    // The words on screen are the ones the reader asked for; the stored document is what did not
    // change. The same policy the unit and the theme already run on, for the same reason.
    expect(await screen.findByRole('alert')).toBeDefined();
  });

  it('leaves the machine vocabulary alone in either catalog', async () => {
    desktopSpeaks(['en-US']);
    updateMock.mockResolvedValue({ ok: false, envelope: SAVE_REFUSED });
    await openSettings();

    fireEvent.click(languageRadio('简体中文'));
    await waitFor(() => expect(screen.getByRole('heading', { level: 1, name: '设置' })).toBeDefined());

    const words = document.body.textContent ?? '';
    // A product name, a file name and an error code are how this tool and its failures are
    // identified. Translating any of them in one language would make the two languages two different
    // products (`ADR-0032` D5). The code is the one Rust emitted, in a panel whose every label around
    // it has just become Chinese.
    for (const kept of ['FirmwareSight', 'settings.json', 'ERR-PREFS-8002']) {
      expect(words).toContain(kept);
    }
    expect(words).toContain('设置没有保存');
  });

  it('comes back with the rest of the defaults on a reset', async () => {
    desktopSpeaks(['zh-CN']);
    preferencesMock.mockResolvedValue(ok(prefs({ language: 'zh-CN', recovery: 'read' })));
    resetMock.mockResolvedValue(ok(prefs({ language: 'system' })));
    render(<App />);
    fireEvent.click(await screen.findByRole('button', { name: '设置 页面' }));
    await screen.findByRole('heading', { level: 1, name: '设置' });

    fireEvent.click(screen.getByRole('button', { name: '重置应用设置' }));
    fireEvent.click(screen.getByRole('button', { name: '重置设置' }));

    await waitFor(() => expect(resetMock).toHaveBeenCalledTimes(1));
    // Reset stores `system`; on a Chinese desktop `system` still answers Chinese — which is the point.
    // A reset that silently switched the interface to English would be a fifth thing the reader never
    // asked for, and the confirm sentence promises only the four it names.
    expect(declaredLanguage()).toBe('zh-CN');
    expect(
      (
        within(screen.getByRole('radiogroup', { name: '语言' })).getByRole('radio', {
          name: 'System',
        }) as HTMLInputElement
      ).checked,
    ).toBe(true);
    expect(await screen.findByText('设置已恢复为本版本的默认值。')).toBeDefined();
  });
});
