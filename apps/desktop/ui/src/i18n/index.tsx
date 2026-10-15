/**
 * The runtime: one lookup, one interpolation rule, one place that knows what a missing key means.
 *
 * `ADR-0032` D2 chose a typed catalog over a library, so the two things a library would have done
 * are done here in the smallest honest form: substitution of named `{placeholders}`, and a fallback
 * that records what it fell back for. Key completeness is not handled here — the compiler handles
 * it, which is the point of the choice.
 *
 * No HTML, no rich text, no sentence built by concatenation. A value from a catalog is a string and
 * is rendered as a text child; a string that needs a number, a name or a code takes a placeholder,
 * which is also what keeps a file name, an error code or a real engineering figure intact across
 * both languages (v0.2 §97, `ADR-0032` D4).
 */

import { createContext, useContext, useMemo, type ReactNode } from 'react';

import { en, type Catalog, type Namespace } from './en';
import type { CatalogShape } from './types';
import { zh } from './zh';
import { setDialogLanguage } from '../ipc/dialogWords';
import type { Language, ResolvedLanguage } from './language';

/** Parameters for one interpolation. Values are shown exactly as given. */
export type Params = Readonly<Record<string, string | number>>;

const PLACEHOLDER = /\{(\w+)\}/g;

/** The placeholder names a template asks for, in first-appearance order, without duplicates. */
export function placeholders(template: string): string[] {
  const found: string[] = [];
  for (const match of template.matchAll(PLACEHOLDER)) {
    if (match[1] !== undefined && !found.includes(match[1])) {
      found.push(match[1]);
    }
  }
  return found;
}

/**
 * Replace `{name}` with a parameter, and leave a placeholder alone when nothing supplies it.
 *
 * A hole that renders as `{count}` is visible and is a bug someone can see; a hole that renders as
 * nothing is a sentence that quietly changes its meaning.
 */
export function fill(template: string, params?: Params): string {
  if (params === undefined) {
    return template;
  }
  return template.replace(PLACEHOLDER, (whole, name: string) =>
    Object.hasOwn(params, name) ? String(params[name]) : whole,
  );
}

const catalogs: Record<ResolvedLanguage, CatalogShape> = {
  en,
  'zh-CN': zh,
};

/** Keys that had to fall back in this session. Read by the tests; empty in a correct build. */
const fellBack: string[] = [];

/** What fell back to English since the module loaded. */
export function fallbacks(): readonly string[] {
  return fellBack;
}

/** Forget the record — for a test that wants to assert the empty case honestly. */
export function clearFallbacks(): void {
  fellBack.length = 0;
}

/** The lookup a component uses. `ns` and `key` are both constrained to the catalog's own shape. */
export type Translate = <N extends Namespace>(
  ns: N,
  key: keyof Catalog[N] & string,
  params?: Params,
) => string;

/**
 * A translator bound to one resolved language.
 *
 * A missing key cannot happen through the type, and if one arrives anyway — a key built at runtime,
 * a catalog edited by hand — the reader gets the English text, which always exists because it is the
 * source language, and the event is recorded rather than swallowed.
 */
export function translate(resolved: ResolvedLanguage): Translate {
  return (ns, key, params) => {
    const table = catalogs[resolved][ns] as Record<string, string> | undefined;
    const value = table?.[key as string];
    if (value !== undefined) {
      return fill(value, params);
    }
    const source = en[ns] as Record<string, string> | undefined;
    const english = source?.[key as string];
    if (english !== undefined) {
      const name = `${String(ns)}.${String(key)}`;
      if (!fellBack.includes(name)) {
        fellBack.push(name);
      }
      return fill(english, params);
    }
    return String(key);
  };
}

/**
 * Whether this build has interface copy for an error code.
 *
 * The panel asks rather than trying a lookup and inspecting the answer, because "the catalog knows
 * this code" is a fact the page states to the reader: knowing it, it says what the code means in the
 * reader's language; not knowing it, it says that too, and leaves the engine's own sentence in view
 * instead of inventing a paraphrase (`ADR-0032` D6).
 */
export function knowsError(code: string): boolean {
  return Object.hasOwn(en.errors, code);
}

export interface I18n {
  /** The lookup, bound to the catalog on screen. */
  readonly t: Translate;
  /** The choice the person made, which may be `system`. */
  readonly language: Language;
  /** The catalog actually on screen. Never `system`. */
  readonly resolved: ResolvedLanguage;
  /** What the platform reported, most preferred first. Empty when it reports nothing. */
  readonly platform: readonly string[];
}

const I18nContext = createContext<I18n | null>(null);

export function I18nProvider({
  language,
  resolved,
  platform,
  children,
}: {
  readonly language: Language;
  readonly resolved: ResolvedLanguage;
  readonly platform: readonly string[];
  readonly children: ReactNode;
}) {
  const value = useMemo<I18n>(() => {
    // Native dialogs speak the language on screen too; see `ipc/dialogWords.ts`.
    setDialogLanguage(resolved);
    return { t: translate(resolved), language, resolved, platform };
  }, [language, resolved, platform]);
  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

/**
 * The shell's translator.
 *
 * Every page is rendered inside the provider, so the context is always there; the throw is a
 * component bug caught in the first render of a test, not a state a reader can reach.
 */
export function useI18n(): I18n {
  const value = useContext(I18nContext);
  if (value === null) {
    throw new Error('useI18n was called outside the I18nProvider');
  }
  return value;
}
