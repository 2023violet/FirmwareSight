import { describe, expect, it } from 'vitest';

import { formatBytes, formatOptional, truncateMiddle } from './format';

describe('value formatting', () => {
  it('groups byte counts without inventing a unit', () => {
    expect(formatBytes(5432)).toBe('5,432 bytes');
    expect(formatBytes(0)).toBe('0 bytes');
  });

  it('says unknown instead of showing zero for a missing total', () => {
    // A missing figure rendered as `0` reads like a measured zero (DESIGN.md 5).
    expect(formatBytes(null)).toBe('unknown');
    expect(formatOptional(null)).toBe('-');
  });

  it('shortens the middle so both ends of a hash stay visible', () => {
    expect(truncateMiddle('0123456789abcdef', 4)).toBe('0123...cdef');
    expect(truncateMiddle('short', 4)).toBe('short');
  });
});
