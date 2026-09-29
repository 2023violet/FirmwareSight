/**
 * The Bytes / KiB switch, in one place because it is one decision.
 *
 * The state itself lives in the shell, above every page: a summary in bytes beside a table in KiB
 * would be two answers to one question (US-001, and US-002 asks the same of Compare). Choosing a
 * unit re-labels text only - no query changes, no data is re-read, and addresses, hashes, counts
 * and ids never pass through the conversion (US-001 addendum 3, prompt §39).
 */

import styles from './SizeUnitSwitch.module.css';
import type { SizeUnit } from '../format';

export function SizeUnitSwitch({
  unit,
  onSelect,
}: {
  readonly unit: SizeUnit;
  readonly onSelect: (unit: SizeUnit) => void;
}) {
  return (
    <div className={styles['switch']} role="group" aria-label="Size units">
      <span className={styles['switchLabel']}>Size units</span>
      <UnitRadio unit="bytes" label="Bytes" checked={unit === 'bytes'} onSelect={onSelect} />
      <UnitRadio unit="kib" label="KiB" checked={unit === 'kib'} onSelect={onSelect} />
    </div>
  );
}

function UnitRadio({
  unit,
  label,
  checked,
  onSelect,
}: {
  readonly unit: SizeUnit;
  readonly label: string;
  readonly checked: boolean;
  readonly onSelect: (unit: SizeUnit) => void;
}) {
  const id = `fs-unit-${unit}`;
  return (
    <span className={styles['radio']}>
      <input
        id={id}
        type="radio"
        name="fs-size-unit"
        checked={checked}
        onChange={() => {
          onSelect(unit);
        }}
      />
      <label htmlFor={id}>{label}</label>
    </span>
  );
}
