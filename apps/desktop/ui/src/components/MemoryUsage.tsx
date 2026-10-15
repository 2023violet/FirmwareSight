/**
 * One bar per linker memory region: how full it is, said as a number, a word and an icon.
 *
 * Every figure here is Rust's. `usedPercent` arrives already rounded (`service::usage_percent`), and the
 * bar's width is that same number read back, clamped to the track so an overfull region still fits on
 * screen while its label keeps the true figure. When Rust has no percentage the bar is not drawn as empty
 * or as zero: the row says Unknown in neutral grey with a hollow icon and the reason, as DESIGN.md 6 asks.
 */

import styles from './MemoryUsage.module.css';

import { StateIcon, type StateName } from './StateBadge';
import { formatSize, type SizeUnit } from '../format';
import { useI18n, type Translate } from '../i18n';
import type { MemoryRegionDto } from '../ipc/types';

/** The icon that travels with each risk word. Colour only repeats what the icon and word already say. */
function riskState(risk: string): StateName {
  switch (risk) {
    case 'within-capacity':
      return 'PASS';
    case 'exceeded':
      return 'BLOCK';
    case 'review':
      return 'REVIEW';
    default:
      return 'UNKNOWN';
  }
}

function riskWord(risk: string, t: Translate): string {
  switch (risk) {
    case 'within-capacity':
      return t('common', 'memory.riskWithin');
    case 'exceeded':
      return t('common', 'memory.riskExceeded');
    case 'review':
      return t('common', 'memory.riskReview');
    default:
      return t('common', 'memory.unknown');
  }
}

function kindWord(kind: string, t: Translate): string {
  switch (kind) {
    case 'nonvolatile':
      return t('common', 'memory.kindFlash');
    case 'volatile-ram':
      return t('common', 'memory.kindRam');
    default:
      return t('common', 'memory.unknown');
  }
}

/** Byte strings stay exact; only a value JavaScript can hold exactly is reformatted for the size unit. */
function bytes(value: string | null, unit: SizeUnit, t: Translate): string {
  if (value === null) {
    return t('common', 'memory.unknown');
  }
  const parsed = Number(value);
  return Number.isSafeInteger(parsed) ? formatSize(parsed, unit) : `${value} bytes`;
}

function missingReason(status: string, t: Translate): string {
  switch (status) {
    case 'capacity_unknown':
      return t('common', 'memory.capacityUnknown');
    case 'zero_capacity':
      return t('common', 'memory.zeroCapacity');
    default:
      return t('common', 'memory.usageUnknown');
  }
}

/**
 * A partial figure is a lower bound, but Rust's two end words already carry a bound of their own.
 * `>99.9%` is itself a lower bound and is shown as it is; `<0.1%` is an upper bound on the share
 * counted so far, and "at least <0.1%" would say both at once (M1 review N2).
 */
function boundText(percent: string, status: string, t: Translate): string {
  if (status !== 'lower_bound' || percent.startsWith('>')) {
    return percent;
  }
  if (percent.startsWith('<')) {
    return t('common', 'memory.atLeastTrace', { percent });
  }
  return t('common', 'memory.atLeast', { percent });
}

function UsageRow({ region, unit }: { readonly region: MemoryRegionDto; readonly unit: SizeUnit }) {
  const { t } = useI18n();
  const percent = region.usedPercent.value;
  const known = percent !== null;
  const state: StateName = known ? riskState(region.risk) : 'UNKNOWN';
  // `<0.1%` and `>99.9%` are Rust's words for the two ends; the bar draws them at the bound they name.
  const numeric = known ? Number.parseFloat(percent.replace(/[<>]/, '')) : 0;
  const width = Math.min(Math.max(numeric, 0), 100);
  const shown = percent === null ? null : boundText(percent, region.usedPercent.status, t);

  return (
    <li className={styles['row']}>
      <div className={styles['head']}>
        <span className={styles['name']}>{region.name}</span>
        <span className={styles['kind']}>{kindWord(region.kind, t)}</span>
        <span className={styles['figure']}>
          {known ? shown : t('common', 'memory.unknown')}
        </span>
      </div>
      <div
        className={styles['track']}
        data-state={state}
        role="meter"
        aria-label={t('common', 'memory.meterLabel', { name: region.name })}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={known ? width : undefined}
        aria-valuetext={
          known
            ? t('common', 'memory.meterText', {
                percent: shown ?? '',
                used: bytes(region.usedBytes, unit, t),
                capacity: bytes(region.capacityBytes, unit, t),
                risk: riskWord(region.risk, t),
              })
            : missingReason(region.usedPercent.status, t)
        }
      >
        {known ? <span className={styles['fill']} style={{ inlineSize: `${width}%` }} /> : null}
      </div>
      <div className={styles['foot']}>
        <span className={styles['state']}>
          <StateIcon state={state} />
          {known ? riskWord(region.risk, t) : missingReason(region.usedPercent.status, t)}
        </span>
        <span className={styles['bytes']}>
          {t('common', 'memory.usedOf', {
            used: bytes(region.usedBytes, unit, t),
            capacity: bytes(region.capacityBytes, unit, t),
          })}
        </span>
      </div>
    </li>
  );
}

/** The regions in the order Core reported them, which is the linker script's own order. */
export function MemoryUsage({
  regions,
  unit,
}: {
  readonly regions: readonly MemoryRegionDto[];
  readonly unit: SizeUnit;
}) {
  const { t } = useI18n();
  if (regions.length === 0) {
    return (
      <p className={styles['none']} role="status">
        <StateIcon state="UNKNOWN" />
        {t('common', 'memory.none')}
      </p>
    );
  }
  return (
    <ul className={styles['list']} aria-label={t('common', 'memory.title')}>
      {regions.map((region) => (
        <UsageRow key={region.evidenceId} region={region} unit={unit} />
      ))}
    </ul>
  );
}
