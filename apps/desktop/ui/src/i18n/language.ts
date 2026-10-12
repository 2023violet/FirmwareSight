/**
 * The interface language: the choice, the language it resolves to, and the rule that connects them.
 *
 * `ADR-0032` D1 keeps this to three words and one default, and the shape mirrors `theme.ts` on
 * purpose: the stored value is the *choice*, the resolved value is derived from it plus what the
 * platform says, and the derived value is never stored. A person whose desktop language changes
 * while they read `English` must keep `English`.
 *
 * The platform is asked through `navigator.languages`, falling back to `navigator.language`,
 * because a list is what a browser actually reports and a single tag is what an older one reports.
 * Neither is a Tauri API: no command, no permission, no host access (`AGENTS.md` 7).
 */

export type Language = 'system' | 'zh-CN' | 'en';

/** Which catalog is on screen. `system` is never a catalog, so it is not one of these. */
export type ResolvedLanguage = 'en' | 'zh-CN';

/** The closed set, in the order the Settings page lists it. Mirrors `LanguagePref::WORDS`. */
export const LANGUAGE_WORDS: readonly Language[] = ['system', 'zh-CN', 'en'];

/**
 * The words a person reads beside each choice.
 *
 * `System` is English in both catalogs because it names the *behaviour*, not a language; the two
 * language options are named in their own language, which is the convention every settings UI a
 * reader has ever used follows and the reason a translated label would be worse than a foreign one.
 */
export const LANGUAGE_CHOICES: readonly {
  readonly word: Language;
  readonly label: string;
}[] = [
  { word: 'system', label: 'System' },
  { word: 'zh-CN', label: '简体中文' },
  { word: 'en', label: 'English' },
];

/** A word from the shell, as a language this window can apply. Unrecognized becomes the default. */
export function asLanguage(word: string | undefined | null): Language {
  return LANGUAGE_WORDS.includes(word as Language) ? (word as Language) : 'system';
}

/**
 * What the platform reports, most preferred first.
 *
 * An empty list is a real answer and not a failure to handle: a WebView with no `navigator` (a test
 * renderer, a page before scripts) cannot know what the desktop is set to, and `ADR-0032` D1 says
 * it runs English rather than inventing an answer.
 */
export function platformLanguages(
  navigatorLike: { languages?: readonly string[]; language?: string } | undefined,
): string[] {
  const listed = navigatorLike?.languages;
  if (Array.isArray(listed) && listed.length > 0) {
    return listed.filter((tag): tag is string => typeof tag === 'string');
  }
  const single = navigatorLike?.language;
  return typeof single === 'string' && single.length > 0 ? [single] : [];
}

/**
 * The primary subtag of a language range, lowercase: `zh-CN` and `zh_Hans` both answer `zh`.
 *
 * Only the primary subtag is consulted, deliberately. Distinguishing `zh-Hans` from `zh-Hant` would
 * be the start of a Traditional catalog this product does not have, and pretending otherwise is
 * the claim `ADR-0032` D1 refuses to make.
 */
function primary(tag: string): string {
  return tag.toLowerCase().split(/[-_]/)[0] ?? '';
}

/**
 * What the platform's own tags answer, or `null` when they answer nothing this build can show.
 *
 * Separated from `resolveLanguage` because the two are different facts, and the Settings page has to
 * say the right one: a reader on a German desktop is not on an English desktop, and a sentence that
 * calls their language English to their face is a false fact about their machine. `en` is what the
 * window runs in, and it is the fallback rather than the answer.
 */
export function pickPlatformLanguage(
  platform: readonly string[],
): ResolvedLanguage | null {
  for (const tag of platform) {
    const base = primary(tag);
    if (base === 'zh') {
      return 'zh-CN';
    }
    if (base === 'en') {
      return 'en';
    }
  }
  return null;
}

/**
 * The language a choice produces, given what the platform says.
 *
 * `zh-CN` and `en` ignore the platform entirely. `system` takes the first reported tag it
 * recognizes, and anything unrecognized — including an empty list — resolves to English, which is
 * the language the source is written in and therefore the only one that cannot be missing.
 */
export function resolveLanguage(
  language: Language,
  platform: readonly string[],
): ResolvedLanguage {
  if (language === 'en' || language === 'zh-CN') {
    return language;
  }
  return pickPlatformLanguage(platform) ?? 'en';
}

/** What goes in `lang` on the document element: the resolved language, never the choice. */
export function documentLanguage(resolved: ResolvedLanguage): string {
  return resolved;
}
