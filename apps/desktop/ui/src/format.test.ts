import { describe, expect, it } from 'vitest';

import { formatOptional, formatSize, truncateMiddle } from './format';

describe('value formatting', () => {
  it('groups byte counts without inventing a unit', () => {
    expect(formatSize(5432, 'bytes')).toBe('5,432 bytes');
    expect(formatSize(0, 'bytes')).toBe('0 bytes');
  });

  it('divides by 1024 for KiB, which is the definition the switch promises', () => {
    // 1 KiB = 1024 bytes exactly, so these two figures are the whole rule.
    expect(formatSize(1024, 'kib')).toBe('1.00 KiB');
    expect(formatSize(5432, 'kib')).toBe('5.30 KiB');
    expect(formatSize(60, 'kib')).toBe('0.0586 KiB');
  });

  it('never rounds a nonzero count down to a zero', () => {
    // Four bytes is a real size, and `0 KiB` on the screen would contradict the artifact.
    expect(formatSize(4, 'kib')).toBe('0.00391 KiB');
    expect(formatSize(1, 'kib')).toBe('0.000977 KiB');
  });

  it('keeps a measured zero a zero in both units', () => {
    expect(formatSize(0, 'bytes')).toBe('0 bytes');
    expect(formatSize(0, 'kib')).toBe('0.00 KiB');
  });

  it('says Unknown instead of showing zero for a missing total', () => {
    // A missing figure rendered as `0` reads like a measured zero (DESIGN.md 5, US-001 addendum 4),
    // and it must not become `0 KiB` just because the reader switched units.
    expect(formatSize(null, 'bytes')).toBe('Unknown');
    expect(formatSize(null, 'kib')).toBe('Unknown');
    expect(formatOptional(null)).toBe('-');
  });

  it('shortens the middle so both ends of a hash stay visible', () => {
    expect(truncateMiddle('0123456789abcdef', 4)).toBe('0123...cdef');
    expect(truncateMiddle('short', 4)).toBe('short');
  });
});
