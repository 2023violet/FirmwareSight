/**
 * This application's own settings - and nothing the Gate judges.
 *
 * Three kinds of state live in three places, and this page holds only the first:
 *
 * - **Application preferences** (`settings.json`, this page): how this window labels a number and
 *   where the next start opens. They belong to the machine and to the person sitting at it.
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
 * Only controls that do something today are rendered. Theme and language are not here, because this
 * build has no second theme to switch to and one language on screen; a disabled switch that goes
 * nowhere would teach the reader that nothing on this page works (`AGENTS.md` 11, `DESIGN.md` 5).
 */

import { useState } from 'react';

import type { SizeUnit } from './format';
import styles from './Settings.module.css';
import { ErrorPanel } from './components/ErrorPanel';
import { SizeUnitSwitch } from './components/SizeUnitSwitch';
import type { ErrorEnvelopeDto, PreferenceRecoveryDto, PreferencesDto } from './ipc/types';

/**
 * Where a start opens, in the two forms this build can actually honour.
 *
 * "Last page I was on" is not among them: a restored page would bring back the *look* of an earlier
 * Gate run while the session behind it is gone, and a reader who mistook that for a current verdict
 * would be shown a false fact. A static choice cannot do that, so a static choice is what is offered.
 */
const START_PAGES: readonly { readonly word: string; readonly label: string }[] = [
  { word: 'analyze', label: 'Analyze' },
  { word: 'history', label: 'Bundle & History' },
];

/** The one sentence that says why the values on this page are what they are. */
function recoveryNote(recovery: PreferenceRecoveryDto, keptCopy: boolean): string | null {
  switch (recovery) {
    case 'read':
      return null;
    case 'missing':
      return 'Nothing has been stored yet, so this window is using FirmwareSight’s defaults.';
    case 'corrupted':
      return keptCopy
        ? 'The stored settings could not be read, so this window is using the defaults. The bytes that could not be read are kept beside them.'
        : 'The stored settings could not be read, so this window is using the defaults. Nothing was written, so the file that could not be read is still exactly as it was.';
    case 'futureVersion':
      return 'The stored settings were written by a newer version of FirmwareSight, so they are not read or re-written here. Reset below replaces them deliberately.';
    case 'nowhereToStore':
      return 'This host gave FirmwareSight no configuration folder, so nothing can be stored. The defaults are in use for this window, and a save will say it did not land.';
  }
}

export function Settings({
  preferences,
  error,
  note,
  unit,
  onUnitChange,
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
  readonly onStartPageChange: (word: string) => void;
  readonly onReset: () => void;
  readonly onGoToHelp: () => void;
}) {
  /** The reset decision is asked on the page and answered there, in the same shape Release asks a replace. */
  const [confirming, setConfirming] = useState(false);
  const stored = preferences !== null ? preferences.fileName : 'settings.json';
  const startupWord = preferences?.startupPage ?? 'analyze';

  return (
    <main className={styles['page']}>
      <header className={styles['header']}>
        <h1>Settings</h1>
        <p className={styles['subhead']}>
          What this window shows and where the next start opens. Nothing here changes what
          FirmwareSight judges about a build, and nothing here is sent anywhere.
        </p>
      </header>

      {error !== null ? (
        <ErrorPanel
          envelope={error}
          label="Preferences error"
          heading="The preferences were not stored"
        />
      ) : null}

      {preferences === null ? (
        <p className={styles['status']} role="status">
          Waiting for the shell to report this application’s settings…
        </p>
      ) : null}

      {note === null ? null : (
        <p className={styles['status']} role="status" aria-live="polite">
          {note}
        </p>
      )}

      <section className={styles['section']} aria-label="General">
        <h2>General</h2>
        {preferences === null ? null : (
          <p className={styles['facts']}>
            Settings are kept in <span className={styles['mono']}>{stored}</span> in this
            application’s own configuration folder, on this machine only.
          </p>
        )}
        {preferences === null ? null : (
          <p className={styles['note']}>
            {recoveryNote(preferences.recovery, preferences.keptUnreadableCopy)}
          </p>
        )}
        <p className={styles['note']}>
          Local first: the build you analyze is read from your own disk, and this application has no
          account, no upload and no service that receives anything. The same facts, with the version
          this binary carries and the diagnostics you can export, are on the Help page.
        </p>
        <div className={styles['actions']}>
          <button type="button" className={styles['control']} onClick={onGoToHelp}>
            Open Help and About
          </button>
        </div>
      </section>

      <section className={styles['section']} aria-label="Appearance">
        <h2>Appearance</h2>
        <p className={styles['note']}>
          Size units label the numbers on every table; they do not re-read or re-compute anything.
          Addresses, hashes, counts and identifiers are never converted.
        </p>
        <SizeUnitSwitch
          unit={unit}
          onSelect={(next) => {
            onUnitChange(next);
          }}
        />
      </section>

      <section className={styles['section']} aria-label="Start page">
        <h2>Start page</h2>
        <p className={styles['note']}>
          Which page the next start opens on. It takes effect the next time this window is opened, not
          now, and it never restores an earlier Gate run.
        </p>
        <div className={styles['radios']} role="radiogroup" aria-label="Start page">
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
              <label htmlFor={`fs-start-${entry.word}`}>{entry.label}</label>
            </span>
          ))}
        </div>
      </section>

      <section className={styles['section']} aria-label="Reset">
        <h2>Reset application preferences</h2>
        <p className={styles['note']}>
          Puts the two settings above back to this version’s defaults. It touches only the
          application’s own settings file: analyzed builds, Gate runs and release records stay in the
          data store, and the files you chose stay where they are.
        </p>
        {confirming ? (
          <div className={styles['confirm']} role="group" aria-labelledby="fs-reset-confirm">
            <h3 id="fs-reset-confirm">Reset both settings to their defaults?</h3>
            <p className={styles['note']}>
              Size units go back to bytes and the start page goes back to Analyze.
            </p>
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
                Reset preferences
              </button>
              <button
                type="button"
                className={styles['control']}
                onClick={() => {
                  setConfirming(false);
                }}
              >
                Keep them
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
              Reset application preferences
            </button>
          </div>
        )}
      </section>
    </main>
  );
}
