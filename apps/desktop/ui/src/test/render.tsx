/**
 * Render a page the way the shell renders it: inside the language provider.
 *
 * A page takes its words from the catalog, so mounting one bare fails in the provider lookup before
 * it can fail in the page — an error about the test harness that says nothing about the behaviour
 * under test.
 *
 * English is the default on purpose. The suite's locators name English text, and `ADR-0032` D8 gives
 * a test renderer no platform to read, so a default that resolved from the host would make an
 * unrelated assertion fail depending on the machine running it. A test that wants the other catalog
 * says so.
 */

import { render as renderBase, type RenderOptions } from '@testing-library/react';
import type { ReactElement, ReactNode } from 'react';

import { I18nProvider } from '../i18n';
import { resolveLanguage, type Language } from '../i18n/language';

export interface I18nRenderOptions extends RenderOptions {
  /** The choice, which may be `system`; `en` is what the locators were written against. */
  readonly language?: Language;
  /** What the platform reports, most preferred first. Only `system` consults it. */
  readonly platform?: readonly string[];
}

export function render(ui: ReactElement, options: I18nRenderOptions = {}) {
  const { language = 'en', platform = ['en'], ...rest } = options;
  const resolved = resolveLanguage(language, platform);
  const wrapper = ({ children }: { readonly children: ReactNode }) => (
    <I18nProvider language={language} resolved={resolved} platform={platform}>
      {children}
    </I18nProvider>
  );
  return renderBase(ui, { ...rest, wrapper });
}
