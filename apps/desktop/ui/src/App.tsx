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
 * Six pieces of state live here because they outlive a page switch:
 *
 * - the size unit, which is one answer to one question and must not differ between two tables the same
 *   reader is comparing (US-001, prompt §39);
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

import { useEffect, useState } from 'react';

import styles from './App.module.css';
import { Analyze } from './Analyze';
import { Compare, EMPTY_COMPARISON, type ComparisonSession } from './Compare';
import type { SizeUnit } from './format';
import { Help } from './Help';
import { History } from './History';
import type { AnalysisSummaryDto, GateRunDto, MainWindowPage, ProjectContextDto, SelectionDto } from './ipc/types';
import { setWindowTitle } from './ipc/bridge';
import { Overview } from './Overview';
import { Release } from './Release';
import { cx } from './styles/classnames';
import { TopBar } from './components/TopBar';

type Page = 'overview' | 'analyze' | 'compare' | 'release' | 'history' | 'help';

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
}[] = [{ key: 'help', label: 'Help', title: 'Help' }];

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
            {PAGES.map((entry) => navItem(page, setPage, entry))}
          </nav>
          <nav className={styles['nav']} aria-label="Help and about">
            {AUXILIARY.map((entry) => navItem(page, setPage, entry))}
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
                setPage(target);
              }}
            />
          ) : page === 'analyze' ? (
            <Analyze
              unit={unit}
              onUnitChange={setUnit}
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
              onUnitChange={setUnit}
              lastAnalyzedSnapshotId={lastGood?.identity.snapshotId ?? null}
              session={comparison}
              onSessionChange={setComparison}
              onGoToAnalyze={() => {
                setPage('analyze');
              }}
            />
          ) : page === 'release' ? (
            <Release
              unit={unit}
              onUnitChange={setUnit}
              lastAnalyzedSnapshotId={lastGood?.identity.snapshotId ?? null}
              project={project}
              onProjectChange={setProject}
              run={gateRun}
              onRunChange={setGateRun}
              onGoToAnalyze={() => {
                setPage('analyze');
              }}
            />
          ) : page === 'history' ? (
            <History unit={unit} onUnitChange={setUnit} />
          ) : (
            <Help />
          )}
        </div>
      </div>
    </div>
  );
}
