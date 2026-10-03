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
 * The store is named, not located. Its full path belongs to Diagnostics (prompt §19), which has its
 * own privacy test and is not this commit.
 */

import { useEffect, useState } from 'react';

import styles from './Help.module.css';
import { ErrorPanel } from './components/ErrorPanel';
import { GettingStartedList } from './GettingStarted';
import { getAppIdentity } from './ipc/bridge';
import type { AppIdentityDto, ErrorEnvelopeDto } from './ipc/types';

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
  { name: 'Diagnostics', path: 'not in this build yet', shipped: false },
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
          The store is named by file, not by folder. Where it sits on this machine is a Diagnostics
          question, and Diagnostics is not in this build yet.
        </p>
      </section>

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
