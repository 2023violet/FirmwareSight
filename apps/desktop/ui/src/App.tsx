/**
 * The desktop shell: one navigation rail and one page at a time.
 *
 * P2 makes Compare a real page rather than a promise, so the rail lists exactly the two stages this
 * build implements. A stage that does not exist gets no entry and no words anywhere on the screen -
 * an empty placeholder you can click is not an honest "not built yet".
 *
 * Three pieces of state live here because they outlive a page switch:
 *
 * - the size unit, which is one answer to one question and must not differ between two tables the
 *   same reader is comparing (US-001, prompt §39);
 * - what Analyze currently holds, so navigating to Compare and back does not make the session forget
 *   the build it just analyzed;
 * - and the last summary Analyze proved, which Compare is allowed to *prefer* as its target - the
 *   build the reader most recently looked at - but never compare on its own initiative (prompt §18).
 */

import { useState } from 'react';

import styles from './App.module.css';
import { Analyze } from './Analyze';
import { Compare } from './Compare';
import type { SizeUnit } from './format';
import type { AnalysisSummaryDto, SelectionDto } from './ipc/types';
import { cx } from './styles/classnames';

/** The pages this build has, in the order the rail lists them. */
const PAGES: readonly { readonly key: 'analyze' | 'compare'; readonly label: string }[] = [
  { key: 'analyze', label: 'Analyze' },
  { key: 'compare', label: 'Compare' },
];

export function App() {
  const [page, setPage] = useState<'analyze' | 'compare'>('analyze');
  const [unit, setUnit] = useState<SizeUnit>('bytes');
  const [selection, setSelection] = useState<SelectionDto | null>(null);
  const [lastGood, setLastGood] = useState<AnalysisSummaryDto | null>(null);

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
            lastGood={lastGood}
            onLastGoodChange={setLastGood}
          />
        ) : (
          <Compare
            unit={unit}
            onUnitChange={setUnit}
            lastAnalyzedSnapshotId={lastGood?.identity.snapshotId ?? null}
            onGoToAnalyze={() => {
              setPage('analyze');
            }}
          />
        )}
      </div>
    </div>
  );
}
