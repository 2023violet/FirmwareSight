/**
 * Analyze → Composition (D7, ADR-0039): where a build's bytes are, as a treemap you can drill into.
 *
 * Every byte figure is Core's (`query_composition`): region → output section → object → symbol, with the
 * gaps stated as their own entries. This view lays the focus node's children out with `squarify` (areas
 * proportional to bytes, geometry only) and lists every child in a table, so the numbers never depend on
 * reading the picture: an Unknown child has no area and is listed with its reason (`AGENTS.md` 8).
 */

import styles from './Composition.module.css';
import { useCallback, useEffect, useRef, useState, type KeyboardEvent } from 'react';

import type { FactEvidence } from './components/FactValue';
import { formatSize, type SizeUnit } from './format';
import { useI18n } from './i18n';
import { queryComposition } from './ipc/bridge';
import type { BudgetSideDto, CompositionDto, CompositionNodeDto, ErrorEnvelopeDto } from './ipc/types';
import { squarify } from './lib/squarify';
import { ErrorPanel } from './components/ErrorPanel';

const KIND_TONE: Readonly<Record<CompositionNodeDto['kind'], string>> = {
  root: 'tone1',
  region: 'tone2',
  noRegion: 'tone1',
  section: 'tone3',
  object: 'tone4',
  symbol: 'tone5',
  linkerBytes: 'other',
  noSymbol: 'other',
  more: 'other',
};

/** A cell drills in when Core has something below it. */
function drillable(node: CompositionNodeDto): boolean {
  return node.kind !== 'more' && (node.children.length > 0 || node.folded > 0);
}

function share(part: number | null, whole: number | null): string | null {
  if (part === null || whole === null || whole === 0) {
    return null;
  }
  return `${((part / whole) * 100).toFixed(1)} %`;
}

export function CompositionView({
  snapshotId,
  unit,
  onInspect,
}: {
  readonly snapshotId: string;
  readonly unit: SizeUnit;
  readonly onInspect: (evidence: FactEvidence, trigger: HTMLElement) => void;
}) {
  const { t } = useI18n();
  const [side, setSide] = useState<BudgetSideDto>('flash');
  const [focus, setFocus] = useState<string | null>(null);
  const [answer, setAnswer] = useState<CompositionDto | null>(null);
  const [error, setError] = useState<ErrorEnvelopeDto | null>(null);
  const cellRefs = useRef<(HTMLButtonElement | null)[]>([]);
  // A drill from a cell (or Backspace from one) removes the focused cell; once the next level has
  // arrived, focus its first cell so the keyboard user keeps a place in the map.
  const refocus = useRef(false);

  useEffect(() => {
    if (refocus.current && answer !== null) {
      refocus.current = false;
      cellRefs.current[0]?.focus();
    }
  }, [answer]);

  useEffect(() => {
    let live = true;
    void queryComposition({ snapshotId, side, focus }).then((result) => {
      if (!live) {
        return;
      }
      if (result.ok) {
        setAnswer(result.value);
        setError(null);
      } else {
        setError(result.envelope);
      }
    });
    return () => {
      live = false;
    };
  }, [snapshotId, side, focus]);

  const chooseSide = (next: BudgetSideDto) => {
    setSide(next);
    setFocus(null);
  };

  const node = answer?.node ?? null;
  const placed = node === null ? [] : squarify(node.children, (c) => c.bytes, { x: 0, y: 0, width: 100, height: 100 });

  const evidence = useCallback(
    (n: CompositionNodeDto): FactEvidence => ({
      field: `composition.${n.kind}`,
      value: n.bytes === null ? t('analyze', 'composition.unknown') : formatSize(n.bytes, unit),
      classification: n.classification,
      source: t('analyze', 'composition.source'),
      ...(n.locator === null ? {} : { locator: n.locator }),
      details: [
        ...(n.reason === null ? [] : [[t('analyze', 'composition.reason'), n.reason] as const]),
        ...(n.capacity === null ? [] : [[t('analyze', 'composition.capacity'), formatSize(n.capacity, unit)] as const]),
        ...(node !== null && share(n.bytes, node.bytes) !== null
          ? [[t('analyze', 'composition.share'), share(n.bytes, node.bytes) ?? ''] as const]
          : []),
      ],
    }),
    [node, t, unit],
  );

  const goUp = () => {
    const path = answer?.path ?? [];
    if (path.length > 1) {
      refocus.current = true;
      setFocus(path[path.length - 2]?.id ?? null);
    }
  };

  const onCellKey = (event: KeyboardEvent<HTMLButtonElement>, index: number) => {
    const move = (to: number) => {
      event.preventDefault();
      cellRefs.current[Math.max(0, Math.min(placed.length - 1, to))]?.focus();
    };
    if (event.key === 'ArrowRight' || event.key === 'ArrowDown') {
      move(index + 1);
    } else if (event.key === 'ArrowLeft' || event.key === 'ArrowUp') {
      move(index - 1);
    } else if (event.key === 'Backspace') {
      event.preventDefault();
      goUp();
    }
  };

  return (
    <section className={styles['area']} aria-label={t('analyze', 'composition.region')}>
      <div className={styles['bar']}>
        <div className={styles['cuts']} role="group" aria-label={t('analyze', 'composition.sides')}>
          {(['flash', 'ram'] as const).map((s) => (
            <button
              key={s}
              type="button"
              className={styles['cut']}
              aria-pressed={side === s}
              onClick={() => {
                chooseSide(s);
              }}
            >
              {t('analyze', s === 'flash' ? 'composition.flash' : 'composition.ram')}
            </button>
          ))}
        </div>
        <nav aria-label={t('analyze', 'composition.path')} className={styles['crumbs']}>
          {(answer?.path ?? []).map((crumb, i, all) =>
            i === all.length - 1 ? (
              <span key={crumb.id} className={styles['here']} aria-current="location">
                {crumb.name}
              </span>
            ) : (
              <span key={crumb.id}>
                <button
                  type="button"
                  className={styles['crumb']}
                  onClick={() => {
                    setFocus(crumb.id);
                  }}
                >
                  {crumb.name}
                </button>
                <span aria-hidden="true" className={styles['sep']}>
                  /
                </span>
              </span>
            ),
          )}
        </nav>
      </div>

      {error === null ? null : (
        <ErrorPanel
          envelope={error}
          label={t('analyze', 'composition.errorLabel')}
          heading={t('analyze', 'composition.errorHeading')}
        />
      )}
      {answer !== null && !answer.available ? (
        <p className={styles['status']} role="status">
          {answer.reason}
        </p>
      ) : null}

      {node === null ? null : (
        <>
          <p className={styles['lead']}>
            <span className={styles['leadName']}>{node.name}</span>{' '}
            <span className={styles['mono']}>
              {node.bytes === null ? t('analyze', 'composition.unknown') : formatSize(node.bytes, unit)}
            </span>
            {node.capacity === null ? null : (
              <span className={styles['muted']}>
                {' '}
                {t('analyze', 'composition.ofCapacity', { capacity: formatSize(node.capacity, unit) })}
              </span>
            )}
          </p>
          <div className={styles['map']} role="group" aria-label={t('analyze', 'composition.mapLabel', { name: node.name })}>
            {placed.length === 0 ? (
              <p className={styles['empty']}>{t('analyze', 'composition.nothingToDraw')}</p>
            ) : (
              placed.map((p, i) => {
                const c = p.item;
                const big = p.width > 9 && p.height > 9;
                const label = `${c.name}, ${c.bytes === null ? t('analyze', 'composition.unknown') : formatSize(c.bytes, unit)}${share(c.bytes, node.bytes) === null ? '' : `, ${share(c.bytes, node.bytes)}`}`;
                return (
                  <button
                    key={c.id}
                    ref={(el) => {
                      cellRefs.current[i] = el;
                    }}
                    type="button"
                    className={`${styles['cell']} ${styles[KIND_TONE[c.kind]] ?? ''}`}
                    style={{ left: `${p.x}%`, top: `${p.y}%`, width: `${p.width}%`, height: `${p.height}%` }}
                    aria-label={drillable(c) ? t('analyze', 'composition.open', { label }) : label}
                    title={label}
                    onKeyDown={(event) => {
                      onCellKey(event, i);
                    }}
                    onClick={(event) => {
                      if (drillable(c)) {
                        refocus.current = true;
                        setFocus(c.id);
                      } else {
                        onInspect(evidence(c), event.currentTarget);
                      }
                    }}
                  >
                    {big ? (
                      <span className={styles['cellText']}>
                        <span className={styles['cellName']}>{c.name}</span>
                        <span className={styles['cellBytes']}>{c.bytes === null ? '' : formatSize(c.bytes, unit)}</span>
                      </span>
                    ) : null}
                  </button>
                );
              })
            )}
          </div>

          <table className={styles['table']}>
            <caption className={styles['caption']}>{t('analyze', 'composition.tableCaption', { name: node.name })}</caption>
            <thead>
              <tr>
                <th scope="col">{t('analyze', 'composition.col.name')}</th>
                <th scope="col">{t('analyze', 'composition.col.kind')}</th>
                <th scope="col" className={styles['num']}>
                  {t('analyze', 'composition.col.bytes')}
                </th>
                <th scope="col" className={styles['num']}>
                  {t('analyze', 'composition.col.share')}
                </th>
                <th scope="col">{t('analyze', 'composition.col.evidence')}</th>
              </tr>
            </thead>
            <tbody>
              {node.children.map((c) => (
                <tr key={c.id}>
                  <th scope="row" className={styles['mono']}>
                    {drillable(c) ? (
                      <button
                        type="button"
                        className={styles['rowLink']}
                        onClick={() => {
                          setFocus(c.id);
                        }}
                      >
                        {c.name}
                      </button>
                    ) : (
                      c.name
                    )}
                  </th>
                  <td>{t('analyze', `composition.kind.${c.kind}`)}</td>
                  <td className={`${styles['num']} ${styles['mono']}`}>
                    {c.bytes === null ? t('analyze', 'composition.unknown') : formatSize(c.bytes, unit)}
                  </td>
                  <td className={`${styles['num']} ${styles['mono']}`}>{share(c.bytes, node.bytes) ?? '—'}</td>
                  <td>
                    <button
                      type="button"
                      className={styles['rowLink']}
                      onClick={(event) => {
                        onInspect(evidence(c), event.currentTarget);
                      }}
                    >
                      {c.classification}
                    </button>
                    {c.reason === null ? null : <span className={styles['reason']}> {c.reason}</span>}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </>
      )}
    </section>
  );
}
