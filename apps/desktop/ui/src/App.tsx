/**
 * The desktop shell: one product bar, one navigation rail and one page at a time.
 *
 * U1 adds the Overview page and moves the brand into a product bar that states what the workspace is
 * holding, so the rail now lists five workflow pages in the order the product runs them - Overview,
 * Analyze, Compare, Release Gate, Bundle & History - plus Help under its own heading because it is not a
 * stage of the workflow and should not read as one. A stage that does not exist still gets no entry and
 * no words anywhere on the screen: the rail is a list of what this build does, never a preview of what
 * it might.
 *
 * The rail also used to carry the brand. It moved to `TopBar` because the reference screens put the
 * product identity, the project chip and the local-first promise on one line above everything, and a
 * promise that lives only in Help is the least visible important sentence in the product.
 *
 * The page still cannot write its own window title. `tauri.conf.json` can only carry a static title, and
 * giving the WebView `core:window:allow-set-title` would be both a capability change (`AGENTS.md` 9
 * reserves that for a human decision) and a way for a file name to reach a window property. So the page
 * moves, this shell tells the shell which page moved, and Rust composes the text from a closed set
 * (`MainWindowPage`).
 *
 * The state that outlives a page switch lives here, because a page is unmounted on every navigation:
 *
 * - the size unit, which is one answer to one question and must not differ between two tables the same
 *   reader is comparing (US-001, prompt §39); and with it the application's own preferences, which are
 *   where that unit is remembered between starts (`NEXT-02`) - the shell holds the stored document, this
 *   shell only ever asks it for the two words, and no workflow page is given a path to write one;
 * - what Analyze currently holds, so navigating to Compare and back does not make the session forget the
 *   build it just analyzed;
 * - the last summary Analyze proved, which Compare and Release are allowed to *prefer* as their choice -
 *   the build the reader most recently looked at - but never act on their own initiative (prompt §18),
 *   and which the Overview page reads as the only analysis this session can honestly summarize;
 * - the handle that earned that summary, which Overview receives too. It is the other half of the same fact:
 *   a remembered analysis cannot be told apart from a stale one without it, and U1P-R1 found the Overview
 *   page answering a shipping question about a build the gate had never looked at because it could not ask
 *   whether the run and the analysis were about the same snapshot;
 * - the loaded project policy plus the last Gate run, because a Release record that vanishes when the
 *   reader checks one section in Compare would leave them with nothing to accept a review against
 *   (prompt §49), and because the Overview's ship question is meaningless without the run it came from;
 * - the comparison the reader actually asked for, which is the same argument one level down: U1P-R3 found
 *   Compare showing two named builds and an empty page after a trip to Overview, because the diff lived in
 *   the page and the page is unmounted on every navigation. It is held here as a pair plus a summary - the
 *   shell's own session facts, not a persisted record, because `diffId` is an in-process handle;
 * - and whether the first-use panel has been hidden, which is one deliberate act of the reader and must
 *   survive a trip to History and back without coming back on its own (prompt §13).
 */

import { useEffect, useRef, useState } from 'react';

import styles from './App.module.css';
import { Analyze } from './Analyze';
import { Compare, EMPTY_COMPARISON, type ComparisonSession } from './Compare';
import type { SizeUnit } from './format';
import { Help } from './Help';
import { History } from './History';
import type {
  AnalysisSummaryDto,
  BaselinePinDto,
  ErrorEnvelopeDto,
  GateRunDto,
  MainWindowPage,
  PreferencesDto,
  ProjectContextDto,
  SelectionDto,
} from './ipc/types';
import {
  getApplicationPreferences,
  getBaselinePin,
  resetApplicationPreferences,
  setWindowTitle,
  updateApplicationPreferences,
} from './ipc/bridge';
import { Overview } from './Overview';
import { Release } from './Release';
import { Settings } from './Settings';
import { cx } from './styles/classnames';
import {
  THEME_CHOICES,
  applyTheme,
  asTheme,
  platformTheme,
  resolveTheme,
  type ResolvedTheme,
  type Theme,
} from './theme';
import {
  LANGUAGE_CHOICES,
  asLanguage,
  documentLanguage,
  pickPlatformLanguage,
  platformLanguages,
  resolveLanguage,
  type Language,
} from './i18n/language';
import { I18nProvider, translate, type Translate } from './i18n';
import { TopBar } from './components/TopBar';
import { Rail, type RailGroup } from './components/Rail';
import type { IconName } from './components/Icon';
import { ErrorPanel } from './components/ErrorPanel';
import { Button } from './components/Button';
import { CommandPalette, type PaletteCommand } from './components/CommandPalette';
import { acceptDroppedFiles, subscribeToDrops } from './ipc/dropEvents';

type Page = 'overview' | 'analyze' | 'compare' | 'release' | 'history' | 'help' | 'settings';

/**
 * The workflow pages, in the order the product flow runs them.
 *
 * U1 renames two labels to the names the pages give themselves on screen: the Release page's own heading
 * already read "Release Gate" (`Release.tsx:311`) while the rail called it "Release", and the History
 * page holds bundles as well as runs. The `title` values stay the Rust enum's words, because those are
 * the wire form of `set_window_title` and a clearer label is not a reason to rename a command argument.
 */
const PAGES: readonly {
  readonly key: Page;
  readonly label: string;
  readonly title: MainWindowPage;
  readonly icon: IconName;
}[] = [
  { key: 'overview', label: 'Overview', title: 'Overview', icon: 'overview' },
  { key: 'analyze', label: 'Analyze', title: 'Analyze', icon: 'analyze' },
  { key: 'compare', label: 'Compare', title: 'Compare', icon: 'compare' },
  { key: 'release', label: 'Release', title: 'Release', icon: 'release' },
  { key: 'history', label: 'History', title: 'History', icon: 'history' },
];

/** The surfaces that describe the product rather than move work along. */
const AUXILIARY: readonly {
  readonly key: Page;
  readonly label: string;
  readonly title: MainWindowPage;
  readonly icon: IconName;
}[] = [
  { key: 'help', label: 'Help', title: 'Help', icon: 'help' },
  { key: 'settings', label: 'Settings', title: 'Settings', icon: 'settings' },
];

/**
 * One rail entry per page. The catalog owns the words; `entry.label` stays because it is what the Rust
 * `title` mapping and the existing tests refer to, and a label that drifted from the catalog would be a
 * second name.
 */
function railGroups(t: Translate): readonly RailGroup<Page>[] {
  const entry = (item: (typeof PAGES)[number]) => ({
    key: item.key,
    label: t('common', `page.${item.key}`),
    icon: item.icon,
  });
  return [
    { label: t('common', 'nav.pages'), entries: PAGES.map(entry) },
    { label: t('common', 'nav.helpAndAbout'), entries: AUXILIARY.map(entry) },
  ];
}

/**
 * The unit word the shell sent, as the one this window can apply.
 *
 * The shell only ever answers with `bytes` or `kib` - Rust reads the closed set and refuses anything
 * else - so this is a narrowing for TypeScript rather than a guess about a value that could arrive.
 * Anything unrecognized becomes the default instead of a third option, which is what the store itself
 * does with a damaged value.
 */
function asSizeUnit(word: string): SizeUnit {
  return word === 'kib' ? 'kib' : 'bytes';
}

export function App() {
  /**
   * The session still opens on Analyze, not on the new Overview page.
   *
   * The reference screen lands a reader on Overview, and the honest reason this build does not is that
   * Overview summarizes an analysis and a gate run, so on a first run it is an empty state wearing the
   * product's front door - while Analyze is where §13's first-use guidance, the artifact chooser and the
   * only way to produce a fact all live. A reader who has never analyzed anything gets more from the page
   * that makes the first fact than from the page that reports on it.
   */
  const [page, setPage] = useState<Page>('analyze');
  const [unit, setUnit] = useState<SizeUnit>('bytes');
  /** The stored choice, not the palette: `system` stays `system` until the platform answers. */
  const [theme, setTheme] = useState<Theme>('system');

  /**
   * The stored language choice, and the tags the platform reported.
   *
   * Unlike the palette, a desktop language has no change signal a page can subscribe to — there is
   * no `navigator.languages` event to listen for. So the honest contract, stated in `ADR-0032` D8
   * rather than implied: the platform is asked when this window mounts and when the choice returns
   * to `system`, and a person who changes their desktop language sees the interface follow at the
   * next start. `system` is not claimed to be live for language when it is live for colour.
   */
  const [language, setLanguage] = useState<Language>('system');
  const [platformTags, setPlatformTags] = useState<readonly string[]>(() =>
    platformLanguages(typeof navigator === 'undefined' ? undefined : navigator),
  );
  /** What the desktop says right now, re-read while the window is open. `null` says it cannot know. */
  const [platform, setPlatform] = useState<ResolvedTheme | null>(platformTheme);
  const [selection, setSelection] = useState<SelectionDto | null>(null);
  const [lastGood, setLastGood] = useState<AnalysisSummaryDto | null>(null);
  /** Which selection produced `lastGood`. It moves with the summary and never on its own. */
  const [analyzedSelectionId, setAnalyzedSelectionId] = useState<string | null>(null);
  const [project, setProject] = useState<ProjectContextDto | null>(null);
  const [gateRun, setGateRun] = useState<GateRunDto | null>(null);
  /**
   * The pair the reader named and the comparison that pair produced, kept together on purpose: they are
   * one fact with two halves, and separating them is how a page ends up showing a diff for a build the
   * selectors no longer point at.
   */
  const [comparison, setComparison] = useState<ComparisonSession>(EMPTY_COMPARISON);
  /**
   * The build this project measures against (R5 / D3). Read once at start and moved only by a pin or an
   * unpin the shell confirmed, so Compare's default base and History's pin line can never disagree.
   */
  const [baselinePin, setBaselinePinState] = useState<BaselinePinDto | null>(null);
  /** A failed read of the pin: the pin is then Unknown, never "none" (R5 review S2-1, AGENTS.md 8). */
  const [pinReadError, setPinReadError] = useState<ErrorEnvelopeDto | null>(null);
  /** Moves on every pin or unpin, so a read that started earlier cannot overwrite a newer answer. */
  const pinVersion = useRef(0);
  const setBaselinePin = (pin: BaselinePinDto | null) => {
    pinVersion.current += 1;
    setPinReadError(null);
    setBaselinePinState(pin);
  };
  const readPin = () => {
    const version = pinVersion.current;
    void getBaselinePin().then((outcome) => {
      if (version !== pinVersion.current) {
        return;
      }
      if (outcome.ok) {
        setPinReadError(null);
        setBaselinePinState(outcome.value);
      } else {
        setPinReadError(outcome.envelope);
      }
    });
  };
  const readPinRef = useRef(readPin);
  readPinRef.current = readPin;
  /** A comparison another page asked for, which Compare runs once when it mounts (D-R5-4). */
  const [autoCompare, setAutoCompare] = useState(false);
  /** The command palette (spec §3.3), and a firmware chooser the shell asked Analyze to open once. */
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [chooseRequested, setChooseRequested] = useState(false);
  /** Hidden for the session by the reader, not by the application deciding they have read it. */
  const [gettingStartedHidden, setGettingStartedHidden] = useState(false);
  /**
   * This application's own settings, as the shell reported them. `null` until the shell answers, which
   * is why every page still opens on a stated default rather than waiting on a reply.
   */
  const [preferences, setPreferences] = useState<PreferencesDto | null>(null);
  /** A refusal to store, shown rather than swallowed: a page that looks saved is the failure here. */
  const [preferencesError, setPreferencesError] = useState<ErrorEnvelopeDto | null>(null);
  /**
   * What the shell agreed to, held as a catalog key rather than as a sentence.
   *
   * A note kept as text would be written in the language the window was in when the save landed and
   * stay there: switch to Chinese one click later and the page would report `Preferences saved.` under
   * Chinese headings, which is two products in one window. Held as a key, it is read out of whatever
   * catalog the reader is looking at.
   */
  const [preferencesNote, setPreferencesNote] = useState<'savedNote' | 'resetNote' | null>(null);
  /**
   * Whether the reader has named a page themselves since the window opened.
   *
   * The stored start page is applied once, from the shell's first answer. Without this, a reader who
   * clicked Compare while that read was in flight would be carried off it a moment later by a
   * preference they set yesterday - the page they chose is the newer fact.
   */
  const readerChosePage = useRef(false);

  /**
   * Spec §3.3: after the reader switches pages, focus lands on the new page's heading, so a screen reader
   * announces where they are and Tab continues from the top of that page. Only a page the reader chose
   * moves focus; the startup page and a drop (which keeps focus on its own notice) do not.
   */
  // The page the reader asked for, until it is on screen. Opening the page already showing changes nothing
  // and must not leave a request behind for a later, unrelated page change (closeout review S2).
  const focusHeading = useRef<Page | null>(null);

  const open = (target: Page) => {
    readerChosePage.current = true;
    focusHeading.current = target === page ? null : target;
    setPage(target);
  };

  useEffect(() => {
    const requested = focusHeading.current;
    focusHeading.current = null;
    if (requested !== page) {
      return;
    }
    const heading = document.querySelector<HTMLElement>('main h1');
    if (heading !== null) {
      heading.tabIndex = -1;
      heading.focus();
    }
  }, [page]);

  /** The skip link (spec §3.3): past the product bar and the rail, to the page being read. */
  const skipToMain = (event: { preventDefault: () => void }) => {
    event.preventDefault();
    const target =
      document.querySelector<HTMLElement>('main h1') ?? document.querySelector<HTMLElement>('main');
    if (target !== null) {
      target.tabIndex = -1;
      target.focus();
    }
  };

  /**
   * Drag and drop (M1). The window keeps the dropped paths in Rust; this side hears names, a reason word or
   * a single-use token. A successful drop stages the files exactly as the dialog would, opens Analyze and
   * asks it to analyze that selection once, so dropping an ELF ends with its facts on screen.
   */
  const [dragging, setDragging] = useState(false);
  const [dropRefusal, setDropRefusal] = useState<
    'directory' | 'too_many' | 'ambiguous' | 'map_without_artifact' | null
  >(null);
  const [dropError, setDropError] = useState<ErrorEnvelopeDto | null>(null);
  const [autoAnalyzeId, setAutoAnalyzeId] = useState<string | null>(null);
  const selectionRef = useRef<SelectionDto | null>(null);
  selectionRef.current = selection;

  useEffect(
    () =>
      subscribeToDrops((event) => {
        if (event.phase === 'hover') {
          setDragging(true);
          return;
        }
        setDragging(false);
        if (event.phase === 'refused') {
          const reason = event.refusal;
          setDropError(null);
          setDropRefusal(
            reason === 'directory' || reason === 'too_many' || reason === 'ambiguous' ? reason : 'ambiguous',
          );
          return;
        }
        if (event.phase !== 'dropped' || event.token === null) {
          return;
        }
        const mapOnly = event.artifactName === null && event.mapName !== null;
        const current = selectionRef.current;
        if (mapOnly && current === null) {
          setDropError(null);
          setDropRefusal('map_without_artifact');
          return;
        }
        const token = event.token;
        void acceptDroppedFiles(token, mapOnly && current !== null ? current.selectionId : null).then(
          (outcome) => {
            if (!outcome.ok) {
              setDropRefusal(null);
              setDropError(outcome.envelope);
              return;
            }
            setDropRefusal(null);
            setDropError(null);
            setSelection(outcome.value);
            setAutoAnalyzeId(outcome.value.selectionId);
            readerChosePage.current = true;
            setPage('analyze');
          },
        );
      }),
    [],
  );

  // The pinned baseline is read once at start. A failure is kept as its own fact and History offers a
  // retry; a pin or unpin made meanwhile wins over the late answer (R5 review S2-1, S3).
  useEffect(() => {
    readPinRef.current();
  }, []);

  /** Open Compare on a named pair and run it once: the History rows and the palette start comparisons. */
  const startComparison = (baseId: string, targetId: string) => {
    setComparison({ ...EMPTY_COMPARISON, baseId, targetId });
    setAutoCompare(true);
    open('compare');
  };

  /** Open the firmware chooser on Analyze, as if its button had been pressed. */
  const chooseFirmware = () => {
    setChooseRequested(true);
    open('analyze');
  };
  const chooseFirmwareRef = useRef(chooseFirmware);
  chooseFirmwareRef.current = chooseFirmware;

  /**
   * The keyboard model (spec §3.3). `Ctrl+K` and `Ctrl+O` work from anywhere. The single keys - `/`, `[`
   * and `]` - are ignored while a person is typing in a field or a modal dialog is open, so a slash typed
   * into a filter stays a slash. They act on what is on screen through the same controls a mouse uses: the
   * page's filter field and its view tabs.
   */
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      const command = event.ctrlKey || event.metaKey;
      // An open modal (a Sheet, the undocked Inspector, the palette itself) owns the keyboard: no shortcut
      // opens another layer over it or navigates under it (R5 review S2-3).
      if (document.querySelector('[aria-modal="true"]') !== null) {
        return;
      }
      if (command && !event.altKey && (event.key === 'k' || event.key === 'K')) {
        event.preventDefault();
        setPaletteOpen(true);
        return;
      }
      if (command && !event.altKey && (event.key === 'o' || event.key === 'O')) {
        event.preventDefault();
        chooseFirmwareRef.current();
        return;
      }
      if (command || event.altKey || event.defaultPrevented) {
        return;
      }
      const target = event.target;
      const typing =
        target instanceof HTMLElement &&
        (target.isContentEditable || ['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName));
      if (typing) {
        return;
      }
      if (event.key === '/') {
        const field = document.querySelector<HTMLInputElement>('main [data-fs-filter]');
        if (field !== null) {
          event.preventDefault();
          field.focus();
        }
        return;
      }
      if (event.key === '[' || event.key === ']') {
        const tabs = Array.from(document.querySelectorAll<HTMLElement>('main [role="tablist"] [role="tab"]'));
        const current = tabs.findIndex((tab) => tab.getAttribute('aria-selected') === 'true');
        if (tabs.length === 0 || current < 0) {
          return;
        }
        event.preventDefault();
        const next = tabs[(current + (event.key === ']' ? 1 : tabs.length - 1)) % tabs.length];
        next?.click();
        next?.focus();
      }
    };
    document.addEventListener('keydown', onKey);
    return () => {
      document.removeEventListener('keydown', onKey);
    };
  }, []);

  /**
   * Ask the shell once, and let the answer move what it owns.
   *
   * The unit and the start page come from the stored document; the window's own data is untouched by
   * either. A read that fails is shown as a refusal on the Settings page, and the defaults stay in
   * effect - the workflow does not depend on a preference file to open.
   */
  useEffect(() => {
    let cancelled = false;
    void getApplicationPreferences().then((outcome) => {
      if (cancelled) {
        return;
      }
      if (!outcome.ok) {
        setPreferencesError(outcome.envelope);
        return;
      }
      const stored = outcome.value;
      setPreferences(stored);
      setUnit(asSizeUnit(stored.sizeUnit));
      setTheme(asTheme(stored.theme));
      setLanguage(asLanguage(stored.language));
      if (!readerChosePage.current && stored.startupPage === 'history') {
        setPage('history');
      }
    });
    return () => {
      cancelled = true;
    };
  }, []);

  /**
   * Follow the desktop while the window is open, and only on the shell's own question.
   *
   * The listener is added here and removed here, so a `system` window that is closed stops being
   * told, and a window whose preference is `light` or `dark` has the subscription but ignores it —
   * `ADR-0031` D4's "the OS is consulted only while the stored value is system".
   */
  useEffect(() => {
    if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
      return;
    }
    const query = window.matchMedia('(prefers-color-scheme: dark)');
    const onChange = (event: MediaQueryListEvent) => {
      setPlatform(event.matches ? 'dark' : 'light');
    };
    query.addEventListener('change', onChange);
    return () => {
      query.removeEventListener('change', onChange);
    };
  }, []);

  /**
   * The palette on the document, in the same commit that paints it.
   *
   * This runs before the reader sees a frame of the new palette because React applies the DOM
   * attribute in the same pass as the re-render, and every colour in the app comes from the
   * selector this attribute selects. What it cannot promise is the WebView's own first paint before
   * React mounts, which is `color-scheme` in the stylesheet doing that part.
   */
  const resolvedTheme = resolveTheme(theme, platform);
  const resolvedLanguage = resolveLanguage(language, platformTags);
  // The shell needs the words the pages get from the provider, and a component cannot read a context
  // it is the one rendering. `translate` is the same lookup the provider hands down, bound here.
  const navLabel = translate(resolvedLanguage);

  useEffect(() => {
    applyTheme(document.documentElement, resolvedTheme);
  }, [resolvedTheme]);

  /**
   * The language the document declares itself in.
   *
   * `lang` is what a screen reader uses to pick a voice and what a translation tool uses to decide
   * what is already localized, so a Chinese window that declares `en` is a defect the reader meets
   * immediately even though nothing looks wrong.
   */
  useEffect(() => {
    document.documentElement.lang = documentLanguage(resolvedLanguage);
  }, [resolvedLanguage]);

  /** Store all four words together: the document is one file, and a half-stored set is not a setting. */
  const persist = async (
    nextUnit: SizeUnit,
    nextStartPage: string,
    nextTheme: Theme,
    nextLanguage: Language,
  ) => {
    const outcome = await updateApplicationPreferences(
      nextUnit,
      nextStartPage,
      nextTheme,
      nextLanguage,
    );
    if (outcome.ok) {
      setPreferences(outcome.value);
      setPreferencesError(null);
      setPreferencesNote('savedNote');
    } else {
      setPreferencesError(outcome.envelope);
      setPreferencesNote(null);
    }
  };

  /**
   * Change the unit in this window and store the change.
   *
   * The label moves first and the save follows: what a person sees on the screen is what the window is
   * doing, and a failed save says so on the page instead of leaving a table that quietly reverted.
   */
  const chooseUnit = (next: SizeUnit) => {
    setUnit(next);
    void persist(next, preferences?.startupPage ?? 'analyze', theme, language);
  };

  const chooseStartPage = (word: string) => {
    void persist(unit, word, theme, language);
  };

  /**
   * Change the palette in this window and store the change.
   *
   * The theme is applied from state, so the window changes colour on the click and the save follows,
   * the same order the unit uses. A refused save leaves the window in the palette it now shows and
   * says the choice did not stick — the alternative is a page that reverts under the reader's hand.
   */
  const chooseTheme = (next: Theme) => {
    setTheme(next);
    void persist(unit, preferences?.startupPage ?? 'analyze', next, language);
  };

  /**
   * Change the interface language and store the change.
   *
   * Returning to `system` re-asks the platform, which is the only moment a re-read is possible: no
   * browser signal reports a desktop language change while the window is open. A named language
   * ignores the platform entirely, exactly as a named palette does.
   */
  const chooseLanguage = (next: Language) => {
    setLanguage(next);
    if (next === 'system') {
      setPlatformTags(
        platformLanguages(typeof navigator === 'undefined' ? undefined : navigator),
      );
    }
    void persist(unit, preferences?.startupPage ?? 'analyze', theme, next);
  };

  const resetPreferences = async () => {
    const outcome = await resetApplicationPreferences();
    if (outcome.ok) {
      setPreferences(outcome.value);
      setPreferencesError(null);
      setPreferencesNote('resetNote');
      // Every field the shell just wrote is re-applied here, not just the ones this page happens to
      // render first. The round's own test caught the shape of this bug: the unit was restored and
      // the theme was not, so a reset left the document on `system` while the window stayed dark
      // until the next start - a page reporting a state it was no longer in.
      setUnit(asSizeUnit(outcome.value.sizeUnit));
      setTheme(asTheme(outcome.value.theme));
      setLanguage(asLanguage(outcome.value.language));
    } else {
      setPreferencesError(outcome.envelope);
      setPreferencesNote(null);
    }
  };

  /**
   * The page names the window, in the shell that owns the page.
   *
   * One rule for all six entries, so a page that fails to load a single fact still cannot leave the
   * previous page's title behind. `setWindowTitle` resolves to an outcome and never throws: the title
   * is presentation, and the data on screen does not depend on it.
   */
  useEffect(() => {
    const title = [...PAGES, ...AUXILIARY].find((entry) => entry.key === page)?.title ?? 'Overview';
    void setWindowTitle(title);
  }, [page]);

  /** Every command the palette offers, in the language on screen. */
  const paletteCommands = (): readonly PaletteCommand[] => {
    const t = navLabel;
    const current = lastGood?.identity.snapshotId ?? null;
    const pinReason =
      pinReadError !== null
        ? t('common', 'palette.pinUnknown')
        : baselinePin === null
        ? t('common', 'palette.noPin')
        : current === null
          ? t('common', 'palette.noCurrent')
          : current === baselinePin.snapshotId
            ? t('common', 'palette.samePin')
            : undefined;
    return [
      ...[...PAGES, ...AUXILIARY].map((entry) => ({
        id: `go-${entry.key}`,
        group: t('common', 'palette.groupGo'),
        label: t('common', `page.${entry.key}`),
        run: () => {
          open(entry.key);
        },
      })),
      {
        id: 'choose',
        group: t('common', 'palette.groupAction'),
        label: t('common', 'palette.choose'),
        shortcut: 'Ctrl+O',
        run: chooseFirmware,
      },
      {
        id: 'compare-pinned',
        group: t('common', 'palette.groupAction'),
        label: t('common', 'palette.comparePinned'),
        ...(pinReason === undefined ? {} : { unavailable: pinReason }),
        run: () => {
          if (baselinePin !== null && current !== null) {
            startComparison(baselinePin.snapshotId, current);
          }
        },
      },
      ...(['bytes', 'kib'] as const).map((word) => ({
        id: `unit-${word}`,
        group: t('common', 'palette.groupUnit'),
        label: t('common', word === 'bytes' ? 'palette.unitBytes' : 'palette.unitKib'),
        run: () => {
          chooseUnit(word);
        },
      })),
      ...THEME_CHOICES.map((choice) => ({
        id: `theme-${choice.word}`,
        group: t('common', 'palette.groupTheme'),
        label: t('common', `palette.theme.${choice.word}`),
        run: () => {
          chooseTheme(choice.word);
        },
      })),
      ...LANGUAGE_CHOICES.map((choice) => ({
        id: `language-${choice.word}`,
        group: t('common', 'palette.groupLanguage'),
        label: choice.word === 'system' ? t('common', 'palette.languageSystem') : choice.label,
        run: () => {
          chooseLanguage(choice.word);
        },
      })),
    ];
  };

  return (
    <I18nProvider
      language={language}
      resolved={resolvedLanguage}
      platform={platformTags}
    >
      <div className={styles['app']}>
        <a className={styles['skip']} href="#main" onClick={skipToMain}>
          {navLabel('common', 'shell.skipToMain')}
        </a>
        <TopBar
          project={project?.projectName ?? null}
          command={
            <Button
              variant="ghost"
              ariaHasPopup="dialog"
              onClick={() => {
                setPaletteOpen(true);
              }}
            >
              {navLabel('common', 'palette.open')}
            </Button>
          }
        />
        <CommandPalette
          open={paletteOpen}
          commands={paletteCommands()}
          onClose={() => {
            setPaletteOpen(false);
          }}
        />
        <div className={styles['shell']}>
          <Rail groups={railGroups(navLabel)} current={page} onOpen={open} />

          <div className={cx(styles['workspace'], dragging ? styles['dropping'] : undefined)}>
          {dragging ? (
            <div className={styles['dropTarget']} role="status">
              {navLabel('common', 'drop.target')}
            </div>
          ) : null}
          {dropRefusal === null ? null : (
            <div className={styles['dropNotice']} role="alert">
              <span>{navLabel('common', `drop.refused.${dropRefusal}`)}</span>
              <button type="button" className={styles['dropDismiss']} onClick={() => setDropRefusal(null)}>
                {navLabel('common', 'drop.dismiss')}
              </button>
            </div>
          )}
          {dropError === null ? null : (
            <ErrorPanel
              envelope={dropError}
              label={navLabel('common', 'drop.errorLabel')}
              heading={navLabel('common', 'drop.errorHeading')}
            />
          )}
          {page === 'overview' ? (
            <Overview
              summary={lastGood}
              selection={selection}
              analyzedSelectionId={analyzedSelectionId}
              gateRun={gateRun}
              project={project}
              comparison={comparison.summary}
              unit={unit}
              onOpen={(target) => {
                open(target);
              }}
            />
          ) : page === 'analyze' ? (
            <Analyze
              unit={unit}
              onUnitChange={chooseUnit}
              selection={selection}
              onSelectionChange={setSelection}
              analyzedSelectionId={analyzedSelectionId}
              lastGood={lastGood}
              gettingStartedHidden={gettingStartedHidden}
              onHideGettingStarted={() => {
                setGettingStartedHidden(true);
              }}
              onLastGoodChange={(summary, selectionId) => {
                setLastGood(summary);
                setAnalyzedSelectionId(selectionId);
              }}
              autoAnalyzeId={autoAnalyzeId}
              onAutoAnalyzeStarted={() => {
                setAutoAnalyzeId(null);
              }}
              chooseRequested={chooseRequested}
              onChooseStarted={() => {
                setChooseRequested(false);
              }}
            />
          ) : page === 'compare' ? (
            <Compare
              unit={unit}
              onUnitChange={chooseUnit}
              lastAnalyzedSnapshotId={lastGood?.identity.snapshotId ?? null}
              session={comparison}
              onSessionChange={setComparison}
              pin={baselinePin}
              onPinChange={setBaselinePin}
              autoCompare={autoCompare}
              onAutoCompareStarted={() => {
                setAutoCompare(false);
              }}
              onGoToAnalyze={() => {
                open('analyze');
              }}
            />
          ) : page === 'release' ? (
            <Release
              unit={unit}
              onUnitChange={chooseUnit}
              lastAnalyzedSnapshotId={lastGood?.identity.snapshotId ?? null}
              project={project}
              onProjectChange={setProject}
              run={gateRun}
              onRunChange={setGateRun}
              onGoToAnalyze={() => {
                open('analyze');
              }}
            />
          ) : page === 'history' ? (
            <History
              unit={unit}
              onUnitChange={chooseUnit}
              pin={baselinePin}
              pinReadError={pinReadError}
              onRetryPinRead={readPin}
              onPinChange={setBaselinePin}
              currentSnapshotId={lastGood?.identity.snapshotId ?? null}
              onCompare={startComparison}
            />
          ) : page === 'settings' ? (
            <Settings
              preferences={preferences}
              error={preferencesError}
              note={preferencesNote === null ? null : navLabel('settings', preferencesNote)}
              unit={unit}
              onUnitChange={chooseUnit}
              theme={theme}
              platformPalette={platform}
              onThemeChange={chooseTheme}
              language={language}
              desktopLanguage={pickPlatformLanguage(platformTags)}
              onLanguageChange={chooseLanguage}
              onStartPageChange={chooseStartPage}
              onReset={() => {
                void resetPreferences();
              }}
              onGoToHelp={() => {
                open('help');
              }}
            />
          ) : (
            <Help />
          )}
          </div>
        </div>
      </div>
    </I18nProvider>
  );
}
