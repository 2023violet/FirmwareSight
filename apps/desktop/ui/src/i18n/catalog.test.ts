/**
 * The catalog contract, checked from both sides.
 *
 * `ADR-0032` D2 chose a typed in-repo catalog so that a missing translation would be a build failure
 * rather than a runtime log. That is true — `tsc --noEmit` is a gate step, and `zh.ts` is typed
 * against `en` — but this file is where the *other* halves of the same promise are held to, the ones
 * a key-by-key type cannot see: that the two catalogs ask for the same placeholders, that no value is
 * markup, that the machine vocabulary survives translation, and that nothing fell back while the
 * suite ran.
 *
 * A `{count}` that exists in English and not in Chinese is the failure mode this is really about: the
 * sentence renders, looks finished, and has lost a number.
 */

import { afterEach, describe, expect, it } from 'vitest';

import { en, type Namespace } from './en';
import type { CatalogShape } from './types';
import { zh } from './zh';
import { clearFallbacks, fallbacks, fill, placeholders, translate } from './index';

const namespaces = Object.keys(en) as [Namespace, ...Namespace[]];

/** Every `namespace.key` the English catalog holds, sorted, so a diff names the key rather than a line. */
function keysOf(catalog: CatalogShape): string[] {
  return (Object.keys(catalog) as Namespace[]).flatMap((ns) =>
    Object.keys(catalog[ns]).map((key) => `${ns}.${key}`),
  );
}

describe('the two catalogs', () => {
  it('have exactly the same keys', () => {
    // The compiler enforces this too; this says it in a form that names the key when it moves.
    expect(keysOf(zh).sort()).toEqual(keysOf(en as unknown as CatalogShape).sort());
  });

  it('have no empty value', () => {
    for (const ns of namespaces) {
      for (const [key, value] of Object.entries(en[ns])) {
        expect(value.trim(), `en.${ns}.${key}`).not.toBe('');
      }
      for (const [key, value] of Object.entries(zh[ns])) {
        expect(value.trim(), `zh.${ns}.${key}`).not.toBe('');
      }
    }
  });

  it('ask for the same placeholders in the same set', () => {
    for (const ns of namespaces) {
      for (const key of Object.keys(en[ns])) {
        expect(
          placeholders(zh[ns as Namespace][key as never] as string).sort(),
          `${ns}.${key}`,
        ).toEqual(placeholders(en[ns][key as never] as string).sort());
      }
    }
  });

  it('carry no markup', () => {
    // A value is a string rendered as a text child. Allowing `<b>` here would put catalog text on the
    // path to `dangerouslySetInnerHTML`, and a translated fragment is a broken sentence waiting to
    // happen (`ADR-0032` D4).
    for (const ns of namespaces) {
      for (const [key, value] of Object.entries(en[ns])) {
        expect(value, `en.${ns}.${key}`).not.toMatch(/[<>]/);
      }
      for (const [key, value] of Object.entries(zh[ns])) {
        expect(value, `zh.${ns}.${key}`).not.toMatch(/[<>]/);
      }
    }
  });

  it('are different sentences, not one language wearing another', () => {
    // The guard against a Chinese catalog that was quietly copied and pasted: enough of it has to
    // actually differ, or the second file is decoration.
    const same = namespaces.flatMap((ns) =>
      Object.keys(en[ns]).filter((key) => en[ns][key as never] === zh[ns][key as never]),
    );
    // Identical strings are legitimate for names, units and machine vocabulary (`KiB`, `FirmwareSight`,
    // `System`, `English`), which is why this is a ceiling rather than a ban.
    expect(same.length).toBeLessThan(namespaces.reduce((n, ns) => n + Object.keys(en[ns]).length, 0) / 4);
  });

  it('keep the machine vocabulary in both languages', () => {
    // `ADR-0032` D5: a verdict name, an evidence class, a product name and a unit are data this
    // product identifies by. Wherever the English carries one, the Chinese has to carry the same
    // word — a translated `PASS` would rename a Gate verdict.
    const kept = [
      'PASS',
      'REVIEW',
      'BLOCK',
      'UNKNOWN',
      'N/A',
      'Observed',
      'Derived',
      'Declared',
      'FirmwareSight',
      'KiB',
      'settings.json',
    ];
    for (const ns of namespaces) {
      for (const [key, value] of Object.entries(en[ns])) {
        for (const word of kept) {
          if (value.includes(word)) {
            expect(zh[ns][key as never] as string, `${ns}.${key} keeps ${word}`).toContain(word);
          }
        }
      }
    }
  });
});

describe('interpolation', () => {
  it('substitutes named placeholders', () => {
    expect(fill('{label}: {value}', { label: 'Flash', value: '128 KiB' })).toBe(
      'Flash: 128 KiB',
    );
  });

  it('takes a number where the sentence asks for one', () => {
    expect(fill('{count} findings', { count: 0 })).toBe('0 findings');
  });

  it('leaves a hole it was not given visible', () => {
    // A `{token}` on screen is a bug a reader can report; an empty hole is a sentence that quietly
    // changed its meaning.
    expect(fill('Release {id} at {offset}', { id: 'rel-1' })).toBe('Release rel-1 at {offset}');
  });

  it('leaves the template alone when nothing was passed', () => {
    expect(fill('{label} page')).toBe('{label} page');
    expect(fill('no holes')).toBe('no holes');
  });

  it('replaces every appearance of one name', () => {
    expect(fill('{n} of {n}', { n: 3 })).toBe('3 of 3');
  });
});

describe('a lookup', () => {
  afterEach(() => {
    clearFallbacks();
  });

  it('reads the catalog it was bound to', () => {
    expect(translate('en')('settings', 'heading')).toBe('Settings');
    expect(translate('zh-CN')('settings', 'heading')).toBe('设置');
  });

  it('interpolates through the same call', () => {
    expect(translate('zh-CN')('common', 'nav.pageAria', { label: '设置' })).toBe('设置 页面');
  });

  it('resolves every key in the catalog, in both languages, without falling back', () => {
    // This is the detectability requirement: a missing translation cannot be invisible, so the suite
    // walks the whole catalog and fails on the record rather than on a console line nobody reads.
    //
    // The parameters are taken from each template's own holes instead of a hand-kept list, because a
    // list is a second place a key can be forgotten — which is the failure this file exists to catch.
    for (const language of ['en', 'zh-CN'] as const) {
      clearFallbacks();
      const t = translate(language);
      for (const ns of namespaces) {
        for (const key of Object.keys(en[ns])) {
          const template = en[ns][key as never] as string;
          const params: Record<string, string> = {};
          for (const name of placeholders(template)) {
            params[name] = 'x';
          }
          expect(
            t(ns, key as never, params),
            `${language}.${ns}.${key}`,
          ).not.toMatch(/\{[a-z]\w*\}/i);
        }
      }
      expect(fallbacks()).toEqual([]);
    }
  });

  it('says the key when neither catalog knows it', () => {
    // A runtime-built key that does not exist is a programming error, and the honest rendering is the
    // name of the hole rather than a blank or a guessed sentence.
    expect(translate('zh-CN')('settings', 'no.such.key' as never)).toBe('no.such.key');
  });
});
