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
 */

import { useCallback, useEffect, useState } from 'react';

import styles from './Help.module.css';
import { ErrorPanel } from './components/ErrorPanel';
import { GettingStartedList } from './GettingStarted';
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
 */
const DOCUMENTS: readonly { readonly name: string; readonly path: string; readonly shipped: boolean }[] = [
  { name: 'Getting Started', path: 'this screen, and the panel on Analyze', shipped: true },
  { name: 'Support Matrix', path: '04_TECH/20_PLATFORM_SUPPORT.md', shipped: false },
  { name: 'Known Limitations', path: 'P5_VALIDATION/P5_KNOWN_LIMITATIONS.md', shipped: false },
  { name: 'Diagnostics', path: 'this screen, and the file you export from it', shipped: true },
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
): string {
  return identity === null ? 'not reported' : pick(identity);
}

/** The same rule for the Diagnostics rows: the screen holds no default for any of them. */
function known(
  diagnostics: DiagnosticsDto | null,
  pick: (value: DiagnosticsDto) => string,
): string {
  return diagnostics === null ? 'not reported' : pick(diagnostics);
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
 */
function health(store: DiagnosticsStoreDto): string {
  switch (store.health) {
    case 'healthy':
      return 'healthy';
    case 'unhealthy':
      return 'unhealthy';
    default:
      return 'unknown';
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
 */
function healthNote(store: DiagnosticsStoreDto): string {
  if (store.health === 'unhealthy') {
    return `The store check reported: ${store.healthSummary ?? 'no detail returned'}. FirmwareSight only reads this check; it does not repair, replace or delete the store.`;
  }
  if (store.health === 'unknown') {
    const code = store.healthErrorCode === null ? '' : ` (code ${store.healthErrorCode})`;
    return `The health check could not be asked${code}. The store is left exactly where it is.`;
  }
  return '';
}

/**
 * What the Save dialog and the write actually did.
 *
 * Declining to choose a folder, and declining to overwrite a file that was already there, are normal
 * outcomes rather than errors - the Compare export established that contract, and Diagnostics keeps the
 * same words for the same decisions.
 */
function outcomeWords(outcome: ExportOutcomeDto): string {
  switch (outcome.status) {
    case 'written':
      return `Wrote ${outcome.fileName ?? 'the diagnostics file'}.`;
    case 'cancelled':
      return 'Export cancelled. No file was written.';
    case 'kept-existing':
      return 'Kept the file that was already there. Nothing was written.';
    default:
      // An outcome this build does not know is reported as what it is, not as a guess.
      return `The export reported an outcome this build does not know: ${outcome.status}`;
  }
}

export function Help() {
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
        <h1>Help</h1>
        <p className={styles['subhead']}>
          What this application is, what it can read, and where the documents that bound its claims
          live. Nothing on this screen needs a browser or a network.
        </p>
      </header>

      <section className={styles['section']} aria-label="About this application">
        <h2>About this application</h2>
        {error !== null ? (
          <ErrorPanel
            envelope={error}
            label="Application identity error"
            heading="The application could not describe itself"
          />
        ) : null}
        <dl className={styles['facts']}>
          <div>
            <dt>Product</dt>
            <dd>{told(identity, (value) => value.productName)}</dd>
          </div>
          <div>
            <dt>Version</dt>
            <dd className={styles['mono']}>{told(identity, (value) => value.appVersion)}</dd>
          </div>
          <div>
            <dt>Executable</dt>
            <dd className={styles['mono']}>{told(identity, (value) => value.binaryName)}</dd>
          </div>
          <div>
            <dt>Application identifier</dt>
            <dd className={styles['mono']}>{told(identity, (value) => value.identifier)}</dd>
          </div>
          <div>
            <dt>Running on</dt>
            <dd className={styles['mono']}>{told(identity, (value) => value.platform)}</dd>
          </div>
          <div>
            <dt>Store schema</dt>
            <dd className={styles['mono']}>
              {told(identity, (value) => `v${String(value.storageSchemaVersion)}`)}
            </dd>
          </div>
          <div>
            <dt>History store</dt>
            <dd className={styles['mono']}>{told(identity, (value) => value.storeFileName)}</dd>
          </div>
        </dl>
        <p className={styles['note']}>
          The store is named by file, not by folder. Where it sits is not stated here, and not stated in
          the diagnostics file either: an absolute path stays inside the application.
        </p>
      </section>

      <Diagnostics />

      <section className={styles['section']} aria-label="What it can read">
        <h2>What it can read</h2>
        <p>
          FirmwareSight analyzes ELF firmware artifacts and, when one is attached, a GNU ld MAP file
          beside it. The Format row on Analyze names the architecture, bitness and endianness it found
          in the file you chose; the Parser row names the parser that read it. A MAP that is not GNU
          ld output is refused rather than partly believed.
        </p>
        <p>
          Anything outside that set is reported as unsupported. The Support Matrix document is the
          authoritative list of what is claimed to work, on which platform, and to what depth; this
          screen is not a substitute for it.
        </p>
      </section>

      <section className={styles['section']} aria-label="Local-first statement">
        <h2>Local first</h2>
        <p>
          Analysis, comparison, the release Gate and the bundle write all happen on this machine. The
          product has no account, no telemetry, no analytics SDK, no update service and no network
          listener, so there is nothing to send a firmware file to. Your firmware bytes are never
          copied into the application&rsquo;s own database: the store keeps the facts derived from
          them, and the file name.
        </p>
      </section>

      <section className={styles['section']} aria-label="Getting started">
        <h2>Getting started</h2>
        <p>
          The same seven answers Analyze shows on a first run. If you hid that panel, everything it
          said is here.
        </p>
        <GettingStartedList />
      </section>

      <section className={styles['section']} aria-label="Where the documents live">
        <h2>Where to find the rest</h2>
        <ul className={styles['documents']}>
          {DOCUMENTS.map((document) => (
            <li key={document.name}>
              <span className={styles['documentName']}>{document.name}</span>
              <span className={styles['mono']}>{document.path}</span>
              {document.shipped ? null : (
                <span className={styles['documentFlag']}>not carried inside the installed package</span>
              )}
            </li>
          ))}
        </ul>
        <p className={styles['note']}>
          History is a page of this window, not a document: the builds, Gate runs and release records
          this computer stored are on the History tab, and they stay readable after the firmware files
          themselves have gone.
        </p>
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
    setNote(outcomeWords(outcome.value));
  }, []);

  const store = payload?.store ?? null;
  const detail = store === null ? '' : healthNote(store);

  return (
    <section className={styles['section']} aria-label="Diagnostics">
      <h2>Diagnostics</h2>
      <p>
        One file that says what this application is and what its own data store can say about itself, so
        a support conversation can start from facts rather than from a person reading values off a
        screen. It is written here and goes nowhere else: there is no upload, no telemetry, and no
        service in this product to send it to.
      </p>
      {error === null ? null : (
        <ErrorPanel
          envelope={error}
          label="Diagnostics error"
          heading="The application could not describe its data store"
        />
      )}
      <dl className={styles['facts']}>
        <div>
          <dt>Version</dt>
          <dd className={styles['mono']}>
            {known(payload, (value) => value.product.appVersion)}
          </dd>
        </div>
        <div>
          <dt>Store schema</dt>
          <dd className={styles['mono']}>
            {known(payload, (value) =>
              value.store.schemaVersion === null
                ? 'not reported'
                : `v${String(value.store.schemaVersion)}`,
            )}
          </dd>
        </div>
        <div>
          <dt>Store health</dt>
          <dd className={styles['health']}>
            {known(payload, (value) => health(value.store))}
          </dd>
        </div>
        <div>
          <dt>Git</dt>
          <dd>
            {known(payload, (value) =>
              value.git.available ? 'available' : 'not available on this machine',
            )}
          </dd>
        </div>
        <div>
          <dt>Local FirmwareSight data store</dt>
          <dd className={styles['mono']}>
            {known(payload, (value) => value.product.storeFileName)}
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
          Export diagnostics
        </button>
        {exporting ? (
          <p className={styles['status']} role="status" aria-live="polite">
            Waiting for the save dialog&hellip;
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
          label="Diagnostics export error"
          heading="The diagnostics file was not written"
        />
      )}
      <p className={styles['note']}>
        The file carries these five facts and the ones this screen has no room for: journal mode, bounded
        row counts, the names of any pre-migration snapshots kept beside the store, the install channel,
        the loaded release policy, and up to eight stable error codes this run produced. Never a folder,
        a user name, a firmware byte, a MAP line, a release note, a symbol name, a Git remote, an SQL
        statement or a stack trace. Codes only, never the message that goes with one.
      </p>
    </section>
  );
}
