/**
 * Presentation helpers only.
 *
 * Nothing here decides a state, a verdict or a fact - the shell sends all of those. Byte counts
 * are formatted with an explicit locale so a build server in another locale cannot make a
 * rendered value differ from the one under test.
 */

const BYTES = new Intl.NumberFormat('en-US');

export function formatBytes(value: number | null): string {
  return value === null ? 'unknown' : `${BYTES.format(value)} bytes`;
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
