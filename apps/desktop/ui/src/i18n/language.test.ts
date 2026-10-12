/**
 * The resolver: what the three words mean, and what the platform is allowed to answer.
 *
 * `ADR-0032` D1's whole claim is that this window has two catalogs and one fallback, and that the
 * fallback is a fact about this build rather than a read of the reader's machine. These tests are the
 * narrowest place that claim can be checked — no DOM, no shell, no provider.
 *
 * The host answer is passed in, never scraped from the runner: a resolver whose test depended on
 * whatever `navigator` the environment happened to provide would pass or fail by geography.
 */

import { describe, expect, it } from 'vitest';

import {
  asLanguage,
  documentLanguage,
  LANGUAGE_CHOICES,
  LANGUAGE_WORDS,
  pickPlatformLanguage,
  platformLanguages,
  resolveLanguage,
} from './language';

describe('the closed set', () => {
  it('offers three words, in the order the page lists them', () => {
    // Mirrors `LanguagePref::WORDS` on the Rust side; a fourth word here without one there would be
    // a control the shell refuses to store.
    expect(LANGUAGE_WORDS).toEqual(['system', 'zh-CN', 'en']);
    expect(LANGUAGE_CHOICES.map((entry) => entry.word)).toEqual([...LANGUAGE_WORDS]);
  });

  it('names each choice in the language it switches to', () => {
    // `System` is English in both catalogs because it names a behaviour, not a language; the two
    // named choices are written the way a reader would recognise them on a screen they cannot read.
    expect(LANGUAGE_CHOICES.map((entry) => entry.label)).toEqual([
      'System',
      '简体中文',
      'English',
    ]);
  });

  it('makes no claim about Traditional Chinese', () => {
    // There is one Chinese catalog. A label or a word that implied a second would be the promise
    // `ADR-0032` D1 refuses to make.
    const joined = LANGUAGE_WORDS.join(' ') + LANGUAGE_CHOICES.map((e) => e.label).join(' ');
    expect(joined).not.toContain('Hant');
    expect(joined).not.toContain('繁體');
    expect(joined).not.toContain('Traditional');
  });

  it('narrows a word from the shell, and falls back to the default for anything else', () => {
    expect(asLanguage('zh-CN')).toBe('zh-CN');
    expect(asLanguage('en')).toBe('en');
    expect(asLanguage('system')).toBe('system');
    // The shell refuses these before they arrive; the narrowing is for TypeScript's benefit.
    expect(asLanguage('zh')).toBe('system');
    expect(asLanguage('zh-TW')).toBe('system');
    expect(asLanguage('en-US')).toBe('system');
    expect(asLanguage('')).toBe('system');
    expect(asLanguage(undefined)).toBe('system');
    expect(asLanguage(null)).toBe('system');
  });
});

describe('what the platform reports', () => {
  it('takes the list when there is one, because a list is what a browser answers', () => {
    expect(platformLanguages({ languages: ['zh-CN', 'en-US'] })).toEqual(['zh-CN', 'en-US']);
  });

  it('falls back to the single tag when the host only has one', () => {
    expect(platformLanguages({ language: 'fr-FR' })).toEqual(['fr-FR']);
  });

  it('answers nothing when the host says nothing', () => {
    // Empty is a state, not a failure: a renderer with no `navigator` cannot know, and the page says
    // so rather than inventing an answer.
    expect(platformLanguages(undefined)).toEqual([]);
    expect(platformLanguages({})).toEqual([]);
    expect(platformLanguages({ languages: [] })).toEqual([]);
    expect(platformLanguages({ language: '' })).toEqual([]);
    expect(platformLanguages({ languages: [''], language: 'de-DE' })).toEqual(['']);
  });

  it('drops anything that is not a string', () => {
    const listed = ['zh-CN', 42, null] as unknown as string[];
    expect(platformLanguages({ languages: listed })).toEqual(['zh-CN']);
  });
});

describe('which language the desktop asked for', () => {
  it('reads only the primary subtag', () => {
    // Distinguishing `zh-Hans` from `zh-Hant` would be the start of a catalog this build does not
    // have, so the region and script subtags are not consulted at all.
    expect(pickPlatformLanguage(['zh-CN'])).toBe('zh-CN');
    expect(pickPlatformLanguage(['zh-TW'])).toBe('zh-CN');
    expect(pickPlatformLanguage(['zh-HK'])).toBe('zh-CN');
    expect(pickPlatformLanguage(['zh-Hans-CN'])).toBe('zh-CN');
    expect(pickPlatformLanguage(['zh_Hant'])).toBe('zh-CN');
    expect(pickPlatformLanguage(['ZH-cn'])).toBe('zh-CN');
    expect(pickPlatformLanguage(['en-GB'])).toBe('en');
    expect(pickPlatformLanguage(['en'])).toBe('en');
  });

  it('answers null rather than a language for a desktop it cannot show', () => {
    // The false sentence would be "your desktop language is English"; `null` is what makes the page
    // able to say "this build has no catalog for it" instead.
    expect(pickPlatformLanguage(['de-DE'])).toBeNull();
    expect(pickPlatformLanguage(['ja-JP', 'ko-KR'])).toBeNull();
    expect(pickPlatformLanguage([])).toBeNull();
  });

  it('takes the first tag it recognizes, not the first tag', () => {
    // Preference order is the host's, and a reader who lists German first and Chinese second has
    // asked for Chinese over nothing.
    expect(pickPlatformLanguage(['de-DE', 'zh-CN'])).toBe('zh-CN');
    expect(pickPlatformLanguage(['en-US', 'zh-CN'])).toBe('en');
    expect(pickPlatformLanguage(['zh-CN', 'en-US'])).toBe('zh-CN');
  });
});

describe('what a choice resolves to', () => {
  it('consults the platform only for System', () => {
    expect(resolveLanguage('zh-CN', ['en-US'])).toBe('zh-CN');
    expect(resolveLanguage('en', ['zh-CN'])).toBe('en');
    expect(resolveLanguage('system', ['zh-CN'])).toBe('zh-CN');
    expect(resolveLanguage('system', ['en-US'])).toBe('en');
  });

  it('runs the source language when there is nothing to follow', () => {
    // English is the language the catalog is written in, so it is the only one that cannot be
    // missing. This is a fallback, and the Settings page is required to say it as one.
    expect(resolveLanguage('system', [])).toBe('en');
    expect(resolveLanguage('system', ['de-DE'])).toBe('en');
  });

  it('never resolves to the choice', () => {
    // `system` is not a catalog; a resolved value of `system` would index a table that does not exist.
    for (const word of LANGUAGE_WORDS) {
      for (const tags of [[], ['zh-CN'], ['en-US'], ['de-DE']]) {
        expect(['en', 'zh-CN']).toContain(resolveLanguage(word, tags));
      }
    }
  });
});

describe('what the document declares', () => {
  it('declares the catalog on screen', () => {
    // `lang` is what a screen reader picks a voice from and what a translation tool reads, so a
    // Chinese window that declares `en` is a defect the reader meets immediately.
    expect(documentLanguage('zh-CN')).toBe('zh-CN');
    expect(documentLanguage('en')).toBe('en');
  });
});
