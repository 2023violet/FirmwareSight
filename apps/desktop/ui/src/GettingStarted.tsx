/**
 * The first-use guidance P5 asks for (prompt §13), in one place two surfaces read.
 *
 * §13 says what a stranger must be able to answer before they do anything: what this tool does, what
 * their first action is, what a MAP changes, what the four steps are, what the five state words mean,
 * that the analysis is local, and where a project's own policy enters. It also says what not to build:
 * no marketing tour, no modal trap, no compulsory wizard, and a preference for a high-quality empty
 * state over a multi-step flow.
 *
 * So this is a list of facts, not a sequence of screens. Analyze shows it as a dismissible panel
 * above its own empty state while nothing has been analyzed; Help shows the same seven items with no
 * dismissal, because "where is Getting Started" is a question with a second reading. One array feeds
 * both, so the two surfaces cannot give two different answers.
 */

import styles from './GettingStarted.module.css';
import { StateBadge, type StateName } from './components/StateBadge';

/** The five state words the product uses, with the sentence each one carries (DESIGN.md 5). */
const STATES: readonly { readonly name: StateName; readonly label: string; readonly meaning: string }[] = [
  { name: 'PASS', label: 'PASS', meaning: 'the rule was evaluated and is satisfied' },
  { name: 'REVIEW', label: 'REVIEW', meaning: 'evaluated, and a person has to dispose of it' },
  { name: 'BLOCK', label: 'BLOCK', meaning: 'evaluated and failed in a blocking way' },
  { name: 'UNKNOWN', label: 'UNKNOWN', meaning: 'the evidence to evaluate it never arrived' },
  { name: 'N/A', label: 'N/A', meaning: 'the rule does not apply to this configuration' },
];

/** The flow a reader is being oriented towards: four steps, one direction. */
const FLOW: readonly string[] = ['Analyze', 'Compare', 'Release', 'Bundle'];

export function GettingStartedList() {
  return (
    <dl className={styles['list']}>
      <div>
        <dt>What FirmwareSight does</dt>
        <dd>
          It reads one firmware artifact on this machine and reports the size and symbol facts that
          can be proved from it: &ldquo;Know exactly what ships.&rdquo;
        </dd>
      </div>
      <div>
        <dt>First action</dt>
        <dd>
          On Analyze, choose an ELF artifact with <em>Choose firmware artifact</em>. The file stays
          where it is; nothing is copied into a project folder.
        </dd>
      </div>
      <div>
        <dt>What a MAP file changes</dt>
        <dd>
          A GNU ld MAP next to the artifact is optional, and it improves region and layout evidence.
          Without it, some facts stay UNKNOWN instead of being guessed at.
        </dd>
      </div>
      <div>
        <dt>The product flow</dt>
        <dd>
          <ol className={styles['flow']}>
            {FLOW.map((step, index) => (
              <li key={step}>
                {index === 0 ? null : <span className={styles['arrow']}>then </span>}
                {step}
              </li>
            ))}
          </ol>
          Analyze proves one build, Compare measures two, Release judges them against a policy, and
          Bundle writes the result out as a folder a release owner can hand to somebody else.
        </dd>
      </div>
      <div>
        <dt>The state words</dt>
        <dd>
          <ul className={styles['states']}>
            {STATES.map((state) => (
              <li key={state.name}>
                <StateBadge state={state.name} label={state.label} />
                <span>{state.meaning}</span>
              </li>
            ))}
          </ul>
        </dd>
      </div>
      <div>
        <dt>Local first</dt>
        <dd>
          Firmware files are analyzed locally. The MVP requires no upload, no account and no internet
          connection: there is no service to send anything to.
        </dd>
      </div>
      <div>
        <dt>Where a project&rsquo;s policy enters</dt>
        <dd>
          A project keeps its rules in <code>firmwaresight.toml</code>. Opening that file is what the
          Release page does first: its budgets, required artifacts and review rules are what the Gate
          judges the stored build against, and the policy&rsquo;s own digest is recorded in the Gate
          run the release stands on.
        </dd>
      </div>
    </dl>
  );
}

/**
 * The panel Analyze shows while it holds nothing: the same seven facts, plus the way out.
 *
 * Dismissing is one click and hides the list for the session; it never blocks the page, and no
 * control on Analyze waits for it to be answered (§13&rsquo;s &ldquo;no modal trap&rdquo;, no
 * compulsory wizard).
 */
export function GettingStartedPanel({ onDismiss }: { readonly onDismiss: () => void }) {
  return (
    <section className={styles['panel']} aria-label="Getting started">
      <header className={styles['head']}>
        <h2>Getting started</h2>
        <button type="button" className={styles['dismiss']} onClick={onDismiss}>
          Hide this
        </button>
      </header>
      <p className={styles['lede']}>
        Seven facts that answer what a first run usually asks. Hiding them does not hide anything
        else: Help repeats them, and nothing here has to be read before Analyze works.
      </p>
      <GettingStartedList />
    </section>
  );
}
