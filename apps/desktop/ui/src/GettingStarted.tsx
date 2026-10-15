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
 *
 * The words come from the `analyze` catalog (`ADR-0032`), and the two inline elements the sentences are
 * built around — the button's own name in an `em`, the project's file name in a `code` — stay elements
 * rather than catalog text, so the sentence is split into the parts on either side of them. A catalog
 * value never holds markup, and the file name and the button label are not prose to translate: the
 * first is a name and the second is the label of the control the reader is being pointed at.
 */

import styles from './GettingStarted.module.css';
import { StateBadge, type StateName } from './components/StateBadge';
import { useI18n } from './i18n';
import type { Catalog } from './i18n/en';

/** A key the `analyze` namespace actually holds, so a typo is a compile error rather than a runtime hole. */
type AnalyzeKey = keyof Catalog['analyze'] & string;

/** The five state words the product uses, with the catalog key of the sentence each one carries (DESIGN.md 5). */
const STATES: readonly {
  readonly name: StateName;
  readonly label: string;
  readonly meaning: AnalyzeKey;
}[] = [
  { name: 'PASS', label: 'PASS', meaning: 'gettingStarted.statePass' },
  { name: 'REVIEW', label: 'REVIEW', meaning: 'gettingStarted.stateReview' },
  { name: 'BLOCK', label: 'BLOCK', meaning: 'gettingStarted.stateBlock' },
  { name: 'UNKNOWN', label: 'UNKNOWN', meaning: 'gettingStarted.stateUnknown' },
  { name: 'N/A', label: 'N/A', meaning: 'gettingStarted.stateNa' },
];

/** The flow a reader is being oriented towards: four steps, one direction, named by the pages they are. */
const FLOW: readonly AnalyzeKey[] = [
  'gettingStarted.stepAnalyze',
  'gettingStarted.stepCompare',
  'gettingStarted.stepRelease',
  'gettingStarted.stepBundle',
];

export function GettingStartedList() {
  const { t } = useI18n();
  return (
    <dl className={styles['list']}>
      <div>
        <dt>{t('analyze', 'gettingStarted.whatTitle')}</dt>
        <dd>{t('analyze', 'gettingStarted.whatBody')}</dd>
      </div>
      <div>
        <dt>{t('analyze', 'gettingStarted.firstActionTitle')}</dt>
        <dd>
          {t('analyze', 'gettingStarted.firstActionBefore')}
          <em>{t('analyze', 'intake.chooseArtifact')}</em>
          {t('analyze', 'gettingStarted.firstActionAfter')}
        </dd>
      </div>
      <div>
        <dt>{t('analyze', 'gettingStarted.mapTitle')}</dt>
        <dd>{t('analyze', 'gettingStarted.mapBody')}</dd>
      </div>
      <div>
        <dt>{t('analyze', 'gettingStarted.flowTitle')}</dt>
        <dd>
          <ol className={styles['flow']}>
            {FLOW.map((step, index) => (
              <li key={step}>
                {index === 0 ? null : (
                  <span className={styles['arrow']}>{t('analyze', 'gettingStarted.stepThen')}</span>
                )}
                {t('analyze', step)}
              </li>
            ))}
          </ol>
          {t('analyze', 'gettingStarted.flowBody')}
        </dd>
      </div>
      <div>
        <dt>{t('analyze', 'gettingStarted.statesTitle')}</dt>
        <dd>
          <ul className={styles['states']}>
            {STATES.map((state) => (
              <li key={state.name}>
                <StateBadge state={state.name} label={state.label} />
                <span>{t('analyze', state.meaning)}</span>
              </li>
            ))}
          </ul>
        </dd>
      </div>
      <div>
        <dt>{t('analyze', 'gettingStarted.localTitle')}</dt>
        <dd>{t('analyze', 'gettingStarted.localBody')}</dd>
      </div>
      <div>
        <dt>{t('analyze', 'gettingStarted.policyTitle')}</dt>
        <dd>
          {t('analyze', 'gettingStarted.policyBefore')}
          <code>firmwaresight.toml</code>
          {t('analyze', 'gettingStarted.policyAfter')}
        </dd>
      </div>
    </dl>
  );
}

/**
 * The panel Analyze shows while it holds nothing: the same seven facts, plus the way out.
 *
 * Dismissing is one click and hides the list for the session; it never blocks the page, and no
 * control on Analyze waits for it to be answered (§13’s “no modal trap”, no
 * compulsory wizard).
 */
/**
 * The first-use panel above Analyze's empty state (M1 T1): three actions, not an essay. The seven reference
 * facts live in Help, which is where a reader who wants the why goes; this panel only gets the first run
 * done.
 */
export function GettingStartedPanel({ onDismiss }: { readonly onDismiss: () => void }) {
  const { t } = useI18n();
  return (
    <section className={styles['panel']} aria-label={t('analyze', 'gettingStarted.title')}>
      <header className={styles['head']}>
        <h2>{t('analyze', 'gettingStarted.title')}</h2>
        <button type="button" className={styles['dismiss']} onClick={onDismiss}>
          {t('analyze', 'gettingStarted.dismiss')}
        </button>
      </header>
      <ol className={styles['steps']}>
        <li>
          <strong>{t('analyze', 'gettingStarted.step1Title')}</strong>
          <span>{t('analyze', 'gettingStarted.step1Body')}</span>
        </li>
        <li>
          <strong>{t('analyze', 'gettingStarted.step2Title')}</strong>
          <span>{t('analyze', 'gettingStarted.step2Body')}</span>
        </li>
        <li>
          <strong>{t('analyze', 'gettingStarted.step3Title')}</strong>
          <span>{t('analyze', 'gettingStarted.step3Body')}</span>
        </li>
      </ol>
      <p className={styles['lede']}>{t('analyze', 'gettingStarted.moreInHelp')}</p>
    </section>
  );
}
