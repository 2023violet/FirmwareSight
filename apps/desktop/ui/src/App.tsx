/**
 * The desktop shell: one navigation rail and one page at a time.
 *
 * P5 adds the fourth workflow page History and the local Help surface, so the rail now lists exactly
 * the stages this build implements - four workflow pages in the order prompt §15 names them, plus
 * Help under its own heading because it is not a stage of the workflow and should not read as one. A
 * stage that does not exist still gets no entry and no words anywhere on the screen.
 *
 * The rail also names the window. `tauri.conf.json` can only carry a static title, and giving the
 * WebView `core:window:allow-set-title` to let it write its own title bar would be both a capability
 * change (`AGENTS.md` 9 reserves that for a human decision) and a way for a file name to reach a
 * window property. So the page moves, this shell tells the shell which page moved, and Rust composes
 * the text from a closed set (`MainWindowPage`).
 *
 * Five pieces of state live here because they outlive a page switch:
 *
 * - the size unit, which is one answer to one question and must not differ between two tables the same
 *   reader is comparing (US-001, prompt §39);
 * - what Analyze currently holds, so navigating to Compare and back does not make the session forget the
 *   build it just analyzed;
 * - the last summary Analyze proved, which Compare and Release are allowed to *prefer* as their choice -
 *   the build the reader most recently looked at - but never act on their own initiative (prompt §18);
 * - the loaded project policy plus the last Gate run, because a Release record that vanishes when the
 *   reader checks one section in Compare would leave them with nothing to accept a review against
 *   (prompt §49);
 * - and whether the first-use panel has been hidden, which is one deliberate act of the reader and must
 *   survive a trip to History and back without coming back on its own (prompt §13).
 */

import { useEffect, useState } from 'react';

import styles from './App.module.css';
import { Analyze } from './Analyze';
import { Compare } from './Compare';
import type { SizeUnit } from './format';
import { Help } from './Help';
import { History } from './History';
import type { AnalysisSummaryDto, GateRunDto, MainWindowPage, ProjectContextDto, SelectionDto } from './ipc/types';
import { setWindowTitle } from './ipc/bridge';
import { Release } from './Release';
import { cx } from './styles/classnames';

type Page = 'analyze' | 'compare' | 'release' | 'history' | 'help';

/** The four workflow stages, in the order the product flow runs. */
const PAGES: readonly {
  readonly key: Page;
  readonly label: string;
  readonly title: MainWindowPage;
}[] = [
  { key: 'analyze', label: 'Analyze', title: 'Analyze' },
  { key: 'compare', label: 'Compare', title: 'Compare' },
  { key: 'release', label: 'Release', title: 'Release' },
  { key: 'history', label: 'History', title: 'History' },
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
  const [page, setPage] = useState<Page>('analyze');
  const [unit, setUnit] = useState<SizeUnit>('bytes');
  const [selection, setSelection] = useState<SelectionDto | null>(null);
  const [lastGood, setLastGood] = useState<AnalysisSummaryDto | null>(null);
  /** Which selection produced `lastGood`. It moves with the summary and never on its own. */
  const [analyzedSelectionId, setAnalyzedSelectionId] = useState<string | null>(null);
  const [project, setProject] = useState<ProjectContextDto | null>(null);
  const [gateRun, setGateRun] = useState<GateRunDto | null>(null);
  /** Hidden for the session by the reader, not by the application deciding they have read it. */
  const [gettingStartedHidden, setGettingStartedHidden] = useState(false);

  /**
   * The page names the window, in the shell that owns the page.
   *
   * One rule for all five entries, so a page that fails to load a single fact still cannot leave the
   * previous page's title behind. `setWindowTitle` resolves to an outcome and never throws: the title
   * is presentation, and the data on screen does not depend on it.
   */
  useEffect(() => {
    const title = [...PAGES, ...AUXILIARY].find((entry) => entry.key === page)?.title ?? 'Analyze';
    void setWindowTitle(title);
  }, [page]);

  return (
    <div className={styles['shell']}>
      <div className={styles['rail']}>
        <div className={styles['railSticky']}>
          <span className={styles['brand']}>FirmwareSight</span>
          <nav className={styles['nav']} aria-label="Pages">
            {PAGES.map((entry) => navItem(page, setPage, entry))}
          </nav>
          <nav className={styles['nav']} aria-label="Help and about">
            {AUXILIARY.map((entry) => navItem(page, setPage, entry))}
          </nav>
        </div>
      </div>

      <div className={styles['workspace']}>
        {page === 'analyze' ? (
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
  );
}
