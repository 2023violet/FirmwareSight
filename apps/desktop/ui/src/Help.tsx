/**
 * The local Help and About surface prompt §14 asks for.
 *
 * It is offline, it is the whole product's own words, and it requires no browser: every line here is
 * either a fact the running application reported or a pointer to a document this repository ships.
 * There is deliberately no external link on the screen, because an installed app has no link to give
 * that a person without the source could follow (§14 allows optional links only; none are earned yet).
 *
 * The identity block is the reason this page calls the shell at all. A version written into the front
 * end would be a second source of truth and would drift the moment the workspace version moved, which
 * is exactly the failure P5 spent a decision on (D1: the workspace version is the single source). So
 * the version, the identifier, the platform, the schema the store carries and the store's file name
 * come from the running binary, and the screen says plainly when it has not received them.
 *
 * The store is named, not located - and the Diagnostics section below does not locate it either. An
 * absolute path is outside that payload by rule (prompt §19), so no surface this product has prints the
 * folder the store sits in.
 *
 * Diagnostics is placed on this page rather than on a fifth navigation verb (prompt §5), and what it
 * shows is a subset of what the exported file says, so a reader can check the facts before writing them
 * down. Two boundaries are the point of that section: the payload is assembled in Rust from a fixed list
 * of fields, and it leaves the machine only when a person names a folder in the native Save dialog the
 * shell owns. Nothing is uploaded, because there is nothing here to upload it to.
 *
 * Every sentence on this page is UI prose and comes from the `help` catalog (`ADR-0032` D6), in either
 * language. What does not come from it is the shell's own answer - the version, the identifier, the
 * platform, the schema number, the store's file name, the engine's bounded health detail, an error code
 * and an export outcome word - which is rendered exactly as it arrives, through a placeholder when the
 * sentence needs one. Two source-tree document paths are kept verbatim for the same reason
 * (`ADR-0032` D5): a path is not a sentence.
 */

import { useCallback, useEffect, useState } from 'react';

import styles from './Help.module.css';
import { DataTable } from './components/DataTable';
import { ErrorPanel } from './components/ErrorPanel';
import { GettingStartedList } from './GettingStarted';
import { useI18n, type Translate } from './i18n';
import { exportDiagnostics, getAppIdentity, getDiagnostics } from './ipc/bridge';
import type {
  AppIdentityDto,
  DiagnosticsDto,
  DiagnosticsStoreDto,
  ErrorEnvelopeDto,
  ExportOutcomeDto,
} from './ipc/types';

/**
 * Where each of the four documents the reader is pointed to lives, and whether it is on this machine.
 *
 * A desktop package installs a binary. The four documents below are part of the product's source
 * distribution, so an honest Help screen says which folder they are in and says that the installed
 * package does not carry them yet. Claiming otherwise would be a pointer a stranger cannot follow.
 *
 * The two entries that live on this screen are pointed at in prose, so their location is a catalog key.
 * The two that live in the source tree are pointed at by their own path, and a path is machine
 * vocabulary (`ADR-0032` D5): it is shown in either language exactly as the repository spells it.
 */
const DOCUMENTS: readonly {
  readonly nameKey:
    | 'documents.name.gettingStarted'
    | 'documents.name.supportMatrix'
    | 'documents.name.knownLimitations'
    | 'documents.name.diagnostics';
  readonly at:
    | { readonly kind: 'prose'; readonly key: 'documents.where.onThisScreen' | 'documents.where.exportedHere' }
    | { readonly kind: 'path'; readonly text: string };
  readonly shipped: boolean;
}[] = [
  {
    nameKey: 'documents.name.gettingStarted',
    at: { kind: 'prose', key: 'documents.where.onThisScreen' },
    shipped: true,
  },
  {
    nameKey: 'documents.name.supportMatrix',
    at: { kind: 'path', text: '04_TECH/20_PLATFORM_SUPPORT.md' },
    shipped: false,
  },
  {
    nameKey: 'documents.name.knownLimitations',
    at: { kind: 'path', text: 'P5_VALIDATION/P5_KNOWN_LIMITATIONS.md' },
    shipped: false,
  },
  {
    nameKey: 'documents.name.diagnostics',
    at: { kind: 'prose', key: 'documents.where.exportedHere' },
    shipped: true,
  },
];

/**
 * One identity row, or the sentence that says this screen has not been told.
 *
 * Every row takes the same shape on purpose: a dash would read as "this application has no version",
 * which is a different and untrue claim, and a copy kept here would be a second source of truth the
 * moment the workspace version moved.
 */
function told(
  identity: AppIdentityDto | null,
  pick: (value: AppIdentityDto) => string,
  t: Translate,
): string {
  return identity === null ? t('help', 'notReported') : pick(identity);
}

/** The same rule for the Diagnostics rows: the screen holds no default for any of them. */
function known(
  diagnostics: DiagnosticsDto | null,
  pick: (value: DiagnosticsDto) => string,
  t: Translate,
): string {
  return diagnostics === null ? t('help', 'notReported') : pick(diagnostics);
}

/**
 * The store's health, as a word.
 *
 * Deliberately prose rather than one of the five state badges: PASS and BLOCK on this page would read
 * as a release verdict the Gate was never asked to make, the same reasoning that keeps the document
 * list below in plain grey (DESIGN.md 5). Carrying that badge's *colours* without its icon would be the
 * worst of both — a value that looks like a verdict and is not one — so health is bold prose in the
 * same grey every other fact on this page uses, and the sentence under it is what makes a damaged store
 * legible. A health value this build does not know is reported as unknown rather than rounded to
 * healthy.
 *
 * Being prose, the three words are the page's own and are catalog-keyed like every other sentence here;
 * none of them is one of the state or evidence words `ADR-0032` D5 keeps in the engine's spelling.
 */
function health(store: DiagnosticsStoreDto, t: Translate): string {
  switch (store.health) {
    case 'healthy':
      return t('help', 'health.healthy');
    case 'unhealthy':
      return t('help', 'health.unhealthy');
    default:
      return t('help', 'health.unknown');
  }
}

/**
 * The one sentence a health value earns: the engine's own bounded detail, and what the product will
 * not do about it.
 *
 * `integrity_check` is a read (`AGENTS.md` 9): FirmwareSight never repairs, replaces or deletes a store
 * it finds damaged, so the screen says that instead of offering a button that would. A check that could
 * not be asked shows its stable code and no message, because the message of an open failure can quote
 * the path it failed on (prompt §19).
 *
 * The detail and the code are handed to the sentence through a placeholder, so the engine's own words
 * land inside a translated sentence without that sentence being cut in half to translate them
 * (`ADR-0032` D6).
 */
function healthNote(store: DiagnosticsStoreDto, t: Translate): string {
  if (store.health === 'unhealthy') {
    return store.healthSummary === null
      ? t('help', 'health.damagedNoDetail')
      : t('help', 'health.damagedReported', { detail: store.healthSummary });
  }
  if (store.health === 'unknown') {
    return store.healthErrorCode === null
      ? t('help', 'health.notAsked')
      : t('help', 'health.notAskedWithCode', { code: store.healthErrorCode });
  }
  return '';
}

/**
 * What the Save dialog and the write actually did.
 *
 * Declining to choose a folder, and declining to overwrite a file that was already there, are normal
 * outcomes rather than errors - the Compare export established that contract, and Diagnostics keeps the
 * same words for the same decisions.
 *
 * `written` without a file name gets its own sentence rather than a noun phrase dropped into this one,
 * because a catalog value is a whole sentence and never a fragment of somebody else's
 * (`ADR-0032` D4). An outcome word this build does not know is the shell's data and is shown as it
 * arrived.
 */
function outcomeWords(outcome: ExportOutcomeDto, t: Translate): string {
  switch (outcome.status) {
    case 'written':
      return outcome.fileName === null
        ? t('help', 'export.wroteUnnamed')
        : t('help', 'export.wrote', { fileName: outcome.fileName });
    case 'cancelled':
      return t('help', 'export.cancelled');
    case 'kept-existing':
      return t('help', 'export.keptExisting');
    default:
      // An outcome this build does not know is reported as what it is, not as a guess.
      return t('help', 'export.unknownOutcome', { status: outcome.status });
  }
}

/** The keyboard model of spec §3.3, as the shell wires it (R5). Keys are written as the reader presses them. */
const SHORTCUTS: readonly { readonly keys: string; readonly does: 'shortcuts.palette' | 'shortcuts.choose' | 'shortcuts.filter' | 'shortcuts.views' | 'shortcuts.escape' }[] = [
  { keys: 'Ctrl+K', does: 'shortcuts.palette' },
  { keys: 'Ctrl+O', does: 'shortcuts.choose' },
  { keys: '/', does: 'shortcuts.filter' },
  { keys: '[  ]', does: 'shortcuts.views' },
  { keys: 'Esc', does: 'shortcuts.escape' },
];

export function Help() {
  const { t } = useI18n();
  const [identity, setIdentity] = useState<AppIdentityDto | null>(null);
  const [error, setError] = useState<ErrorEnvelopeDto | null>(null);

  useEffect(() => {
    // The shell owns the window title, so this page asks for one fact only: what the running
    // application says it is.
    void getAppIdentity().then((outcome) => {
      if (outcome.ok) {
        setIdentity(outcome.value);
        setError(null);
      } else {
        setError(outcome.envelope);
      }
    });
  }, []);

  return (
    <main className={styles['page']}>
      <header className={styles['header']}>
        <h1>{t('help', 'title')}</h1>
        <p className={styles['subhead']}>{t('help', 'intro')}</p>
      </header>

      <section className={styles['section']} aria-label={t('help', 'about.heading')}>
        <h2>{t('help', 'about.heading')}</h2>
        {error !== null ? (
          <ErrorPanel
            envelope={error}
            label={t('help', 'about.errorLabel')}
            heading={t('help', 'about.errorHeading')}
          />
        ) : null}
        <dl className={styles['facts']}>
          <div>
            <dt>{t('help', 'field.product')}</dt>
            <dd>{told(identity, (value) => value.productName, t)}</dd>
          </div>
          <div>
            <dt>{t('help', 'field.version')}</dt>
            <dd className={styles['mono']}>{told(identity, (value) => value.appVersion, t)}</dd>
          </div>
          <div>
            <dt>{t('help', 'field.executable')}</dt>
            <dd className={styles['mono']}>{told(identity, (value) => value.binaryName, t)}</dd>
          </div>
          <div>
            <dt>{t('help', 'field.identifier')}</dt>
            <dd className={styles['mono']}>{told(identity, (value) => value.identifier, t)}</dd>
          </div>
          <div>
            <dt>{t('help', 'field.runningOn')}</dt>
            <dd className={styles['mono']}>{told(identity, (value) => value.platform, t)}</dd>
          </div>
          <div>
            <dt>{t('help', 'field.storeSchema')}</dt>
            <dd className={styles['mono']}>
              {told(identity, (value) => `v${String(value.storageSchemaVersion)}`, t)}
            </dd>
          </div>
          <div>
            <dt>{t('help', 'field.historyStore')}</dt>
            <dd className={styles['mono']}>{told(identity, (value) => value.storeFileName, t)}</dd>
          </div>
        </dl>
        <p className={styles['note']}>{t('help', 'about.storeNote')}</p>
      </section>

      <Diagnostics />

      <section className={styles['section']} aria-label={t('help', 'reads.heading')}>
        <h2>{t('help', 'reads.heading')}</h2>
        <p>{t('help', 'reads.artifacts')}</p>
        <p>{t('help', 'reads.matrix')}</p>
        <p>{t('help', 'reads.binHex')}</p>
      </section>

      <section className={styles['section']} aria-label={t('help', 'local.ariaLabel')}>
        <h2>{t('help', 'local.heading')}</h2>
        <p>{t('help', 'local.boundary')}</p>
      </section>

      <section className={styles['section']} aria-label={t('help', 'getting.heading')}>
        <h2>{t('help', 'getting.heading')}</h2>
        <p>{t('help', 'getting.intro')}</p>
        <GettingStartedList />
      </section>

      <section className={styles['section']} aria-label={t('help', 'shortcuts.heading')}>
        <h2>{t('help', 'shortcuts.heading')}</h2>
        <DataTable
          caption={t('help', 'shortcuts.caption')}
          columns={[
            { key: 'keys', header: t('help', 'shortcuts.colKeys'), cell: (row) => <kbd className={styles['mono']}>{row.keys}</kbd> },
            { key: 'does', header: t('help', 'shortcuts.colDoes'), cell: (row) => t('help', row.does) },
          ]}
          rows={SHORTCUTS}
          rowKey={(row) => row.keys}
          density="compact"
        />
      </section>

      <section className={styles['section']} aria-label={t('help', 'documents.ariaLabel')}>
        <h2>{t('help', 'documents.heading')}</h2>
        <ul className={styles['documents']}>
          {DOCUMENTS.map((document) => (
            <li key={document.nameKey}>
              <span className={styles['documentName']}>{t('help', document.nameKey)}</span>
              <span className={styles['mono']}>
                {document.at.kind === 'prose' ? t('help', document.at.key) : document.at.text}
              </span>
              {document.shipped ? null : (
                <span className={styles['documentFlag']}>{t('help', 'documents.notCarried')}</span>
              )}
            </li>
          ))}
        </ul>
        <p className={styles['note']}>{t('help', 'documents.historyNote')}</p>
      </section>
    </main>
  );
}

/**
 * The Diagnostics section: five facts, one action, and the file those facts become.
 *
 * It owns its own IPC call and its own state, so an identity read that has not arrived cannot blank
 * this section and a diagnostics read that failed cannot blank the About block above it. The Version
 * row repeats the About block on purpose - it is the payload's copy of the same answer, read from
 * `collect_diagnostics` rather than from `get_app_identity`, and it is the number that will be written
 * into the file.
 */
function Diagnostics() {
  const { t } = useI18n();
  const [payload, setPayload] = useState<DiagnosticsDto | null>(null);
  const [error, setError] = useState<ErrorEnvelopeDto | null>(null);
  const [exporting, setExporting] = useState(false);
  const [note, setNote] = useState<string | null>(null);
  const [exportError, setExportError] = useState<ErrorEnvelopeDto | null>(null);

  useEffect(() => {
    void getDiagnostics().then((outcome) => {
      if (outcome.ok) {
        setPayload(outcome.value);
        setError(null);
      } else {
        setError(outcome.envelope);
      }
    });
  }, []);

  const exportNow = useCallback(async () => {
    setExporting(true);
    const outcome = await exportDiagnostics();
    setExporting(false);
    if (!outcome.ok) {
      // A failed export says nothing about the facts above, so the section stays standing.
      setExportError(outcome.envelope);
      setNote(null);
      return;
    }
    setExportError(null);
    setNote(outcomeWords(outcome.value, t));
  }, [t]);

  const store = payload?.store ?? null;
  const detail = store === null ? '' : healthNote(store, t);

  return (
    <section className={styles['section']} aria-label={t('help', 'diagnostics.heading')}>
      <h2>{t('help', 'diagnostics.heading')}</h2>
      <p>{t('help', 'diagnostics.intro')}</p>
      {error === null ? null : (
        <ErrorPanel
          envelope={error}
          label={t('help', 'diagnostics.errorLabel')}
          heading={t('help', 'diagnostics.errorHeading')}
        />
      )}
      <dl className={styles['facts']}>
        <div>
          <dt>{t('help', 'field.version')}</dt>
          <dd className={styles['mono']}>
            {known(payload, (value) => value.product.appVersion, t)}
          </dd>
        </div>
        <div>
          <dt>{t('help', 'field.storeSchema')}</dt>
          <dd className={styles['mono']}>
            {known(
              payload,
              (value) =>
                value.store.schemaVersion === null
                  ? t('help', 'notReported')
                  : `v${String(value.store.schemaVersion)}`,
              t,
            )}
          </dd>
        </div>
        <div>
          <dt>{t('help', 'field.storeHealth')}</dt>
          <dd className={styles['health']}>{known(payload, (value) => health(value.store, t), t)}</dd>
        </div>
        <div>
          <dt>{t('help', 'field.git')}</dt>
          <dd>
            {known(
              payload,
              (value) =>
                value.git.available ? t('help', 'git.available') : t('help', 'git.unavailable'),
              t,
            )}
          </dd>
        </div>
        <div>
          <dt>{t('help', 'field.localStore')}</dt>
          <dd className={styles['mono']}>
            {known(payload, (value) => value.product.storeFileName, t)}
          </dd>
        </div>
      </dl>
      {detail === '' ? null : <p className={styles['note']}>{detail}</p>}
      <div className={styles['exports']}>
        <button
          type="button"
          className={styles['control']}
          disabled={exporting}
          onClick={() => {
            void exportNow();
          }}
        >
          {t('help', 'diagnostics.exportButton')}
        </button>
        {exporting ? (
          <p className={styles['status']} role="status" aria-live="polite">
            {t('help', 'diagnostics.waitingForDialog')}
          </p>
        ) : null}
        {note === null ? null : (
          <p className={styles['status']} role="status" aria-live="polite">
            {note}
          </p>
        )}
      </div>
      {exportError === null ? null : (
        <ErrorPanel
          envelope={exportError}
          label={t('help', 'export.errorLabel')}
          heading={t('help', 'export.errorHeading')}
        />
      )}
      <p className={styles['note']}>{t('help', 'diagnostics.fileContents')}</p>
    </section>
  );
}
