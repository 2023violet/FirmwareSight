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
  ErrorEnvelopeDto,
  GateRunDto,
  MainWindowPage,
  PreferencesDto,
  ProjectContextDto,
  SelectionDto,
} from './ipc/types';
import {
  getApplicationPreferences,
  resetApplicationPreferences,
  setWindowTitle,
  updateApplicationPreferences,
} from './ipc/bridge';
import { Overview } from './Overview';
import { Release } from './Release';
import { Settings } from './Settings';
import { cx } from './styles/classnames';
import { TopBar } from './components/TopBar';

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
}[] = [
  { key: 'overview', label: 'Overview', title: 'Overview' },
  { key: 'analyze', label: 'Analyze', title: 'Analyze' },
  { key: 'compare', label: 'Compare', title: 'Compare' },
  { key: 'release', label: 'Release Gate', title: 'Release' },
  { key: 'history', label: 'Bundle & History', title: 'History' },
];

/** The surfaces that describe the product rather than move work along. */
const AUXILIARY: readonly {
  readonly key: Page;
  readonly label: string;
  readonly title: MainWindowPage;
}[] = [
  { key: 'help', label: 'Help', title: 'Help' },
  { key: 'settings', label: 'Settings', title: 'Settings' },
];

function navItem(
  page: Page,
  setPage: (page: Page) => void,
  entry: { readonly key: Page; readonly label: string },
) {
  const active = page === entry.key;
  return (
    <button
      key={entry.key}
      type="button"
      aria-label={`${entry.label} page`}
      aria-current={active ? 'page' : undefined}
      className={cx(styles['navItem'], active ? styles['navItemActive'] : undefined)}
      onClick={() => {
        setPage(entry.key);
      }}
    >
      {entry.label}
    </button>
  );
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
  /** Hidden for the session by the reader, not by the application deciding they have read it. */
  const [gettingStartedHidden, setGettingStartedHidden] = useState(false);
  /**
   * This application's own settings, as the shell reported them. `null` until the shell answers, which
   * is why every page still opens on a stated default rather than waiting on a reply.
   */
  const [preferences, setPreferences] = useState<PreferencesDto | null>(null);
  /** A refusal to store, shown rather than swallowed: a page that looks saved is the failure here. */
  const [preferencesError, setPreferencesError] = useState<ErrorEnvelopeDto | null>(null);
  const [preferencesNote, setPreferencesNote] = useState<string | null>(null);
  /**
   * Whether the reader has named a page themselves since the window opened.
   *
   * The stored start page is applied once, from the shell's first answer. Without this, a reader who
   * clicked Compare while that read was in flight would be carried off it a moment later by a
   * preference they set yesterday - the page they chose is the newer fact.
   */
  const readerChosePage = useRef(false);

  const open = (target: Page) => {
    readerChosePage.current = true;
    setPage(target);
  };

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
      if (!readerChosePage.current && stored.startupPage === 'history') {
        setPage('history');
      }
    });
    return () => {
      cancelled = true;
    };
  }, []);

  /** Store both words together: the document is one file, and a half-stored pair is not a setting. */
  const persist = async (nextUnit: SizeUnit, nextStartPage: string) => {
    const outcome = await updateApplicationPreferences(nextUnit, nextStartPage);
    if (outcome.ok) {
      setPreferences(outcome.value);
      setPreferencesError(null);
      setPreferencesNote('Preferences saved.');
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
    void persist(next, preferences?.startupPage ?? 'analyze');
  };

  const chooseStartPage = (word: string) => {
    void persist(unit, word);
  };

  const resetPreferences = async () => {
    const outcome = await resetApplicationPreferences();
    if (outcome.ok) {
      setPreferences(outcome.value);
      setPreferencesError(null);
      setPreferencesNote('Preferences reset to this version’s defaults.');
      setUnit(asSizeUnit(outcome.value.sizeUnit));
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

  return (
    <div className={styles['app']}>
      <TopBar project={project?.projectName ?? null} />
      <div className={styles['shell']}>
        <div className={styles['rail']}>
          <nav className={styles['nav']} aria-label="Pages">
            {PAGES.map((entry) => navItem(page, open, entry))}
          </nav>
          <nav className={styles['nav']} aria-label="Help and about">
            {AUXILIARY.map((entry) => navItem(page, open, entry))}
          </nav>
        </div>

        <div className={styles['workspace']}>
          {page === 'overview' ? (
            <Overview
              summary={lastGood}
              selection={selection}
              analyzedSelectionId={analyzedSelectionId}
              gateRun={gateRun}
              project={project}
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
            />
          ) : page === 'compare' ? (
            <Compare
              unit={unit}
              onUnitChange={chooseUnit}
              lastAnalyzedSnapshotId={lastGood?.identity.snapshotId ?? null}
              session={comparison}
              onSessionChange={setComparison}
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
            <History unit={unit} onUnitChange={chooseUnit} />
          ) : page === 'settings' ? (
            <Settings
              preferences={preferences}
              error={preferencesError}
              note={preferencesNote}
              unit={unit}
              onUnitChange={chooseUnit}
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
  );
}
