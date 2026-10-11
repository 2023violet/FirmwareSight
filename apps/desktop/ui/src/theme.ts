/**
 * The theme choice, the palette it resolves to, and the one place that knows the difference.
 *
 * `ADR-0031` D1 keeps this to three words and one default. The stored value is the *choice*; the
 * palette is derived, and only ever derived from the choice plus what the platform says right now.
 * Storing the derived palette would turn every OS change into a preference write, and would make a
 * person who picked "System" lose that pick the first time their OS flipped after dark.
 *
 * Nothing here imports Tauri or the IPC bridge: the resolution has to be testable without a window,
 * and the answer has to be the same one the Rust side means by `theme`.
 */

export type Theme = 'system' | 'light' | 'dark';

/** Which palette is on screen. `system` is never a palette, so it is not one of these. */
export type ResolvedTheme = 'light' | 'dark';

/** The closed set, in the order the Settings page lists it. Mirrors `ThemePref::WORDS`. */
export const THEME_WORDS: readonly Theme[] = ['system', 'light', 'dark'];

/** The same set with the words a person reads. `ADR-0031` D1: three, and no fourth option. */
export const THEME_CHOICES: readonly {
  readonly word: Theme;
  readonly label: string;
}[] = [
  { word: 'system', label: 'System' },
  { word: 'light', label: 'Light' },
  { word: 'dark', label: 'Dark' },
];

/** The attribute the generated `tokens.css` selects on, and the only way a palette is chosen. */
export const THEME_ATTRIBUTE = 'data-fs-theme';

/**
 * A word from the shell, as a theme this window can apply.
 *
 * The shell reads a closed set and refuses anything else, so this is a narrowing for TypeScript
 * rather than a guess about a value that could arrive. Anything unrecognized becomes the default.
 */
export function asTheme(word: string | undefined | null): Theme {
  return THEME_WORDS.includes(word as Theme) ? (word as Theme) : 'system';
}

/**
 * What the platform answers right now, or `null` when it answers nothing at all.
 *
 * `null` is a real state and not a failure to handle: a WebView with no `matchMedia` cannot know
 * what the desktop is set to, and `ADR-0031` D4 says it runs light and says so rather than
 * inventing an answer.
 */
export function platformTheme(): ResolvedTheme | null {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
    return null;
  }
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

/** The palette a choice produces, given what the platform says. `system` is the only branch that asks. */
export function resolveTheme(theme: Theme, platform: ResolvedTheme | null): ResolvedTheme {
  if (theme === 'light' || theme === 'dark') {
    return theme;
  }
  return platform ?? 'light';
}

/**
 * Put the palette on the document, or take it off.
 *
 * Light is `:root`, so a light window carries no attribute: the absence is the fact, and one
 * selector cannot disagree with the other.
 */
export function applyTheme(root: HTMLElement, resolved: ResolvedTheme): void {
  if (resolved === 'dark') {
    root.setAttribute(THEME_ATTRIBUTE, 'dark');
  } else {
    root.removeAttribute(THEME_ATTRIBUTE);
  }
}
