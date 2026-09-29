/**
 * Presentation helpers only.
 *
 * Nothing here decides a state, a verdict or a fact - the shell sends all of those. Byte counts
 * are formatted with an explicit locale so a build server in another locale cannot make a
 * rendered value differ from the one under test.
 *
 * `formatSize` is the only place a unit is applied, and applying it changes this string and nothing
 * else: no re-parse, no new snapshot, no stored row, no golden (US-001 addendum 3). Addresses,
 * offsets, hashes, counts and ordinals never pass through it.
 */

const BYTES = new Intl.NumberFormat('en-US');
/**
 * Three significant figures, because a section of four bytes is not zero: rounding a nonzero count
 * to `0 KiB` would put a number on the screen that the artifact contradicts. `maximumFractionDigits`
 * alone would do exactly that, so the precision follows the value instead of the decimal place.
 */
const KI_B = new Intl.NumberFormat('en-US', {
  minimumSignificantDigits: 3,
  maximumSignificantDigits: 3,
});

/** The two units US-001 offers. `bytes` is the default, so the screen opens on the raw figure. */
export type SizeUnit = 'bytes' | 'kib';

export const BYTES_PER_KIB = 1024;

export function formatSize(value: number | null, unit: SizeUnit): string {
  if (value === null) {
    return 'Unknown';
  }
  if (unit === 'kib') {
    return `${KI_B.format(value / BYTES_PER_KIB)} KiB`;
  }
  return `${BYTES.format(value)} bytes`;
}

/** A value that is genuinely absent must never render as `0` (DESIGN.md 5, Diff Row rule). */
export function formatOptional(value: string | null): string {
  return value === null || value.length === 0 ? '-' : value;
}

export function truncateMiddle(value: string, keep = 12): string {
  if (value.length <= keep * 2 + 1) {
    return value;
  }
  return `${value.slice(0, keep)}...${value.slice(-keep)}`;
}
