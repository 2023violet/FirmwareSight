/**
 * The two words-from-Core-to-five-states mappings, in one file.
 *
 * Before U1 the Analyze page owned `capabilityState` privately and the Release page owned `stateName`
 * privately. The Overview page needs both - it states the capability of each input and the severity of
 * the latest gate run - and a third copy of a switch over the same vocabulary is how one screen ends up
 * calling a value PASS while another calls it UNKNOWN.
 *
 * Neither function decides anything. Core has already chosen the capability, the state and the severity
 * (`AGENTS.md` 3, 8); these turn those words into the five frozen presentation states and nothing else.
 */

import type { StateName } from './components/StateBadge';
import type { Translate } from './i18n';
import type { GateRunDto } from './ipc/types';

/** The five Gate state words, exactly. */
const STATE_NAMES: readonly string[] = ['PASS', 'REVIEW', 'BLOCK', 'UNKNOWN', 'N/A'];

/** The three severities `EffectiveSeverity::as_str` can emit. */
export function severityState(value: string): StateName {
  switch (value) {
    case 'PASS':
      return 'PASS';
    case 'REVIEW':
      return 'REVIEW';
    case 'BLOCK':
      return 'BLOCK';
    default:
      // An unmapped word is shown as the state that means "this cannot be evaluated" rather than
      // promoted to a verdict, which is the same choice the Gate page makes.
      return 'UNKNOWN';
  }
}

/** A finding's factual state. Anything outside the five words is UNKNOWN, never dropped. */
export function gateStateName(value: string): StateName {
  return STATE_NAMES.includes(value) ? (value as StateName) : 'UNKNOWN';
}

/**
 * Presentation mapping only: the shell already decided availability.
 *
 * `not-provided` is N/A rather than UNKNOWN because the reader chose not to supply that input; it is not
 * missing evidence, and calling it UNKNOWN would put a question mark on a decision.
 */
export function capabilityState(value: string): StateName {
  switch (value) {
    case 'supported':
    case 'available':
    case 'provided':
      return 'PASS';
    case 'partial':
      return 'REVIEW';
    case 'unsupported':
      return 'BLOCK';
    case 'unavailable':
    case 'not-provided':
      return 'N/A';
    default:
      return 'UNKNOWN';
  }
}

/**
 * The verdict sentence a gate run deserves, assembled from that run alone, in the language on screen.
 *
 * Two pages show a verdict - `Release` where the run was made, `Overview` where the latest one is
 * remembered - and a reader who sees both must not get two wordings of the same fact. The earlier draft
 * of Overview wrote "a blocked build never produces a bundle" while Release had always said "cannot"; the
 * two sentences meant the same thing and only one of them is the product rule, which is precisely the
 * drift this file exists to prevent. That is also why the wording is not a parameter: both pages call
 * this one builder.
 *
 * The counts are Core's aggregate and the severities are Core's dispositions (`AGENTS.md` 3, 8). What is
 * assembled here is prose over those numbers, plus the one case a bare severity word hides: when accepted
 * reviews moved the aggregate, `overallEffectiveSeverity` and `dispositionEffectiveSeverity` are two
 * different facts about the same findings, and naming only the second would let a reader believe the
 * build cleared on its own.
 *
 * The translator arrives as an argument rather than being read from context because this is a `.ts`
 * module with no React in it, and the sentence it builds has to be the same one on both pages.
 */
export function gateVerdictSentence(run: GateRunDto, t: Translate): string {
  const severity = severityState(run.dispositionEffectiveSeverity);
  const head = verdictHead(severity, run, t);
  const accepted = run.findings.filter((finding) => finding.acceptance !== null).length;
  if (accepted === 0 || run.overallEffectiveSeverity === run.dispositionEffectiveSeverity) {
    return head;
  }
  return t('gate', 'verdict.accepted', {
    head,
    accepted,
    overall: run.overallEffectiveSeverity,
  });
}

/**
 * The head names the findings that carry the aggregate. Those are the rows whose effective severity is
 * the aggregate's word, which is Core's own field on each row, so nothing here re-judges a rule. A rule
 * whose evidence is missing can carry a REVIEW or a BLOCK through `[gate.on_unknown]`, and counting only
 * REVIEW- or BLOCK-state rows would then read "0 findings wait" under a REVIEW disposition (R4 smoke).
 */
function verdictHead(severity: StateName, run: GateRunDto, t: Translate): string {
  const counts = run.counts;
  const carrying = (word: string) =>
    run.findings.filter((finding) => finding.effectiveSeverity === word && finding.acceptance === null);
  switch (severity) {
    case 'BLOCK': {
      const rows = carrying('BLOCK');
      const unknown = rows.filter((finding) => finding.state === 'UNKNOWN').length;
      return unknown === 0
        ? t('gate', 'verdict.block', { count: rows.length === 0 ? counts.block : rows.length })
        : t('gate', 'verdict.blockUnknown', { count: rows.length, unknown });
    }
    case 'REVIEW': {
      const rows = carrying('REVIEW');
      const unknown = rows.filter((finding) => finding.state === 'UNKNOWN').length;
      return unknown === 0
        ? t('gate', 'verdict.review', { count: rows.length === 0 ? counts.review : rows.length })
        : t('gate', 'verdict.reviewUnknown', { count: rows.length, unknown });
    }
    case 'UNKNOWN':
      return t('gate', 'verdict.unknown', { count: counts.unknown });
    default:
      return t('gate', 'verdict.pass', { count: counts.pass });
  }
}

/**
 * Whether the analysis itself was made with a MAP, from Core's own capability word - `null` when the word does
 * not say. R2 review S2-1: the SubjectBar must take the MAP fact from the result it is describing, not from the
 * selection, because a MAP added or removed after analyzing changes the selection and not the result.
 */
export function analyzedWithMap(summary: { readonly capabilities: { readonly map: string } }): boolean | null {
  switch (summary.capabilities.map) {
    case 'provided':
      return true;
    case 'not-provided':
      return false;
    default:
      return null;
  }
}

/** The selection's MAP differs from the one the analysis was made with: the result is not the selection's yet. */
export function mapChangedSince(
  summary: { readonly capabilities: { readonly map: string } },
  selection: { readonly mapAttached: boolean } | null,
): boolean {
  const analyzed = analyzedWithMap(summary);
  return selection !== null && analyzed !== null && selection.mapAttached !== analyzed;
}
