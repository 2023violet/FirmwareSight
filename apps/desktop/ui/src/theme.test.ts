/**
 * The theme resolution, apart from any window.
 *
 * `ADR-0031` D4 puts three claims in this file rather than in a screenshot: a stored `light` or
 * `dark` ignores the platform, `system` follows it, and a platform that answers nothing resolves to
 * light rather than to a guess.
 */

import { describe, expect, it } from 'vitest';

import {
  applyTheme,
  asTheme,
  resolveTheme,
  THEME_ATTRIBUTE,
  THEME_WORDS,
} from './theme';

describe('the theme words', () => {
  it('are the three the shell offers, and no fourth', () => {
    expect(THEME_WORDS).toEqual(['system', 'light', 'dark']);
  });

  it('narrow an unknown word to the default instead of to a palette', () => {
    expect(asTheme('dark')).toBe('dark');
    expect(asTheme('system')).toBe('system');
    // A word from a newer build is not a theme this build can paint, and silently picking one
    // would store a choice the person never made.
    for (const word of ['auto', 'HIGH_CONTRAST', '', 'sepia', undefined, null]) {
      expect(asTheme(word as string | undefined | null)).toBe('system');
    }
  });
});

describe('resolving a choice to a palette', () => {
  it('ignores the platform when the person named one', () => {
    expect(resolveTheme('light', 'dark')).toBe('light');
    expect(resolveTheme('dark', 'light')).toBe('dark');
  });

  it('follows the platform only while the choice is system', () => {
    expect(resolveTheme('system', 'dark')).toBe('dark');
    expect(resolveTheme('system', 'light')).toBe('light');
  });

  it('falls back to light when the platform cannot answer', () => {
    expect(resolveTheme('system', null)).toBe('light');
  });
});

describe('putting a palette on the document', () => {
  it('marks dark and leaves light as the absence of a mark', () => {
    const root = document.createElement('div');
    applyTheme(root, 'dark');
    expect(root.getAttribute(THEME_ATTRIBUTE)).toBe('dark');
    applyTheme(root, 'light');
    expect(root.hasAttribute(THEME_ATTRIBUTE)).toBe(false);
  });

  it('is idempotent, because a re-render is not a theme change', () => {
    const root = document.createElement('div');
    applyTheme(root, 'dark');
    applyTheme(root, 'dark');
    expect(root.getAttribute(THEME_ATTRIBUTE)).toBe('dark');
  });
});
