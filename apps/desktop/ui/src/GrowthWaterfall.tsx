/**
 * Compare → Summary → where the growth came from (D7, ADR-0039 D4).
 *
 * The steps are Core's (`query_growth_waterfall`): base total, one step per object or library by size of
 * change, the folded rest, the linker's own bytes, target total. Each step is drawn as a bar on a shared
 * scale (the largest move is the full length), growth to the right with ▲ and shrinkage to the left with
 * ▼, in one neutral colour: a change is a fact, not a verdict (ADR-0034). An Unknown step has no bar and
 * says why.
 */

import styles from './GrowthWaterfall.module.css';
import { useEffect, useState } from 'react';

import { ErrorPanel } from './components/ErrorPanel';
import { formatSize, type SizeUnit } from './format';
import { useI18n } from './i18n';
import { queryGrowthWaterfall } from './ipc/bridge';
import type { BudgetSideDto, ErrorEnvelopeDto, WaterfallDto, WaterfallStepDto } from './ipc/types';

function label(step: WaterfallStepDto, t: ReturnType<typeof useI18n>['t']): string {
  switch (step.kind) {
    case 'start':
      return t('compare', 'waterfall.base');
    case 'end':
      return t('compare', 'waterfall.target');
    case 'rest':
      return t('compare', 'waterfall.rest', { count: String(step.folded) });
    case 'linker':
      return t('compare', 'waterfall.linker');
    case 'item':
      return step.label;
  }
}

/** ▲ for growth, ▼ for shrinkage, the size beside it: direction in text, not in colour. */
function glyphDelta(delta: number, unit: SizeUnit): string {
  return delta === 0 ? formatSize(0, unit) : `${delta > 0 ? '▲' : '▼'}${formatSize(Math.abs(delta), unit)}`;
}

export function GrowthWaterfall({ diffId, unit }: { readonly diffId: string; readonly unit: SizeUnit }) {
  const { t } = useI18n();
  const [side, setSide] = useState<BudgetSideDto>('flash');
  const [libraries, setLibraries] = useState(false);
  const [fall, setFall] = useState<WaterfallDto | null>(null);
  const [error, setError] = useState<ErrorEnvelopeDto | null>(null);

  useEffect(() => {
    let live = true;
    void queryGrowthWaterfall({ diffId, side, libraries, topN: 12 }).then((result) => {
      if (!live) {
        return;
      }
      if (result.ok) {
        setFall(result.value);
        setError(null);
      } else {
        setError(result.envelope);
      }
    });
    return () => {
      live = false;
    };
  }, [diffId, side, libraries]);

  const moves = (fall?.steps ?? []).filter((s) => s.kind !== 'start' && s.kind !== 'end');
  const largest = Math.max(0, ...moves.map((s) => (s.delta === null ? 0 : Math.abs(s.delta))));

  return (
    <section className={styles['area']} aria-labelledby="fs-waterfall-title">
      <div className={styles['head']}>
        <h3 id="fs-waterfall-title" className={styles['title']}>
          {t('compare', 'waterfall.title')}
        </h3>
        <div className={styles['cuts']} role="group" aria-label={t('compare', 'waterfall.sides')}>
          {(['flash', 'ram'] as const).map((s) => (
            <button
              key={s}
              type="button"
              className={styles['cut']}
              aria-pressed={side === s}
              onClick={() => {
                setSide(s);
              }}
            >
              {t('compare', s === 'flash' ? 'waterfall.flash' : 'waterfall.ram')}
            </button>
          ))}
        </div>
        <div className={styles['cuts']} role="group" aria-label={t('compare', 'waterfall.grain')}>
          {([false, true] as const).map((lib) => (
            <button
              key={String(lib)}
              type="button"
              className={styles['cut']}
              aria-pressed={libraries === lib}
              onClick={() => {
                setLibraries(lib);
              }}
            >
              {t('compare', lib ? 'waterfall.libraries' : 'waterfall.objects')}
            </button>
          ))}
        </div>
      </div>

      {error === null ? null : (
        <ErrorPanel envelope={error} label={t('compare', 'waterfall.errorLabel')} heading={t('compare', 'waterfall.errorHeading')} />
      )}
      {fall !== null && !fall.available ? (
        <p className={styles['status']} role="status">
          {fall.reason}
        </p>
      ) : null}

      {fall === null || !fall.available ? null : (
        <table className={styles['table']}>
          <caption className={styles['caption']}>{t('compare', 'waterfall.caption')}</caption>
          <thead>
            <tr>
              <th scope="col">{t('compare', 'waterfall.col.step')}</th>
              <th scope="col">
                <span className={styles['sr']}>{t('compare', 'waterfall.col.bar')}</span>
              </th>
              <th scope="col" className={styles['num']}>
                {t('compare', 'waterfall.col.delta')}
              </th>
              <th scope="col" className={styles['num']}>
                {t('compare', 'waterfall.col.total')}
              </th>
            </tr>
          </thead>
          <tbody>
            {fall.steps.map((step, i) => {
              const total = step.kind === 'start' || step.kind === 'end';
              const ratio = step.delta === null || largest === 0 ? null : Math.abs(step.delta) / largest;
              return (
                <tr key={`${step.kind}-${step.label}-${String(i)}`} className={total ? styles['totalRow'] : undefined}>
                  <th scope="row" className={step.kind === 'item' ? styles['mono'] : undefined}>
                    {label(step, t)}
                    {step.change === null ? null : (
                      <span className={styles['change']}> {t('compare', step.change === 'added' ? 'waterfall.change.added' : step.change === 'removed' ? 'waterfall.change.removed' : 'waterfall.change.changed')}</span>
                    )}
                  </th>
                  <td className={styles['barCell']} aria-hidden="true">
                    {total ? null : ratio === null ? null : (
                      <span className={styles['axis']}>
                        <span
                          className={step.delta !== null && step.delta < 0 ? styles['down'] : styles['up']}
                          style={{ width: `${(ratio * 50).toFixed(2)}%` }}
                        />
                      </span>
                    )}
                  </td>
                  <td className={`${styles['num']} ${styles['mono']}`}>
                    {total ? '' : step.delta === null ? t('compare', 'waterfall.unknown') : glyphDelta(step.delta, unit)}
                    {step.reason !== null && step.delta === null ? <span className={styles['reason']}> {step.reason}</span> : null}
                  </td>
                  <td className={`${styles['num']} ${styles['mono']}`}>
                    {step.total === null ? t('compare', 'waterfall.unknown') : formatSize(step.total, unit)}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      )}
    </section>
  );
}
