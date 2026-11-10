/**
 * History · Trends (ADR-0041 D4): the store read as a sequence. Flash and RAM over the stored builds, how
 * much room each linker region has left, and which objects moved most.
 *
 * Like the rest of History, this is a viewer of persisted facts. Every number comes from one IPC read
 * (`query_history_trend`); Core decided the deltas, the headroom and the movers, and this file only lays
 * them out and draws them (AGENTS.md §3). Three rules shape it:
 *
 * 1. **Table first, chart as an aid.** The charts are hidden from assistive technology; the table carries
 *    every value, its state and the selection, so the keyboard can do everything the pointer can.
 * 2. **Unknown is a gap, a floor is hollow.** A build whose total is Unknown has no point and no delta;
 *    a partial total is labelled "at least".
 * 3. **Three region answers stay apart.** Stored before this version, no region table, and recorded are
 *    different facts and read differently.
 */

import { useEffect, useMemo, useRef, useState, type ReactNode } from 'react';

import styles from './Trends.module.css';
import { Button } from './components/Button';
import { DeltaValue } from './components/DeltaValue';
import { ErrorPanel } from './components/ErrorPanel';
import type { FactEvidence } from './components/FactValue';
import { Icon } from './components/Icon';
import { LineChart, Sparkline, type ChartMark } from './components/LineChart';
import { StateBadge, type StateName } from './components/StateBadge';
import { formatSize, type SizeUnit } from './format';
import { useI18n, type Translate } from './i18n';
import { queryHistoryTrend } from './ipc/bridge';
import type {
  ErrorEnvelopeDto,
  HistoryTrendDto,
  ObjectTrendDto,
  RegionTrendDto,
  TrendPointDto,
  TrendValueDto,
} from './ipc/types';

type Inspect = (evidence: FactEvidence, trigger: HTMLElement) => void;
type Budget = 'flash' | 'ram';

export function Trends({
  unit,
  onInspect,
  onCompare,
}: {
  readonly unit: SizeUnit;
  readonly onInspect: Inspect;
  /** Open Compare on this pair (older first) and run it once. */
  readonly onCompare: (baseId: string, targetId: string) => void;
}) {
  const { t } = useI18n();
  const [trend, setTrend] = useState<HistoryTrendDto | null>(null);
  const [error, setError] = useState<ErrorEnvelopeDto | null>(null);
  const [selected, setSelected] = useState<readonly number[]>([]);
  const [focused, setFocused] = useState<number | null>(null);
  const [moverSide, setMoverSide] = useState<Budget>('flash');
  const request = useRef(0);
  const rowRefs = useRef(new Map<number, HTMLButtonElement>());

  useEffect(() => {
    const id = ++request.current;
    void queryHistoryTrend({ limit: null }).then((outcome) => {
      if (request.current !== id) {
        return;
      }
      if (outcome.ok) {
        setTrend(outcome.value);
        setError(null);
      } else {
        setError(outcome.envelope);
      }
    });
  }, []);

  const points = useMemo(() => trend?.points ?? [], [trend]);
  const marks = useMemo(() => chartMarks(points, t), [points, t]);

  if (error !== null) {
    return <ErrorPanel envelope={error} label={t('history', 'trend.errorLabel')} heading={t('history', 'trend.errorHeading')} />;
  }
  if (trend === null) {
    return (
      <p className={styles['status']} role="status">
        {t('history', 'trend.loading')}
      </p>
    );
  }
  if (points.length === 0) {
    return <p className={styles['status']}>{t('history', 'trend.empty')}</p>;
  }

  const toggle = (index: number) => {
    setSelected((now) => (now.includes(index) ? now.filter((i) => i !== index) : [...now, index].slice(-2)));
  };
  const compare = () => {
    const [a, b] = [...selected].sort((x, y) => x - y);
    const base = a === undefined ? undefined : points[a];
    const target = b === undefined ? undefined : points[b];
    if (base !== undefined && target !== undefined) {
      onCompare(base.snapshotId, target.snapshotId);
    }
  };
  const inspect = (index: number, trigger: HTMLElement) => {
    const point = points[index];
    if (point !== undefined) {
      setFocused(index);
      onInspect(pointEvidence(point, unit, t), trigger);
    }
  };
  const pick = (index: number) => {
    // A pointer pick opens the same details the row's button opens, anchored to that button.
    const trigger = rowRefs.current.get(index);
    if (trigger !== undefined) {
      trigger.focus();
      inspect(index, trigger);
    }
  };
  const latest = points[points.length - 1];
  const tightest = tightestRegion(trend.regions);
  const newestFirst = points.map((point, index) => ({ point, index })).reverse();

  return (
    <section className={styles['trends']} aria-label={t('history', 'trend.region')}>
      <h2 className={styles['heading']}>{t('history', 'views.trends')}</h2>
      <p className={styles['status']}>
        {t('history', 'trend.status', { shown: points.length, total: trend.totalBuilds })}
        {points.length === 1 ? ` ${t('history', 'trend.single')}` : ''}
      </p>

      {latest === undefined ? null : (
        <dl className={styles['lead']}>
          <div>
            <dt>{t('history', 'trend.lead.flash')}</dt>
            <dd>
              <Value value={latest.flash} unit={unit} />
            </dd>
          </div>
          <div>
            <dt>{t('history', 'trend.lead.ram')}</dt>
            <dd>
              <Value value={latest.ram} unit={unit} />
            </dd>
          </div>
          <div>
            <dt>{t('history', 'trend.lead.tightest')}</dt>
            <dd>{tightest === null ? <span className={styles['muted']}>{t('history', 'trend.lead.noRegion')}</span> : <Headroom region={tightest} unit={unit} />}</dd>
          </div>
        </dl>
      )}

      <div className={styles['charts']}>
        {(['flash', 'ram'] as const).map((budget) => (
          <figure key={budget} className={styles['figure']}>
            <figcaption className={styles['figcaption']}>{t('history', budget === 'flash' ? 'trend.chart.flash' : 'trend.chart.ram')}</figcaption>
            <LineChart
              values={points.map((p) => p[budget].bytes)}
              floors={points.flatMap((p, i) => (p[budget].state === 'partial' ? [i] : []))}
              marks={marks}
              selected={selected}
              focused={focused}
              onPick={pick}
            />
          </figure>
        ))}
      </div>
      <p className={styles['legend']}>{t('history', 'trend.chart.legend')}</p>

      <div className={styles['tableBar']}>
        <Button variant="primary" disabled={selected.length !== 2} onClick={compare}>
          {t('history', 'trend.compare', { count: selected.length })}
        </Button>
      </div>
      <div className={styles['scroll']}>
        <table className={styles['table']}>
          <caption className={styles['caption']}>{t('history', 'trend.table.caption')}</caption>
          <thead>
            <tr>
              <th scope="col">
                <span className={styles['sr']}>{t('history', 'builds.col.select')}</span>
              </th>
              <th scope="col">{t('history', 'builds.col.artifact')}</th>
              <th scope="col">{t('history', 'col.stored')}</th>
              <th scope="col">{t('history', 'trend.col.flash')}</th>
              <th scope="col">{t('history', 'trend.col.ram')}</th>
              <th scope="col">{t('history', 'trend.col.gate')}</th>
              <th scope="col">{t('history', 'trend.col.marks')}</th>
            </tr>
          </thead>
          <tbody>
            {newestFirst.map(({ point, index }) => (
              <tr key={point.buildId} className={focused === index ? styles['current'] : undefined}>
                <td>
                  <input
                    type="checkbox"
                    aria-label={t('history', 'trend.selectRow', { name: point.fileName, stored: point.storedAt })}
                    checked={selected.includes(index)}
                    onChange={() => {
                      toggle(index);
                    }}
                  />
                </td>
                <th scope="row">
                  <button
                    type="button"
                    className={styles['rowLink']}
                    ref={(node) => {
                      if (node === null) {
                        rowRefs.current.delete(index);
                      } else {
                        rowRefs.current.set(index, node);
                      }
                    }}
                    aria-label={t('history', 'trend.inspect', { name: point.fileName })}
                    onClick={(event) => {
                      inspect(index, event.currentTarget);
                    }}
                  >
                    {point.fileName}
                  </button>
                </th>
                <td className={styles['mono']}>{point.storedAt}</td>
                <td>
                  <Value value={point.flash} unit={unit} />
                </td>
                <td>
                  <Value value={point.ram} unit={unit} />
                </td>
                <td>
                  <Gate disposition={point.gate} t={t} />
                </td>
                <td>
                  <Marks point={point} t={t} />
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <section className={styles['block']} aria-labelledby="fs-trend-regions">
        <h3 id="fs-trend-regions" className={styles['subheading']}>
          {t('history', 'trend.regions.heading')}
        </h3>
        <p className={styles['note']}>{t('history', 'trend.regions.note')}</p>
        {trend.regions.length === 0 ? (
          <p className={styles['muted']}>{t('history', 'trend.regions.empty')}</p>
        ) : (
          <ul className={styles['regions']}>
            {trend.regions.map((region) => (
              <RegionCard key={region.name} region={region} unit={unit} t={t} />
            ))}
          </ul>
        )}
      </section>

      <section className={styles['block']} aria-labelledby="fs-trend-movers">
        <div className={styles['blockHead']}>
          <h3 id="fs-trend-movers" className={styles['subheading']}>
            {t('history', 'trend.movers.heading')}
          </h3>
          <div className={styles['cuts']} role="group" aria-label={t('history', 'trend.movers.side')}>
            {(['flash', 'ram'] as const).map((side) => (
              <button
                key={side}
                type="button"
                className={styles['cut']}
                aria-pressed={moverSide === side}
                onClick={() => {
                  setMoverSide(side);
                }}
              >
                {t('history', side === 'flash' ? 'trend.movers.flash' : 'trend.movers.ram')}
              </button>
            ))}
          </div>
        </div>
        <Movers movers={moverSide === 'flash' ? trend.moversFlash : trend.moversRam} unit={unit} t={t} />
      </section>
    </section>
  );
}

function Value({ value, unit }: { readonly value: TrendValueDto; readonly unit: SizeUnit }) {
  const { t } = useI18n();
  if (value.bytes === null) {
    return <StateBadge state="UNKNOWN" label={t('history', 'budget.unknown')} />;
  }
  return (
    <span className={styles['value']}>
      <span className={styles['mono']}>
        {value.state === 'partial' ? `${t('history', 'trend.floor')} ` : ''}
        {formatSize(value.bytes, unit)}
      </span>
      {value.delta === null ? null : (
        <span className={styles['delta']}>
          <DeltaValue delta={value.delta} format={(magnitude) => formatSize(magnitude, unit)} />
        </span>
      )}
    </span>
  );
}

function Gate({ disposition, t }: { readonly disposition: string | null; readonly t: Translate }) {
  if (disposition === null) {
    return <span className={styles['muted']}>{t('history', 'trend.gate.none')}</span>;
  }
  const name: StateName = disposition === 'PASS' || disposition === 'REVIEW' || disposition === 'BLOCK' ? disposition : 'UNKNOWN';
  const label =
    name === 'PASS'
      ? t('history', 'disposition.pass')
      : name === 'REVIEW'
        ? t('history', 'disposition.review')
        : name === 'BLOCK'
          ? t('history', 'disposition.block')
          : t('history', 'disposition.unknown');
  return <StateBadge state={name} label={label} />;
}

function Marks({ point, t }: { readonly point: TrendPointDto; readonly t: Translate }) {
  if (!point.pinned && point.releases.length === 0) {
    return null;
  }
  return (
    <span className={styles['marks']}>
      {point.pinned ? (
        <span className={styles['mark']}>
          <Icon name="pin" size="inline" />
          {t('history', 'trend.mark.baseline')}
        </span>
      ) : null}
      {point.releases.map((version) => (
        <span key={version} className={`${styles['mark']} ${styles['mono']}`}>
          {version}
        </span>
      ))}
    </span>
  );
}

function Headroom({
  region,
  unit,
  named = true,
}: {
  readonly region: RegionTrendDto;
  readonly unit: SizeUnit;
  /** False inside a region card, whose head already names the region. */
  readonly named?: boolean;
}) {
  const { t } = useI18n();
  const latest = region.latest;
  if (latest === null) {
    return <span className={styles['muted']}>{t('history', 'trend.regions.noLatest')}</span>;
  }
  const used = formatSize(latest.used, unit);
  const capacity = formatSize(latest.capacity, unit);
  let answer: ReactNode;
  if (latest.exceeded) {
    answer = <StateBadge state="BLOCK" label={t('history', 'trend.regions.exceeded', { used, capacity })} />;
  } else if (!latest.exact || latest.remaining === null || latest.remainingPerMille === null) {
    // A floor: at least this much is used, so no headroom is claimed (ADR-0041 D3).
    answer = <StateBadge state="REVIEW" label={t('history', 'trend.regions.floor', { used, capacity })} />;
  } else {
    answer = (
      <span className={styles['mono']}>
        {t('history', 'trend.regions.left', {
          remaining: formatSize(latest.remaining, unit),
          capacity,
          percent: (latest.remainingPerMille / 10).toFixed(1),
        })}
      </span>
    );
  }
  return (
    <span className={styles['headroom']}>
      {named ? <span className={styles['regionName']}>{region.name}</span> : null}
      {answer}
    </span>
  );
}

function RegionCard({ region, unit, t }: { readonly region: RegionTrendDto; readonly unit: SizeUnit; readonly t: Translate }) {
  const notRecorded = region.cells.filter((cell) => cell.state === 'notRecorded').length;
  const kind =
    region.kind === 'flash'
      ? t('history', 'trend.regions.kind.flash')
      : region.kind === 'ram'
        ? t('history', 'trend.regions.kind.ram')
        : t('history', 'trend.regions.kind.unknown');
  return (
    <li className={styles['regionCard']}>
      <div className={styles['regionHead']}>
        <span className={styles['regionName']}>{region.name}</span>
        <span className={styles['muted']}>{kind}</span>
      </div>
      <Sparkline values={region.cells.map((cell) => cell.used)} reference={region.cells.map((cell) => cell.capacity)} />
      <Headroom region={region} unit={unit} named={false} />
      {notRecorded === 0 ? null : (
        <span className={styles['unknownNote']}>
          <StateBadge state="UNKNOWN" label={t('history', 'trend.regions.notRecorded', { count: notRecorded })} />
        </span>
      )}
    </li>
  );
}

function Movers({ movers, unit, t }: { readonly movers: readonly ObjectTrendDto[]; readonly unit: SizeUnit; readonly t: Translate }) {
  if (movers.length === 0) {
    return <p className={styles['muted']}>{t('history', 'trend.movers.empty')}</p>;
  }
  return (
    <div className={styles['scroll']}>
      <table className={styles['table']}>
        <caption className={styles['captionVisible']}>{t('history', 'trend.movers.caption')}</caption>
        <thead>
          <tr>
            <th scope="col">{t('history', 'trend.movers.col.object')}</th>
            <th scope="col">{t('history', 'trend.movers.col.trend')}</th>
            <th scope="col">{t('history', 'trend.movers.col.first')}</th>
            <th scope="col">{t('history', 'trend.movers.col.last')}</th>
            <th scope="col">{t('history', 'trend.movers.col.change')}</th>
          </tr>
        </thead>
        <tbody>
          {movers.map((mover) => (
            <tr key={mover.object}>
              <th scope="row" className={styles['mono']}>
                {mover.object}
              </th>
              <td>
                <Sparkline values={mover.values} />
              </td>
              <td className={styles['mono']}>{formatSize(mover.first, unit)}</td>
              <td className={styles['mono']}>{formatSize(mover.last, unit)}</td>
              <td>
                <DeltaValue delta={mover.delta} format={(magnitude) => formatSize(magnitude, unit)} />
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

/**
 * The region with the smallest exact remaining share. An exceeded region is the tightest of all; a floor
 * claims no share, so it is never named as roomy and only leads when nothing exact does.
 */
function tightestRegion(regions: readonly RegionTrendDto[]): RegionTrendDto | null {
  let best: RegionTrendDto | null = null;
  let bestShare = Infinity;
  for (const region of regions) {
    const latest = region.latest;
    if (latest === null) {
      continue;
    }
    const share = latest.exceeded ? -1 : latest.exact ? (latest.remainingPerMille ?? Infinity) : 1001;
    if (share < bestShare) {
      best = region;
      bestShare = share;
    }
  }
  return best;
}

function chartMarks(points: readonly TrendPointDto[], t: Translate): ChartMark[] {
  return points.flatMap((point, index) => {
    const labels = [...(point.pinned ? [t('history', 'trend.mark.baseline')] : []), ...point.releases];
    return labels.length === 0 ? [] : [{ index, label: labels.join(' · ') }];
  });
}

function regionsWord(state: string, t: Translate): string {
  return state === 'recorded'
    ? t('history', 'trend.regionsState.recorded')
    : state === 'noRegionTable'
      ? t('history', 'trend.regionsState.noRegionTable')
      : t('history', 'trend.regionsState.notRecorded');
}

function valueText(value: TrendValueDto, unit: SizeUnit, t: Translate): string {
  if (value.bytes === null) {
    return t('history', 'budget.unknown');
  }
  const amount = formatSize(value.bytes, unit);
  return value.state === 'partial' ? `${t('history', 'trend.floor')} ${amount}` : amount;
}

function pointEvidence(point: TrendPointDto, unit: SizeUnit, t: Translate): FactEvidence {
  return {
    field: point.fileName,
    value: valueText(point.flash, unit, t),
    classification: point.flash.state === 'exact' ? 'observed' : point.flash.state === 'partial' ? 'derived' : 'unknown',
    source: t('history', 'inspector.source'),
    details: [
      [t('history', 'detail.buildId'), point.buildId],
      [t('history', 'detail.snapshotId'), point.snapshotId],
      [t('history', 'col.stored'), point.storedAt],
      [t('history', 'trend.detail.flash'), valueText(point.flash, unit, t)],
      [t('history', 'trend.detail.ram'), valueText(point.ram, unit, t)],
      [t('history', 'trend.detail.gate'), point.gate ?? t('history', 'trend.gate.none')],
      [t('history', 'trend.detail.releases'), point.releases.length === 0 ? '-' : point.releases.join(', ')],
      [t('history', 'trend.detail.regions'), regionsWord(point.regions, t)],
      [t('history', 'trend.detail.pinned'), point.pinned ? t('history', 'trend.detail.yes') : t('history', 'trend.detail.no')],
    ],
  };
}
