/**
 * This application's own settings - and nothing the Gate judges.
 *
 * Three kinds of state live in three places, and this page holds only the first:
 *
 * - **Application preferences** (`settings.json`, this page): how this window labels a number, what
 *   colour it is, which language it speaks, and where the next start opens. They belong to the
 *   machine and to the person sitting at it.
 * - **Project policy** (`firmwaresight.toml`, the Release page): what the Gate is required to check.
 *   It travels with the project, and the Settings page never writes into it.
 * - **Session state** (React, on the workflow pages): which build is staged, which comparison is
 *   open. It ends when the window closes.
 *
 * The first kind is deliberately powerless. A preference reaches no Core call, no Gate input, no
 * run id, no release id, no portable document and no bundle: `release_gate.rs`'s
 * `a_stored_preference_moves_no_gate_identity_no_verdict_and_no_policy_fingerprint` runs the Gate
 * twice across a saved preference and compares the payloads, because a run id is the digest of the
 * canonical input and a preference that got near it would move it.
 *
 * Only controls that do something today are rendered. The theme is here because `ADR-0031` gave this
 * build a second palette, and the language is here because `ADR-0032` gave it a second catalog — and
 * because this page is itself rendered from that catalog, so a reader who changes it watches the
 * words under their hand change. A disabled switch that goes nowhere would teach the reader that
 * nothing on this page works (`AGENTS.md` 11, `DESIGN.md` 5).
 */

import { useState } from 'react';

import type { SizeUnit } from './format';
import styles from './Settings.module.css';
import { ErrorPanel } from './components/ErrorPanel';
import { SizeUnitSwitch } from './components/SizeUnitSwitch';
import { LANGUAGE_CHOICES, type Language, type ResolvedLanguage } from './i18n/language';
import { useI18n, type Translate } from './i18n';
import { THEME_CHOICES, type ResolvedTheme, type Theme } from './theme';
import type { ErrorEnvelopeDto, PreferenceRecoveryDto, PreferencesDto } from './ipc/types';

/**
 * Where a start opens, in the two forms this build can actually honour.
 *
 * "Last page I was on" is not among them: a restored page would bring back the *look* of an earlier
 * Gate run while the session behind it is gone, and a reader who mistook that for a current verdict
 * would be shown a false fact. A static choice cannot do that, so a static choice is what is offered.
 */
const START_PAGES: readonly { readonly word: string; readonly labelKey: 'start.analyze' | 'start.history' }[] =
  [
    { word: 'analyze', labelKey: 'start.analyze' },
    { word: 'history', labelKey: 'start.history' },
  ];

/**
 * The one sentence that says why the values on this page are what they are.
 *
 * Every branch is a fact about the file, not a mood: what was found, what is being used instead, and
 * whether the bytes that could not be read still exist somewhere.
 */
function recoveryNote(recovery: PreferenceRecoveryDto, keptCopy: boolean, t: Translate): string | null {
  switch (recovery) {
    case 'read':
      return null;
    case 'missing':
      return t('settings', 'recovery.missing');
    case 'corrupted':
      return keptCopy
        ? t('settings', 'recovery.corruptedKept')
        : t('settings', 'recovery.corruptedPlain');
    case 'futureVersion':
      return t('settings', 'recovery.futureVersion');
    case 'nowhereToStore':
      return t('settings', 'recovery.nowhereToStore');
  }
}

/** The palette words, in the language on screen. `system` is never one of them. */
function themeWord(theme: Theme, t: Translate): string {
  return t('settings', `theme.${theme}`);
}

/**
 * What the theme choice is actually doing, said as a fact about this window.
 *
 * The parenthetical belongs here rather than inside the "System" label because it describes the
 * desktop, not the choice: a window set to `light` on a dark desktop must not tell the reader that
 * System is currently light. And where the desktop cannot be read at all, saying so beats the
 * default wearing a confident name (`ADR-0031` D4).
 */
function themeFact(theme: Theme, platform: ResolvedTheme | null, t: Translate): string {
  if (theme !== 'system') {
    return platform === null
      ? t('settings', 'appearance.themeChosenUnknown', { theme: themeWord(theme, t) })
      : t('settings', 'appearance.themeChosen', {
          theme: themeWord(theme, t),
          platform: t('settings', `platform.${platform}`),
        });
  }
  return platform === null
    ? t('settings', 'appearance.themeSystemUnknown')
    : t('settings', 'appearance.themeSystem', {
        platform: t('settings', `platform.${platform}`),
      });
}

/**
 * What the language choice is actually doing.
 *
 * `system` is the one branch that can say something false, and it has two ways to be false: the
 * window may be unable to ask the desktop anything, or the desktop may answer in a language this
 * build has no catalog for. Neither is "your desktop language is English", which is what a sentence
 * built from the *fallback* would claim, and `ADR-0032` D4's rule for the palette applies here
 * unchanged — where the platform cannot be read, saying so beats the default wearing a confident
 * name.
 *
 * There is also no live signal to promise: no browser event reports a desktop language change, so the
 * contract this sentence is allowed to imply is the one in `ADR-0032` D8 — read at this start, and
 * re-read when the choice returns to System.
 */
function languageFact(
  language: Language,
  desktop: ResolvedLanguage | null,
  canReadPlatform: boolean,
  t: Translate,
): string {
  const nameOf = (word: ResolvedLanguage | Language): string =>
    LANGUAGE_CHOICES.find((entry) => entry.word === word)?.label ?? word;
  if (language !== 'system') {
    return t('settings', 'appearance.languageChosen', { language: nameOf(language) });
  }
  if (!canReadPlatform) {
    return t('settings', 'appearance.languageSystemUnknown');
  }
  return desktop === null
    ? t('settings', 'appearance.languageSystemNoCatalog')
    : t('settings', 'appearance.languageSystem', { platform: nameOf(desktop) });
}

export function Settings({
  preferences,
  error,
  note,
  unit,
  onUnitChange,
  theme,
  platformPalette,
  onThemeChange,
  language,
  desktopLanguage,
  onLanguageChange,
  onStartPageChange,
  onReset,
  onGoToHelp,
}: {
  readonly preferences: PreferencesDto | null;
  readonly error: ErrorEnvelopeDto | null;
  /** What the shell agreed to, in its own words. Absent unless an action actually succeeded. */
  readonly note: string | null;
  readonly unit: SizeUnit;
  readonly onUnitChange: (unit: SizeUnit) => void;
  /** The stored choice, which for `system` is not a palette. */
  readonly theme: Theme;
  /** What the desktop says, or `null` when this window cannot ask it. */
  readonly platformPalette: ResolvedTheme | null;
  readonly onThemeChange: (theme: Theme) => void;
  /** The stored choice, which for `system` is not a catalog. */
  readonly language: Language;
  /**
   * Which language the platform's own tags named, or `null` when they named none this build has.
   *
   * Deliberately not the window's resolved language: the resolved value is `en` both when the desktop
   * says English and when the desktop says something this catalog set cannot show, and the two need
   * different sentences.
   */
  readonly desktopLanguage: ResolvedLanguage | null;
  readonly onLanguageChange: (language: Language) => void;
  readonly onStartPageChange: (word: string) => void;
  readonly onReset: () => void;
  readonly onGoToHelp: () => void;
}) {
  const { t, platform } = useI18n();
  /** The reset decision is asked on the page and answered there, in the same shape Release asks a replace. */
  const [confirming, setConfirming] = useState(false);
  const stored = preferences !== null ? preferences.fileName : 'settings.json';
  const startupWord = preferences?.startupPage ?? 'analyze';

  return (
    <main className={styles['page']}>
      <header className={styles['header']}>
        <h1>{t('settings', 'heading')}</h1>
        <p className={styles['subhead']}>{t('settings', 'intro')}</p>
      </header>

      {error !== null ? (
        <ErrorPanel
          envelope={error}
          label={t('settings', 'errorLabel')}
          heading={t('settings', 'errorHeading')}
        />
      ) : null}

      {preferences === null ? (
        <p className={styles['status']} role="status">
          {t('settings', 'waiting')}
        </p>
      ) : null}

      {note === null ? null : (
        <p className={styles['status']} role="status" aria-live="polite">
          {note}
        </p>
      )}

      <section className={styles['section']} aria-label={t('settings', 'general.heading')}>
        <h2>{t('settings', 'general.heading')}</h2>
        {preferences === null ? null : (
          <p className={styles['facts']}>{t('settings', 'general.whereTheyLive')}</p>
        )}
        {preferences === null ? null : (
          <p className={styles['facts']}>
            {t('settings', 'document.heading')}{' '}
            <span className={styles['mono']}>{stored}</span>
          </p>
        )}
        {preferences === null ? null : (
          <p className={styles['note']}>
            {recoveryNote(preferences.recovery, preferences.keptUnreadableCopy, t)}
          </p>
        )}
        <p className={styles['note']}>{t('settings', 'general.localFirst')}</p>
        <div className={styles['actions']}>
          <button type="button" className={styles['control']} onClick={onGoToHelp}>
            {t('settings', 'general.openHelp')}
          </button>
        </div>
      </section>

      <section className={styles['section']} aria-label={t('settings', 'appearance.heading')}>
        <h2>{t('settings', 'appearance.heading')}</h2>
        <p className={styles['note']}>{t('settings', 'appearance.unitFact')}</p>
        <SizeUnitSwitch
          unit={unit}
          onSelect={(next) => {
            onUnitChange(next);
          }}
        />
        <p className={styles['note']}>{t('settings', 'appearance.themeFact')}</p>
        <div className={styles['radios']} role="radiogroup" aria-label={t('settings', 'appearance.themeGroup')}>
          {THEME_CHOICES.map((entry) => (
            <span key={entry.word} className={styles['radio']}>
              <input
                id={`fs-theme-${entry.word}`}
                type="radio"
                name="fs-theme"
                checked={theme === entry.word}
                onChange={() => {
                  onThemeChange(entry.word);
                }}
              />
              <label htmlFor={`fs-theme-${entry.word}`}>{themeWord(entry.word, t)}</label>
            </span>
          ))}
        </div>
        <p className={styles['note']}>{themeFact(theme, platformPalette, t)}</p>
        <p className={styles['note']}>{t('settings', 'appearance.languageFact')}</p>
        <p className={styles['note']}>{t('settings', 'appearance.languageIntro')}</p>
        <div
          className={styles['radios']}
          role="radiogroup"
          aria-label={t('settings', 'appearance.languageGroup')}
        >
          {LANGUAGE_CHOICES.map((entry) => (
            <span key={entry.word} className={styles['radio']}>
              <input
                id={`fs-language-${entry.word}`}
                type="radio"
                name="fs-language"
                checked={language === entry.word}
                onChange={() => {
                  onLanguageChange(entry.word);
                }}
              />
              <label htmlFor={`fs-language-${entry.word}`}>{entry.label}</label>
            </span>
          ))}
        </div>
        <p className={styles['note']}>
          {languageFact(language, desktopLanguage, platform.length > 0, t)}
        </p>
      </section>

      <section className={styles['section']} aria-label={t('settings', 'start.heading')}>
        <h2>{t('settings', 'start.heading')}</h2>
        <p className={styles['note']}>{t('settings', 'start.fact')}</p>
        <div className={styles['radios']} role="radiogroup" aria-label={t('settings', 'start.group')}>
          {START_PAGES.map((entry) => (
            <span key={entry.word} className={styles['radio']}>
              <input
                id={`fs-start-${entry.word}`}
                type="radio"
                name="fs-start-page"
                checked={startupWord === entry.word}
                onChange={() => {
                  onStartPageChange(entry.word);
                }}
              />
              <label htmlFor={`fs-start-${entry.word}`}>{t('settings', entry.labelKey)}</label>
            </span>
          ))}
        </div>
      </section>

      <section className={styles['section']} aria-label={t('settings', 'reset.heading')}>
        <h2>{t('settings', 'reset.heading')}</h2>
        <p className={styles['note']}>{t('settings', 'reset.fact')}</p>
        {confirming ? (
          <div className={styles['confirm']} role="group" aria-labelledby="fs-reset-confirm">
            <h3 id="fs-reset-confirm">{t('settings', 'reset.confirm')}</h3>
            <p className={styles['note']}>{t('settings', 'reset.confirmFact')}</p>
            <div className={styles['actions']}>
              <button
                type="button"
                className={styles['control']}
                autoFocus
                onClick={() => {
                  setConfirming(false);
                  onReset();
                }}
              >
                {t('settings', 'reset.confirmButton')}
              </button>
              <button
                type="button"
                className={styles['control']}
                onClick={() => {
                  setConfirming(false);
                }}
              >
                {t('settings', 'reset.cancel')}
              </button>
            </div>
          </div>
        ) : (
          <div className={styles['actions']}>
            <button
              type="button"
              className={styles['control']}
              onClick={() => {
                setConfirming(true);
              }}
            >
              {t('settings', 'reset.trigger')}
            </button>
          </div>
        )}
      </section>
    </main>
  );
}
