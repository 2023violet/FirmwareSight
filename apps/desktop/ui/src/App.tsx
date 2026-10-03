/**
 * The desktop shell: one navigation rail and one page at a time.
 *
 * P3 makes Release a real page, so the rail lists exactly the three stages this build implements. A
 * stage that does not exist gets no entry and no words anywhere on the screen - there is no Bundle, no
 * History and no Settings tab, because an empty placeholder you can click is not an honest "not built
 * yet".
 *
 * Four pieces of state live here because they outlive a page switch:
 *
 * - the size unit, which is one answer to one question and must not differ between two tables the same
 *   reader is comparing (US-001, prompt §39);
 * - what Analyze currently holds, so navigating to Compare and back does not make the session forget the
 *   build it just analyzed;
 * - the last summary Analyze proved, which Compare and Release are allowed to *prefer* as their choice -
 *   the build the reader most recently looked at - but never act on their own initiative (prompt §18);
 * - and the loaded project policy plus the last Gate run, because a Release record that vanishes when
 *   the reader checks one section in Compare would leave them with nothing to accept a review against
 *   (prompt §49).
 */

import { useState } from 'react';

import styles from './App.module.css';
import { Analyze } from './Analyze';
import { Compare } from './Compare';
import type { SizeUnit } from './format';
import type { AnalysisSummaryDto, GateRunDto, ProjectContextDto, SelectionDto } from './ipc/types';
import { Release } from './Release';
import { cx } from './styles/classnames';

/** The pages this build has, in the order the rail lists them. */
const PAGES: readonly {
  readonly key: 'analyze' | 'compare' | 'release';
  readonly label: string;
}[] = [
  { key: 'analyze', label: 'Analyze' },
  { key: 'compare', label: 'Compare' },
  { key: 'release', label: 'Release' },
];

export function App() {
  const [page, setPage] = useState<'analyze' | 'compare' | 'release'>('analyze');
  const [unit, setUnit] = useState<SizeUnit>('bytes');
  const [selection, setSelection] = useState<SelectionDto | null>(null);
  const [lastGood, setLastGood] = useState<AnalysisSummaryDto | null>(null);
  /** Which selection produced `lastGood`. It moves with the summary and never on its own. */
  const [analyzedSelectionId, setAnalyzedSelectionId] = useState<string | null>(null);
  const [project, setProject] = useState<ProjectContextDto | null>(null);
  const [gateRun, setGateRun] = useState<GateRunDto | null>(null);

  return (
    <div className={styles['shell']}>
      <div className={styles['rail']}>
        <div className={styles['railSticky']}>
          <span className={styles['brand']}>FirmwareSight</span>
          <nav className={styles['nav']} aria-label="Pages">
            {PAGES.map((entry) => {
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
            })}
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
        ) : (
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
        )}
      </div>
    </div>
  );
}
